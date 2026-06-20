use rustpython_parser::ast;

use crate::models::{RuleResult, Status};

pub fn check_sl001(call: &ast::ExprCall) -> RuleResult {
    if call.args.len() > 1 {
        return RuleResult::new(
            "SL001",
            Status::Fail,
            "too many positional arguments; expected at most one (the event string)".to_string(),
        );
    }

    RuleResult::new("SL001", Status::Pass, String::new())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rules::test_helpers::check_first_call_expr;

    #[test]
    fn passes_with_keyword_args_only() {
        let result =
            check_first_call_expr("log.info('user_logged_in', user_id='u_123')", check_sl001);
        assert_eq!(result.status, Status::Pass, "{}", result.feedback);
    }

    #[test]
    fn passes_with_no_args() {
        let result = check_first_call_expr("log.info('payment_complete')", check_sl001);
        assert_eq!(result.status, Status::Pass, "{}", result.feedback);
    }

    #[test]
    fn passes_with_multiple_keyword_args() {
        let result = check_first_call_expr(
            "log.info('payment_complete', payment_id='pay_1', duration_ms=42)",
            check_sl001,
        );
        assert_eq!(result.status, Status::Pass, "{}", result.feedback);
    }

    #[test]
    fn fails_with_one_positional_arg() {
        let result = check_first_call_expr("log.info('user_logged_in', user_id)", check_sl001);
        assert_eq!(result.status, Status::Fail, "expected SL001 to fire");
    }

    #[test]
    fn fails_with_multiple_positional_args() {
        let result = check_first_call_expr(
            "log.info('payment_processed', user_id, order_id, 4999)",
            check_sl001,
        );
        assert_eq!(result.status, Status::Fail, "expected SL001 to fire");
    }

    #[test]
    fn fails_with_mixed_positional_and_keyword() {
        let result = check_first_call_expr(
            "log.warning('rate_limit_exceeded', user_id, limit=100)",
            check_sl001,
        );
        assert_eq!(result.status, Status::Fail, "expected SL001 to fire");
    }
}
