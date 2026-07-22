use rustpython_parser::ast::{self, Suite};

use crate::ast_walker::{self, ParentContext};
use crate::config::Config;
use crate::display::LineIndex;
use crate::models::{Finding, Status};
use crate::noqa;
use crate::rules;

pub fn analyze<'a>(stmts: &'a Suite, config: &Config, source: &str) -> Vec<Finding<'a>> {
    if config.check_imports && !has_structlog_import(stmts) {
        return vec![];
    }
    let noqa_map = noqa::parse_noqa_comments(source);
    let line_index = LineIndex::new(source);
    stmts
        .iter()
        .flat_map(|s| ast_walker::collect_log_calls(s, ParentContext::Module))
        .map(|log_call| {
            let mut results = rules::check_all(&log_call, config);
            let start_offset = u32::from(log_call.call.range.start()) as usize;
            let (line, _) = line_index.line_col(start_offset);
            if let Some(directive) = noqa_map.get(&line) {
                results.retain(|r| r.status == Status::Pass || !directive.suppresses(r.rule_id));
            }
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
