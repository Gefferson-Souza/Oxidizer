use quote::{format_ident, quote};
use swc_ecma_ast::{
    AwaitExpr, BinExpr, BinaryOp, CallExpr, Callee, Expr, ExprOrSpread, FnDecl, Lit, MemberExpr,
    Pat, ReturnStmt, Stmt,
};

use super::type_mapper::{map_ts_type, unwrap_promise_type};

impl super::interface::RustGenerator {
    pub fn visit_fn_decl(&mut self, n: &FnDecl) {
        let fn_name = to_snake_case(&n.ident.sym);
        let fn_ident = format_ident!("{}", fn_name);

        // Check if async
        let is_async = n.function.is_async;

        // Extract parameters
        let mut params = Vec::new();
        for param in &n.function.params {
            if let Pat::Ident(ident_pat) = &param.pat {
                let param_name = format_ident!("{}", ident_pat.sym.to_string());
                let param_type = map_ts_type(ident_pat.type_ann.as_ref());
                params.push(quote! { #param_name: #param_type });
            }
        }

        // Extract return type - unwrap Promise<T> for async functions
        let return_type = if is_async {
            unwrap_promise_type(n.function.return_type.as_ref())
        } else {
            map_ts_type(n.function.return_type.as_ref())
        };

        // Convert body
        let mut body_stmts = Vec::new();
        if let Some(block_stmt) = &n.function.body {
            for stmt in &block_stmt.stmts {
                body_stmts.push(convert_stmt(stmt));
            }
        }

        let fn_def = if is_async {
            quote! {
                pub async fn #fn_ident(#(#params),*) -> #return_type {
                    #(#body_stmts)*
                }
            }
        } else {
            quote! {
                pub fn #fn_ident(#(#params),*) -> #return_type {
                    #(#body_stmts)*
                }
            }
        };

        self.code.push_str(&fn_def.to_string());
        self.code.push('\n');
    }
}

pub fn convert_stmt_pub(stmt: &Stmt) -> proc_macro2::TokenStream {
    convert_stmt(stmt)
}

pub fn convert_expr_pub(expr: &Expr) -> proc_macro2::TokenStream {
    convert_expr(expr)
}

fn convert_stmt(stmt: &Stmt) -> proc_macro2::TokenStream {
    match stmt {
        Stmt::Return(ret) => convert_return_stmt(ret),
        Stmt::Expr(expr_stmt) => {
            let expr = convert_expr(&expr_stmt.expr);
            quote! { #expr; }
        }
        _ => quote! {}, // Skip other statements for now
    }
}

fn convert_return_stmt(ret: &ReturnStmt) -> proc_macro2::TokenStream {
    if let Some(arg) = &ret.arg {
        let expr = convert_expr(arg);
        quote! { return #expr; }
    } else {
        quote! { return; }
    }
}

fn convert_expr(expr: &Expr) -> proc_macro2::TokenStream {
    match expr {
        Expr::Bin(bin) => convert_bin_expr(bin),
        Expr::Ident(ident) => {
            // Convert to snake_case for consistency
            let ident_name = to_snake_case(&ident.sym);
            let ident_token = format_ident!("{}", ident_name);
            quote! { #ident_token }
        }
        Expr::Lit(lit) => {
            // Handle literals
            match lit {
                Lit::Num(num) => {
                    let value = num.value;
                    quote! { #value }
                }
                Lit::Str(str_lit) => {
                    // Use proc_macro2::Literal to create a string literal token
                    use proc_macro2::Literal;
                    let string_val = Literal::string(str_lit.value.as_str().unwrap_or(""));
                    quote! { #string_val }
                }
                _ => quote! { todo!("unsupported literal") },
            }
        }
        Expr::Member(member) => convert_member_expr(member),
        Expr::Await(await_expr) => convert_await_expr(await_expr),
        Expr::Call(call_expr) => convert_call_expr(call_expr),
        _ => quote! { todo!() },
    }
}

fn convert_member_expr(member: &MemberExpr) -> proc_macro2::TokenStream {
    // Handle this.prop -> self.prop
    if member.obj.is_this() {
        if let Some(prop_ident) = member.prop.as_ident() {
            let field = format_ident!("{}", prop_ident.sym.to_string());
            return quote! { self.#field };
        }
    }
    // Handle other.prop
    let obj = convert_expr(&member.obj);
    if let Some(prop_ident) = member.prop.as_ident() {
        let prop = format_ident!("{}", prop_ident.sym.to_string());
        quote! { #obj.#prop }
    } else {
        quote! { todo!("complex member access") }
    }
}

fn convert_bin_expr(bin: &BinExpr) -> proc_macro2::TokenStream {
    let left = convert_expr(&bin.left);
    let right = convert_expr(&bin.right);

    let op = match bin.op {
        BinaryOp::Add => quote! { + },
        BinaryOp::Sub => quote! { - },
        BinaryOp::Mul => quote! { * },
        BinaryOp::Div => quote! { / },
        _ => quote! { /* unsupported op */ },
    };

    quote! { #left #op #right }
}

fn convert_await_expr(await_expr: &AwaitExpr) -> proc_macro2::TokenStream {
    let inner = convert_expr(&await_expr.arg);
    quote! { #inner.await }
}

fn convert_call_expr(call: &CallExpr) -> proc_macro2::TokenStream {
    // Check for axios calls (axios.get, axios.post, etc.)
    if let Callee::Expr(expr) = &call.callee {
        if let Expr::Member(member) = &**expr {
            // Check if object is "axios"
            if let Expr::Ident(obj_ident) = &*member.obj {
                if obj_ident.sym == "axios" {
                    // Get the HTTP method
                    if let Some(method_ident) = member.prop.as_ident() {
                        let method = method_ident.sym.to_string();
                        return convert_axios_call(&method, &call.args);
                    }
                }
            }
        }

        // Check for fetch calls
        if let Expr::Ident(ident) = &**expr {
            if ident.sym == "fetch" {
                return convert_fetch_call(&call.args);
            }
        }
    }

    // Fallback to generic call conversion
    let callee = match &call.callee {
        Callee::Expr(expr) => convert_expr(expr),
        _ => quote! { unknown_callee },
    };

    let args: Vec<_> = call.args.iter().map(convert_expr_or_spread).collect();

    quote! { #callee(#(#args),*) }
}

fn convert_axios_call(method: &str, args: &[ExprOrSpread]) -> proc_macro2::TokenStream {
    let method_lower = method.to_lowercase();
    let method_ident = format_ident!("{}", method_lower);

    if args.is_empty() {
        return quote! { reqwest::Client::new().#method_ident("").send().await? };
    }

    // First argument is the URL
    let url = convert_expr_or_spread(&args[0]);

    // For POST/PUT, second argument might be data
    if (method_lower == "post" || method_lower == "put") && args.len() > 1 {
        let data = convert_expr_or_spread(&args[1]);
        quote! {
            reqwest::Client::new()
                .#method_ident(#url)
                .json(&#data)
                .send()
                .await?
        }
    } else {
        // GET/DELETE or POST/PUT without body
        quote! {
            reqwest::Client::new()
                .#method_ident(#url)
                .send()
                .await?
        }
    }
}

fn convert_fetch_call(args: &[ExprOrSpread]) -> proc_macro2::TokenStream {
    if args.is_empty() {
        return quote! { reqwest::get("").await? };
    }

    let url = convert_expr_or_spread(&args[0]);
    quote! { reqwest::get(#url).await? }
}

fn convert_expr_or_spread(arg: &ExprOrSpread) -> proc_macro2::TokenStream {
    convert_expr(&arg.expr)
}

fn to_snake_case(s: &str) -> String {
    let mut result = String::new();
    for (i, ch) in s.chars().enumerate() {
        if ch.is_uppercase() && i > 0 {
            result.push('_');
        }
        result.push(ch.to_lowercase().next().unwrap_or(ch));
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_to_snake_case_simple() {
        assert_eq!(to_snake_case("fetchData"), "fetch_data");
        assert_eq!(to_snake_case("getUserName"), "get_user_name");
        assert_eq!(to_snake_case("HTTPRequest"), "h_t_t_p_request");
    }

    #[test]
    fn test_to_snake_case_already_snake() {
        assert_eq!(to_snake_case("already_snake"), "already_snake");
    }

    #[test]
    fn test_to_snake_case_single_word() {
        assert_eq!(to_snake_case("simple"), "simple");
        assert_eq!(to_snake_case("Simple"), "simple");
    }

    #[test]
    fn test_to_snake_case_empty() {
        assert_eq!(to_snake_case(""), "");
    }
}
