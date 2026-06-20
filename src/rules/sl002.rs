use rustpython_parser::ast;

use crate::expr_helpers::is_fstring;
use crate::models::{RuleResult, Status};

pub fn check_sl002(call: &ast::ExprCall) -> RuleResult {
    if let Some(first_arg) = call.args.first()
        && is_fstring(first_arg)
    {
        return RuleResult::new(
            "SL002",
            Status::Fail,
            "Passing an f-string as the event argument in structlog, consider using a constant string instead".to_string(),
        );
    }

    RuleResult::new("SL002", Status::Pass, String::new())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rules::test_helpers::check_first_call_expr;

    #[test]
    fn passes_with_plain_string_event() {
        let result = check_first_call_expr("log.info('user_logged_in')", check_sl002);
        assert_eq!(result.status, Status::Pass, "{}", result.feedback);
    }

    #[test]
    fn passes_with_keyword_args_only() {
        let result = check_first_call_expr(
            "log.info('user_logged_in', user_id='u_123', ip='1.2.3.4')",
            check_sl002,
        );
        assert_eq!(result.status, Status::Pass, "{}", result.feedback);
    }

    #[test]
    fn fails_with_f_string_but_no_interpolation() {
        let result = check_first_call_expr("log.info(f'user logged in')", check_sl002);
        assert_eq!(result.status, Status::Fail, "{}", result.feedback);
    }

    #[test]
    fn fails_with_f_string_with_interpolation() {
        let result = check_first_call_expr(
            r#"log.warning(f"rate limit exceeded for {'u_123'}")"#,
            check_sl002,
        );
        assert_eq!(result.status, Status::Fail, "{}", result.feedback)
    }

    #[test]
    fn fails_with_f_string_with_interpolation_log_error() {
        let result = check_first_call_expr(
            r#"log.error(f"login failed for u_123 from 1.2.3.4", exc_info=True)"#,
            check_sl002,
        );
        assert_eq!(result.status, Status::Fail, "{}", result.feedback)
    }

    #[test]
    fn passes_when_passing_variable_as_event_arg() {
        let result = check_first_call_expr("log.info(var)", check_sl002);
        assert_eq!(result.status, Status::Pass, "{}", result.feedback)
    }

    #[test]
    fn passes_when_constant_string_event_and_variable_kwarg() {
        let result = check_first_call_expr("log.info('user_logged_in', user_id=var)", check_sl002);
        assert_eq!(result.status, Status::Pass, "{}", result.feedback)
    }

    #[test]
    fn passes_when_constant_string_event_and_f_string_kwarg() {
        let result = check_first_call_expr(
            "log.info('user_logged_in', user_id=f'user_{var}')",
            check_sl002,
        );
        assert_eq!(result.status, Status::Pass, "{}", result.feedback)
    }
}
