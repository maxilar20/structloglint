use colored::Colorize;
use rustpython_parser::ast;
use std::fmt;
use std::str::FromStr;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Status {
    Pass,
    Warning,
    Fail,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum LogLevel {
    Trace = 0,
    Debug = 1,
    Info = 2,
    Warning = 3,
    Error = 4,
    Critical = 5,
    Exception = 6,
}

impl LogLevel {
    pub const ALL: &[LogLevel] = &[
        Self::Trace,
        Self::Debug,
        Self::Info,
        Self::Warning,
        Self::Error,
        Self::Critical,
        Self::Exception,
    ];

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Trace => "trace",
            Self::Debug => "debug",
            Self::Info => "info",
            Self::Warning => "warning",
            Self::Error => "error",
            Self::Critical => "critical",
            Self::Exception => "exception",
        }
    }
}

impl FromStr for LogLevel {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::ALL.iter().find(|l| l.as_str() == s).copied().ok_or(())
    }
}

impl fmt::Display for LogLevel {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The parent context of a log call — what kind of block it sits directly inside.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParentContext {
    /// Not inside any block we track (top-level, etc.).
    Module,
    /// Inside an `except` handler (`try … except` or `try … except*`).
    Except,
    /// Inside a `for` loop body.
    For,
    /// Inside the `else` clause of a `for` loop.
    ForElse,
    /// Inside a `while` loop body.
    While,
    /// Inside the `else` clause of a `while` loop.
    WhileElse,
    /// Inside an `if` body (includes `elif` bodies).
    If,
    /// Inside the `else` clause of an `if`/`elif`.
    Else,
    /// Inside a function body.
    Function,
    /// Inside an async function body.
    AsyncFunction,
    /// Inside a `with` body.
    With,
    /// Inside an `async with` body.
    AsyncWith,
    /// Inside a class body.
    Class,
    /// Inside the body of a `try` block (before `except`).
    Try,
    /// Inside the `else` clause of a `try` block.
    TryElse,
    /// Inside a `finally` block.
    Finally,
    /// Inside an `async for` loop body.
    AsyncFor,
    /// Inside the `else` clause of an `async for` loop.
    AsyncForElse,
    /// Inside a `match` case body.
    Match,
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
