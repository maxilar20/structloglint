use crate::models::{LogCall, LogLevel, RuleResult};

mod sl001;
mod sl002;
mod sl003;
mod sl004;
mod sl005;
mod sl006;
mod sl007;

pub fn check_all(log_call: &LogCall) -> Vec<RuleResult> {
    vec![
        sl001::check_sl001(log_call.call),
        sl002::check_sl002(log_call.call),
        sl003::check_sl003(log_call.call),
        sl004::check_sl004(log_call.call),
        sl005::check_sl005(log_call),
        sl006::check_sl006(log_call),
        sl007::check_sl007(log_call, LogLevel::Info),
    ]
}

#[cfg(test)]
mod test_helpers {
    use crate::ast_walker::collect_log_calls;
    use crate::models::{LogCall, ParentContext, RuleResult};
    use rustpython_parser::Parse;
    use rustpython_parser::ast::Suite;

    /// Parse the source and run `check_fn` on the first log call found.
    /// Returns the RuleResult so the test can assert on it.
    pub fn check_first_call<F>(source: &str, check_fn: F) -> RuleResult
    where
        F: Fn(&LogCall) -> RuleResult,
    {
        let stmts = Suite::parse(source, "<test>").expect("parse failed");
        let calls: Vec<LogCall> = stmts
            .iter()
            .flat_map(|s| collect_log_calls(s, ParentContext::Module))
            .collect();
        let call = calls.first().expect("no log call found");
        check_fn(call)
    }

    /// Parse the source and run `check_fn` on the first log call found.
    /// Uses the old-style signature that takes `&ExprCall` directly
    /// (for rules that don't need parent context).
    pub fn check_first_call_expr(
        source: &str,
        check_fn: fn(&rustpython_parser::ast::ExprCall) -> RuleResult,
    ) -> RuleResult {
        let stmts = Suite::parse(source, "<test>").expect("parse failed");
        let calls: Vec<LogCall> = stmts
            .iter()
            .flat_map(|s| collect_log_calls(s, ParentContext::Module))
            .collect();
        let call = calls.first().expect("no log call found");
        check_fn(call.call)
    }
}
