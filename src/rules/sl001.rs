use rustpython_parser::ast;

use crate::models::{RuleResult, Status};

pub fn check_sl001(call: &ast::ExprCall) -> RuleResult {
    if call.args.len() > 1 {
        return RuleResult::new(
            "SL001",
            Status::Fail,
            "Too many positional arguments. Only one positional argument should be provided."
                .to_string(),
        );
    }

    RuleResult::new("SL001", Status::Pass, String::new())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast_walker::collect_log_calls;
    use rustpython_parser::Parse;
    use rustpython_parser::ast::Suite;

    // Parse the source and run check_sl001 on the first log call found.
    // Returns the RuleResult so the test can assert on it.
    fn check_first_call(source: &str) -> RuleResult {
        let stmts = Suite::parse(source, "<test>").expect("parse failed");
        let calls: Vec<&ast::ExprCall> = stmts.iter().flat_map(collect_log_calls).collect();
        let call = calls.first().expect("no log call found");
        check_sl001(call)
    }

    #[test]
    fn passes_with_keyword_args_only() {
        let result = check_first_call("log.info('user_logged_in', user_id='u_123')");
        assert_eq!(result.status, Status::Pass, "{}", result.feedback);
    }

    #[test]
    fn passes_with_no_args() {
        let result = check_first_call("log.info('payment_complete')");
        assert_eq!(result.status, Status::Pass, "{}", result.feedback);
    }

    #[test]
    fn passes_with_multiple_keyword_args() {
        let result =
            check_first_call("log.info('payment_complete', payment_id='pay_1', duration_ms=42)");
        assert_eq!(result.status, Status::Pass, "{}", result.feedback);
    }

    #[test]
    fn fails_with_one_positional_arg() {
        let result = check_first_call("log.info('user_logged_in', user_id)");
        assert_eq!(result.status, Status::Fail, "expected SL001 to fire");
    }

    #[test]
    fn fails_with_multiple_positional_args() {
        let result = check_first_call("log.info('payment_processed', user_id, order_id, 4999)");
        assert_eq!(result.status, Status::Fail, "expected SL001 to fire");
    }

    #[test]
    fn fails_with_mixed_positional_and_keyword() {
        let result = check_first_call("log.warning('rate_limit_exceeded', user_id, limit=100)");
        assert_eq!(result.status, Status::Fail, "expected SL001 to fire");
    }
}
