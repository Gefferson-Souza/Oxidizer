//! Miscellaneous expression code generation.
//!
//! Handles: assignment expressions, update expressions (++/--),
//! and optional chaining (?.).

use proc_macro2::{Ident, TokenStream};
use quote::{format_ident, quote};
use swc_ecma_ast::*;

use crate::convert::helpers::to_snake_case;
use crate::convert::interface::RustGenerator;

/// Resolved shape of an assignment LHS, decoupled from operator dispatch.
enum AssignLhs {
    /// `this.field = …` where the field is a tracked state field — the emitter
    /// must split read/write to avoid double-locking the same `Mutex`.
    StateField { receiver: TokenStream, field: Ident },
    /// `obj.prop = v` where `prop` has a setter — emit `obj.set_prop(v)`.
    /// Compound assigns never reach this variant.
    Setter { obj: TokenStream, setter: Ident },
    /// Regular LHS expression that can be combined with any operator verbatim.
    Plain(TokenStream),
    /// Unsupported pattern — `compile_error!` token stream.
    Invalid(TokenStream),
}

impl RustGenerator {
    pub(crate) fn convert_assign_expr(&self, assign: &AssignExpr) -> TokenStream {
        let right = self.convert_expr(&assign.right);
        match self.resolve_assign_lhs(&assign.left, assign.op) {
            AssignLhs::StateField { receiver, field } => quote! {
                {
                    let __new_val = #right;
                    *#receiver.#field.lock().unwrap_or_else(|e| e.into_inner()) = __new_val;
                }
            },
            AssignLhs::Setter { obj, setter } => quote! { #obj.#setter(#right) },
            AssignLhs::Plain(lhs) => self.emit_assign_op(assign.op, &lhs, &right),
            AssignLhs::Invalid(err) => err,
        }
    }

    fn resolve_assign_lhs(&self, target: &AssignTarget, op: AssignOp) -> AssignLhs {
        let simple = match target {
            AssignTarget::Simple(simple) => simple,
            _ => {
                return AssignLhs::Invalid(
                    quote! { compile_error!("Tyrus: unsupported assignment pattern") },
                );
            }
        };
        match simple {
            SimpleAssignTarget::Member(member) => self.resolve_member_lhs(member, op),
            SimpleAssignTarget::Ident(ident) => {
                let name = format_ident!("{}", to_snake_case(ident.sym.as_ref()));
                AssignLhs::Plain(quote! { #name })
            }
            _ => AssignLhs::Invalid(
                quote! { compile_error!("Tyrus: unsupported assignment target") },
            ),
        }
    }

    fn resolve_member_lhs(&self, member: &MemberExpr, op: AssignOp) -> AssignLhs {
        if member.obj.is_this() {
            return self.resolve_this_member_lhs(member);
        }
        let prop_ident = match member.prop.as_ident() {
            Some(ident) => ident,
            None => {
                return AssignLhs::Invalid(
                    quote! { compile_error!("Tyrus: unsupported member assignment pattern") },
                );
            }
        };
        let prop_name = to_snake_case(prop_ident.sym.as_ref());
        let obj = self.convert_expr(&member.obj);
        if op == AssignOp::Assign && self.setter_names.contains(prop_ident.sym.as_ref()) {
            let setter = format_ident!("set_{}", prop_name);
            return AssignLhs::Setter { obj, setter };
        }
        let field = format_ident!("{}", prop_name);
        AssignLhs::Plain(quote! { #obj.#field })
    }

    fn resolve_this_member_lhs(&self, member: &MemberExpr) -> AssignLhs {
        let prop_ident = match member.prop.as_ident() {
            Some(ident) => ident,
            None => {
                return AssignLhs::Invalid(
                    quote! { compile_error!("Tyrus: unsupported self member assignment") },
                );
            }
        };
        let prop_name = to_snake_case(prop_ident.sym.as_ref());
        let field = format_ident!("{}", prop_name);
        let receiver = if self.use_state_for_this.get() {
            quote! { state }
        } else {
            quote! { self }
        };
        if self
            .current_class_state_fields
            .contains_key(prop_ident.sym.as_ref())
        {
            AssignLhs::StateField { receiver, field }
        } else {
            AssignLhs::Plain(quote! { #receiver.#field })
        }
    }

    fn emit_assign_op(&self, op: AssignOp, lhs: &TokenStream, rhs: &TokenStream) -> TokenStream {
        match op {
            AssignOp::Assign => quote! { #lhs = #rhs },
            AssignOp::AddAssign => quote! { #lhs += #rhs },
            AssignOp::SubAssign => quote! { #lhs -= #rhs },
            AssignOp::MulAssign => quote! { #lhs *= #rhs },
            AssignOp::DivAssign => quote! { #lhs /= #rhs },
            AssignOp::ModAssign => quote! { #lhs %= #rhs },
            // Bitwise ops: TS numbers are f64 but bitwise truncates to i32.
            // Rust f64 doesn't implement BitAnd/BitOr/etc, so cast through i64.
            AssignOp::BitAndAssign => {
                quote! { #lhs = ((#lhs as i64) & (#rhs as i64)) as f64 }
            }
            AssignOp::BitOrAssign => {
                quote! { #lhs = ((#lhs as i64) | (#rhs as i64)) as f64 }
            }
            AssignOp::BitXorAssign => {
                quote! { #lhs = ((#lhs as i64) ^ (#rhs as i64)) as f64 }
            }
            AssignOp::LShiftAssign => {
                quote! { #lhs = ((#lhs as i64) << (#rhs as i64)) as f64 }
            }
            AssignOp::RShiftAssign => {
                quote! { #lhs = ((#lhs as i64) >> (#rhs as i64)) as f64 }
            }
            _ => quote! { compile_error!("Tyrus: unsupported assignment operator") },
        }
    }

    pub(crate) fn convert_update_expr(&self, update: &UpdateExpr) -> TokenStream {
        let arg = self.convert_expr(&update.arg);
        match update.op {
            UpdateOp::PlusPlus => quote! { #arg += 1.0 },
            UpdateOp::MinusMinus => quote! { #arg -= 1.0 },
        }
    }

    pub(crate) fn convert_unary_expr(&self, expr: &UnaryExpr) -> TokenStream {
        let arg = self.convert_expr(&expr.arg);
        match expr.op {
            UnaryOp::Minus => quote! { -(#arg) },
            UnaryOp::Plus => arg,
            UnaryOp::Bang => quote! { !(#arg) },
            UnaryOp::TypeOf => {
                quote! { compile_error!("Tyrus: typeof not supported") }
            }
            _ => quote! { compile_error!("Tyrus: unsupported unary operator") },
        }
    }

    pub(crate) fn convert_opt_chain(&self, opt_chain: &swc_ecma_ast::OptChainExpr) -> TokenStream {
        match &*opt_chain.base {
            swc_ecma_ast::OptChainBase::Member(member) => {
                let obj = self.convert_expr(&member.obj);
                if let Some(prop_ident) = member.prop.as_ident() {
                    let prop_name = format_ident!("{}", to_snake_case(prop_ident.sym.as_ref()));
                    quote! { #obj.as_ref().and_then(|__v| Some(__v.#prop_name.clone())) }
                } else {
                    quote! { #obj }
                }
            }
            swc_ecma_ast::OptChainBase::Call(call) => {
                let callee = self.convert_expr(&call.callee);
                let args: Vec<_> = call
                    .args
                    .iter()
                    .map(|a| self.convert_expr(&a.expr))
                    .collect();
                quote! { #callee(#(#args),*) }
            }
        }
    }
}
