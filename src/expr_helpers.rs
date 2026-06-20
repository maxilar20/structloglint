use rustpython_parser::ast;

/// Returns true if the expression is a constant string literal (e.g. `'hello'`, `"world"`).
pub fn is_constant(expr: &ast::Expr) -> bool {
    matches!(expr, ast::Expr::Constant(_))
}

/// Returns true if the expression is a bare name reference (e.g. `event`, `var`).
pub fn is_name(expr: &ast::Expr) -> bool {
    matches!(expr, ast::Expr::Name(_))
}

/// Returns true if the expression is a `%`-based string substitution
/// (e.g. `'user %s' % username` or `'a %s b %s' % (x, y)`).
pub fn is_substitution(expr: &ast::Expr) -> bool {
    if let ast::Expr::BinOp(binop) = expr {
        matches!(binop.op, ast::Operator::Mod)
            && matches!(
                binop.left.as_ref(),
                ast::Expr::Constant(c) if matches!(c.value, ast::Constant::Str(_))
            )
    } else {
        false
    }
}

/// Returns true if the expression is a `f`-string literal (e.g. `f'user {username}'`).
pub fn is_fstring(expr: &ast::Expr) -> bool {
    matches!(expr, ast::Expr::JoinedStr(_))
}

pub fn is_format(expr: &ast::Expr) -> bool {
    if let ast::Expr::Call(call) = expr {
        if let ast::Expr::Attribute(attr) = call.func.as_ref() {
            matches!(attr.value.as_ref(), ast::Expr::Constant(c)
                if matches!(c.value, ast::Constant::Str(_)))
                && attr.attr.as_str() == "format"
        } else {
            false
        }
    } else {
        false
    }
}
