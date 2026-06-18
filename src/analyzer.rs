use rustpython_parser::{Parse, ast::Suite};

use crate::ast_walker;
use crate::models::Finding;
use crate::rules;

pub fn analyze(source: &str, file_name: &str) -> Result<Vec<Finding>, Box<dyn std::error::Error>> {
    let stmts = Suite::parse(source, file_name)?;

    let findings: Vec<Finding> = stmts
        .iter()
        .flat_map(ast_walker::collect_log_calls)
        .map(|call| Finding::new(call, rules::check_all(call)))
        .collect();

    Ok(findings)
}
