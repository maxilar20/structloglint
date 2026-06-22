use rustpython_parser::ast::{self, Suite};

use crate::ast_walker::{self, ParentContext};
use crate::config::Config;
use crate::models::Finding;
use crate::rules;

pub fn analyze<'a>(stmts: &'a Suite, config: &Config) -> Vec<Finding<'a>> {
    if config.check_imports && !has_structlog_import(stmts) {
        return vec![];
    }
    stmts
        .iter()
        .flat_map(|s| ast_walker::collect_log_calls(s, ParentContext::Module))
        .map(|log_call| {
            let results = rules::check_all(&log_call, config);
            Finding::new(log_call, results)
        })
        .collect()
}

fn has_structlog_import(stmts: &[ast::Stmt]) -> bool {
    stmts.iter().any(|stmt| match stmt {
        ast::Stmt::Import(import) => import
            .names
            .iter()
            .any(|alias| alias.name.as_str() == "structlog"),
        ast::Stmt::ImportFrom(import) => import
            .module
            .as_ref()
            .is_some_and(|m| m.as_str() == "structlog" || m.as_str().starts_with("structlog.")),
        _ => false,
    })
}
