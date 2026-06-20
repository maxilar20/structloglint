use rustpython_parser::ast::Suite;

use crate::ast_walker;
use crate::models::{Finding, ParentContext};
use crate::rules;

pub fn analyze<'a>(stmts: &'a Suite) -> Vec<Finding<'a>> {
    stmts
        .iter()
        .flat_map(|s| ast_walker::collect_log_calls(s, ParentContext::Module))
        .map(|log_call| {
            let results = rules::check_all(&log_call);
            Finding::new(log_call, results)
        })
        .collect()
}
