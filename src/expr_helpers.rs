use rustpython_parser::ast;

/// Returns true if the expression is a constant literal (e.g. `'hello'`, `42`).
pub fn is_constant(expr: &ast::Expr) -> bool {
    matches!(expr, ast::Expr::Constant(_))
}

/// Returns true if the expression is a bare name reference (e.g. `event`, `var`).
pub fn is_name(expr: &ast::Expr) -> bool {
    matches!(expr, ast::Expr::Name(_))
}

/// Returns true if the expression is an f-string (e.g. `f'user {username}'`).
pub fn is_fstring(expr: &ast::Expr) -> bool {
    matches!(expr, ast::Expr::JoinedStr(_))
}

/// Returns true if the expression is `%`-style formatting (e.g. `'user %s' % username`).
pub fn is_substitution(expr: &ast::Expr) -> bool {
    let ast::Expr::BinOp(binop) = expr else {
        return false;
    };
    let ast::Expr::Constant(left) = binop.left.as_ref() else {
        return false;
    };
    matches!(binop.op, ast::Operator::Mod) && matches!(left.value, ast::Constant::Str(_))
}

/// Returns true if the expression is `.format()` called on a string literal
/// (e.g. `'hello {}'.format(name)`).
pub fn is_format(expr: &ast::Expr) -> bool {
    let ast::Expr::Call(call) = expr else {
        return false;
    };
    let ast::Expr::Attribute(attr) = call.func.as_ref() else {
        return false;
    };
    attr.attr.as_str() == "format"
}

/// Returns true if the expression is a call to `.exception()` on any object
/// (e.g. `log.exception('...')`).
pub fn is_exception(expr: &ast::Expr) -> bool {
    let ast::Expr::Call(call) = expr else {
        return false;
    };
    let ast::Expr::Attribute(attr) = call.func.as_ref() else {
        return false;
    };
    attr.attr.as_str() == "exception"
}

pub fn is_call_exception(call_expr: &ast::ExprCall) -> bool {
    matches!(call_expr.func.as_ref(), ast::Expr::Attribute(attr) if attr.attr.as_str() == "exception")
}
