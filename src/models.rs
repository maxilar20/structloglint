use colored::Colorize;
use rustpython_parser::ast;
use std::fmt;

pub struct RuleResult {
    pub rule_id: &'static str,
    pub pass: bool,
    pub feedback: String,
}

impl RuleResult {
    pub fn new(rule_id: &'static str, pass: bool, feedback: String) -> Self {
        Self {
            rule_id,
            pass,
            feedback,
        }
    }
}

pub struct Finding {
    pub statement: ast::ExprCall,
    pub results: Vec<RuleResult>,
}

impl Finding {
    pub fn new(statement: &ast::ExprCall, results: Vec<RuleResult>) -> Self {
        Self {
            statement: statement.clone(),
            results,
        }
    }
}

impl fmt::Display for Finding {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for result in &self.results {
            let icon = if result.pass {
                "OK".green()
            } else {
                "FAIL".red()
            };
            write!(f, "{icon} {}  {}", result.rule_id, result.feedback)?;
        }
        Ok(())
    }
}
