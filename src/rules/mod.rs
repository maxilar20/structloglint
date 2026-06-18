use rustpython_parser::ast;

use crate::models::RuleResult;

mod sl001;

pub fn check_all(call: &ast::ExprCall) -> Vec<RuleResult> {
    vec![sl001::check_sl001(call)]
}
