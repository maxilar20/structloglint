use rustpython_parser::ast;

use crate::models::RuleResult;

mod sl001;
mod sl002;
mod sl003;

pub fn check_all(call: &ast::ExprCall) -> Vec<RuleResult> {
    vec![
        sl001::check_sl001(call),
        sl002::check_sl002(call),
        sl003::check_sl003(call),
    ]
}

#[cfg(test)]
mod test_helpers {
    use crate::ast_walker::collect_log_calls;
    use crate::models::RuleResult;
    use rustpython_parser::Parse;
    use rustpython_parser::ast::{self, Suite};

    /// Parse the source and run `check_fn` on the first log call found.
    /// Returns the RuleResult so the test can assert on it.
    pub fn check_first_call(
        source: &str,
        check_fn: fn(&ast::ExprCall) -> RuleResult,
    ) -> RuleResult {
        let stmts = Suite::parse(source, "<test>").expect("parse failed");
        let calls: Vec<&ast::ExprCall> = stmts.iter().flat_map(collect_log_calls).collect();
        let call = calls.first().expect("no log call found");
        check_fn(call)
    }
}
