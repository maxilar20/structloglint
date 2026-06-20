use colored::Colorize;
use rustpython_parser::ast;
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Status {
    Pass,
    Warning,
    Fail,
}

/// The parent context of a log call — what kind of block it sits directly inside.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParentContext {
    /// Not inside any block we track (top-level, etc.).
    Module,
    /// Inside an `except` handler.
    Except,
    /// Inside a `for` loop body.
    For,
    /// Inside a `for or else` loop body.
    ForElse,
    /// Inside a `while` loop body.
    While,
    /// Inside a `while else` block.
    WhileElse,
    /// Inside an `if` body.
    If,
    /// Inside an `else` block.
    Else,
    /// Inside a function body (but not a deeper tracked block).
    Function,
    /// Inside a `with` body.
    With,
    /// Inside a class body.
    Class,
}

/// A log call paired with its parent context.
#[derive(Clone, Copy)]
pub struct LogCall<'a> {
    pub call: &'a ast::ExprCall,
    pub context: ParentContext,
}

impl<'a> LogCall<'a> {
    pub fn new(call: &'a ast::ExprCall, context: ParentContext) -> Self {
        Self { call, context }
    }
}

pub struct RuleResult {
    pub rule_id: &'static str,
    pub status: Status,
    pub feedback: String,
}

impl RuleResult {
    pub fn new(rule_id: &'static str, status: Status, feedback: String) -> Self {
        Self {
            rule_id,
            status,
            feedback,
        }
    }
}

pub struct Finding<'a> {
    pub log_call: LogCall<'a>,
    pub results: Vec<RuleResult>,
}

impl<'a> Finding<'a> {
    pub fn new(log_call: LogCall<'a>, results: Vec<RuleResult>) -> Self {
        Self { log_call, results }
    }

    /// Convenience accessor for the underlying call expression.
    pub fn statement(&self) -> &ast::ExprCall {
        self.log_call.call
    }
}

impl fmt::Display for Finding<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for result in &self.results {
            let icon = match result.status {
                Status::Pass => "OK".green(),
                Status::Warning => "WARN".yellow(),
                Status::Fail => "FAIL".red(),
            };
            write!(f, "{icon} {}  {}", result.rule_id, result.feedback)?;
        }
        Ok(())
    }
}
