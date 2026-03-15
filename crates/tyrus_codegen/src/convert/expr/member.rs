//! Member expression code generation.
//!
//! Handles property access: `obj.prop`, `this.field`, enum access (`Status::Active`),
//! computed properties (`arr[0]`), and Mutex-wrapped state fields.

use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use swc_ecma_ast::*;

use crate::convert::helpers::to_snake_case;
use crate::convert::interface::RustGenerator;

impl RustGenerator {
    pub(crate) fn convert_member_expr(&self, member: &MemberExpr) -> TokenStream {
        if member.obj.is_this() {
            if let Some(prop_ident) = member.prop.as_ident() {
                return self.convert_this_member(prop_ident);
            }
        }

        let obj = self.convert_expr(&member.obj);
        match &member.prop {
            MemberProp::Ident(ident) => self.convert_ident_member(&member.obj, &obj, ident),
            MemberProp::Computed(computed) => Self::convert_computed_member(&obj, computed, self),
            _ => quote! { compile_error!("Tyrus: unsupported member expression") },
        }
    }

    /// Handles `this.field` access, including Mutex-wrapped state fields.
    fn convert_this_member(&self, prop_ident: &IdentName) -> TokenStream {
        let prop_name = to_snake_case(prop_ident.sym.as_ref());
        let field_ident = format_ident!("{}", prop_name);

        // Use `state` for handler bodies (State<Arc<Self>> pattern)
        let receiver = if self.use_state_for_this.get() {
            quote! { state }
        } else {
            quote! { self }
        };

        let Some(type_str) = self.current_class_state_fields.get(prop_ident.sym.as_ref()) else {
            return quote! { #receiver.#field_ident.clone() };
        };

        let needs_deref = matches!(
            type_str.as_str(),
            "f64" | "bool" | "i32" | "usize" | "u64" | "i64"
        );

        if needs_deref {
            quote! { *#receiver.#field_ident.lock().unwrap_or_else(|e| e.into_inner()) }
        } else {
            quote! { #receiver.#field_ident.lock().unwrap_or_else(|e| e.into_inner()).clone() }
        }
    }

    /// Handles named property access: `obj.prop`, `Math.PI`, enum variants.
    fn convert_ident_member(
        &self,
        obj_expr: &Expr,
        obj: &TokenStream,
        ident: &IdentName,
    ) -> TokenStream {
        let name = ident.sym.as_ref();

        if name == "length" {
            return quote! { #obj.len() as f64 };
        }

        // Map/Set .size → .len()
        if name == "size" {
            if let Expr::Ident(ident) = obj_expr {
                let snake = to_snake_case(ident.sym.as_ref());
                let is_collection = self.map_vars.borrow().contains(&snake)
                    || self.set_vars.borrow().contains(&snake);
                if is_collection {
                    return quote! { #obj.len() as f64 };
                }
            }
        }

        if let Some(constant) = Self::try_math_constant(obj_expr, name) {
            return constant;
        }

        let prop_name = if name.chars().next().is_some_and(char::is_uppercase) {
            name.to_string()
        } else {
            to_snake_case(name)
        };
        let prop = format_ident!("{}", prop_name);

        // Getter call-site: obj.prop → obj.prop() when prop is a known getter
        if self.getter_names.contains(name) && !Self::is_enum_or_static_access(obj) {
            return quote! { #obj.#prop() };
        }

        if Self::is_enum_or_static_access(obj) {
            quote! { #obj::#prop }
        } else {
            quote! { #obj.#prop }
        }
    }

    /// Returns the Rust constant for `Math.PI` or `Math.E`, if applicable.
    fn try_math_constant(obj_expr: &Expr, prop_name: &str) -> Option<TokenStream> {
        let Expr::Ident(obj_ident) = obj_expr else {
            return None;
        };
        if obj_ident.sym.as_ref() != "Math" {
            return None;
        }
        match prop_name {
            "PI" => Some(quote! { std::f64::consts::PI }),
            "E" => Some(quote! { std::f64::consts::E }),
            _ => None,
        }
    }

    /// Detects PascalCase identifiers that map to Rust enum/static access (`Foo::Bar`).
    fn is_enum_or_static_access(obj: &TokenStream) -> bool {
        let obj_str = obj.to_string();
        obj_str.chars().next().is_some_and(char::is_uppercase)
            && !obj_str.contains('.')
            && !obj_str.contains('(')
    }

    /// Handles computed member access: `arr[0]`, `obj[expr]`.
    fn convert_computed_member(
        obj: &TokenStream,
        computed: &ComputedPropName,
        gen: &RustGenerator,
    ) -> TokenStream {
        let mut expr = &*computed.expr;
        while let Expr::Paren(p) = expr {
            expr = &p.expr;
        }

        if let Expr::Lit(Lit::Num(n)) = expr {
            let idx = n.value as usize;
            quote! { #obj[#idx].clone() }
        } else {
            let expr_tokens = gen.convert_expr(&computed.expr);
            quote! { #obj[#expr_tokens].clone() }
        }
    }
}
