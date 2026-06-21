use rustpython_parser::ast;

use crate::ast_walker::ParentContext;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Status {
    Pass,
    Warning,
    Fail,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, strum::EnumString, strum::Display)]
#[strum(serialize_all = "snake_case")]
pub enum RuleSeverity {
    Error,
    Warning,
    Off,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, strum::EnumString, strum::Display)]
#[strum(serialize_all = "snake_case")]
pub enum LogLevel {
    Trace = 0,
    Debug = 1,
    Info = 2,
    Warning = 3,
    Error = 4,
    Critical = 5,
    Exception = 6,
}

/// A log call paired with its parent context and resolved log level.
#[derive(Clone, Copy)]
pub struct LogCall<'a> {
    pub call: &'a ast::ExprCall,
    pub context: ParentContext,
    pub level: LogLevel,
}

impl<'a> LogCall<'a> {
    pub fn new(call: &'a ast::ExprCall, context: ParentContext, level: LogLevel) -> Self {
        Self {
            call,
            context,
            level,
        }
    }
}

pub struct Fix {
    pub replacement: String,
    pub start: usize,
    pub end: usize,
}

pub struct RuleResult {
    pub rule_id: &'static str,
    pub status: Status,
    pub feedback: String,
    pub fix: Option<Fix>,
}

impl RuleResult {
    pub fn new(rule_id: &'static str, status: Status, feedback: String) -> Self {
        Self {
            rule_id,
            status,
            feedback,
            fix: None,
        }
    }

    pub fn with_fix(mut self, fix: Fix) -> Self {
        self.fix = Some(fix);
        self
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
