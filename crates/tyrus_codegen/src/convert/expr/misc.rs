//! Miscellaneous expression code generation.
//!
//! Handles: assignment expressions, update expressions (++/--),
//! and optional chaining (?.).

use proc_macro2::{Ident, TokenStream};
use quote::{format_ident, quote};
use swc_ecma_ast::{
    AssignExpr, AssignOp, AssignTarget, Expr, MemberExpr, SimpleAssignTarget, UnaryExpr, UnaryOp,
    UpdateExpr, UpdateOp,
};

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
            AssignLhs::StateField { receiver, field } => {
                Self::emit_state_field_assign(&receiver, &field, assign.op, &right)
            }
            AssignLhs::Setter { obj, setter } => quote! { #obj.#setter(#right) },
            AssignLhs::Plain(lhs) => Self::emit_assign_op(assign.op, &lhs, &right),
            AssignLhs::Invalid(err) => err,
        }
    }

    /// Emits a Mutex-safe write to a `this.field` state field (#129).
    ///
    /// All ops route through a read-then-write split using a separate
    /// statement for each `.lock()` call. The `MutexGuard` from each lock
    /// is dropped at its statement's `;`, so the two locks never coexist —
    /// preventing `std::sync::Mutex` re-entrance deadlocks.
    fn emit_state_field_assign(
        receiver: &TokenStream,
        field: &Ident,
        op: AssignOp,
        rhs: &TokenStream,
    ) -> TokenStream {
        // Guarded read: block-scoped guard, returns Copy value.
        let guarded_read = quote! {
            { let __g = #receiver.#field.lock().unwrap_or_else(|e| e.into_inner()); *__g }
        };
        let new_val = match op {
            AssignOp::Assign => quote! { #rhs },
            AssignOp::AddAssign => quote! { #guarded_read + #rhs },
            AssignOp::SubAssign => quote! { #guarded_read - #rhs },
            AssignOp::MulAssign => quote! { #guarded_read * #rhs },
            AssignOp::DivAssign => quote! { #guarded_read / #rhs },
            AssignOp::ModAssign => quote! { #guarded_read % #rhs },
            AssignOp::BitAndAssign => {
                quote! { ((#guarded_read as i64) & (#rhs as i64)) as f64 }
            }
            AssignOp::BitOrAssign => {
                quote! { ((#guarded_read as i64) | (#rhs as i64)) as f64 }
            }
            AssignOp::BitXorAssign => {
                quote! { ((#guarded_read as i64) ^ (#rhs as i64)) as f64 }
            }
            AssignOp::LShiftAssign => {
                quote! { ((#guarded_read as i64) << (#rhs as i64)) as f64 }
            }
            AssignOp::RShiftAssign => {
                quote! { ((#guarded_read as i64) >> (#rhs as i64)) as f64 }
            }
            _ => {
                return quote! {
                    compile_error!("Tyrus: unsupported assignment operator on state field")
                }
            }
        };
        quote! {
            {
                let __new_val = #new_val;
                *#receiver.#field.lock().unwrap_or_else(|e| e.into_inner()) = __new_val;
            }
        }
    }

    /// Returns `(receiver, field_ident)` if `expr` is `this.<field>` and
    /// `<field>` is a tracked state field of the current class. Used by
    /// update-expr (`++`/`--`) to route through the same split-write path
    /// as compound assigns.
    fn try_resolve_this_state_field(&self, expr: &Expr) -> Option<(TokenStream, Ident)> {
        let Expr::Member(member) = expr else {
            return None;
        };
        if !member.obj.is_this() {
            return None;
        }
        let prop_ident = member.prop.as_ident()?;
        if !self
            .current_class_state_fields
            .contains_key(prop_ident.sym.as_ref())
        {
            return None;
        }
        let field = format_ident!("{}", to_snake_case(prop_ident.sym.as_ref()));
        let receiver = if self.use_state_for_this.get() {
            quote! { state }
        } else {
            quote! { self }
        };
        Some((receiver, field))
    }

    fn resolve_assign_lhs(&self, target: &AssignTarget, op: AssignOp) -> AssignLhs {
        let AssignTarget::Simple(simple) = target else {
            return AssignLhs::Invalid(
                quote! { compile_error!("Tyrus: unsupported assignment pattern") },
            );
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
        let Some(prop_ident) = member.prop.as_ident() else {
            return AssignLhs::Invalid(
                quote! { compile_error!("Tyrus: unsupported member assignment pattern") },
            );
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
        let Some(prop_ident) = member.prop.as_ident() else {
            return AssignLhs::Invalid(
                quote! { compile_error!("Tyrus: unsupported self member assignment") },
            );
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

    fn emit_assign_op(op: AssignOp, lhs: &TokenStream, rhs: &TokenStream) -> TokenStream {
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
        if let Some((receiver, field)) = self.try_resolve_this_state_field(&update.arg) {
            let op = match update.op {
                UpdateOp::PlusPlus => AssignOp::AddAssign,
                UpdateOp::MinusMinus => AssignOp::SubAssign,
            };
            return Self::emit_state_field_assign(&receiver, &field, op, &quote! { 1.0 });
        }

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
