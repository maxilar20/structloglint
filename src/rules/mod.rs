use crate::models::{LogCall, LogLevel, RuleResult};

use self::case_style::CaseStyle;

mod case_style;
mod expr_helpers;
mod sl001;
mod sl002;
mod sl003;
mod sl004;
mod sl005;
mod sl006;
mod sl007;
mod sl008;
mod sl009;

pub fn check_all(log_call: &LogCall) -> Vec<RuleResult> {
    vec![
        sl001::check_sl001(log_call.call),
        sl002::check_sl002(log_call.call),
        sl003::check_sl003(log_call.call),
        sl004::check_sl004(log_call.call),
        sl005::check_sl005(log_call),
        sl006::check_sl006(log_call),
        sl007::check_sl007(log_call, LogLevel::Info),
        sl008::check_sl008(log_call, CaseStyle::SnakeCase),
        sl009::check_sl009(log_call, 30),
    ]
}

#[cfg(test)]
mod test_helpers {
    use crate::ast_walker::{ParentContext, collect_log_calls};
    use crate::models::{LogCall, RuleResult};
    use rustpython_parser::Parse;
    use rustpython_parser::ast::Suite;

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
