use rustpython_parser::ast;

use crate::models::RuleResult;

mod sl001;
mod sl002;

pub fn check_all(call: &ast::ExprCall) -> Vec<RuleResult> {
    vec![sl001::check_sl001(call), sl002::check_sl002(call)]
}
