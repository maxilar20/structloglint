use super::expr_helpers::is_format;
use crate::models::{LogCall, RuleResult, Status};

pub fn check_sl004(log_call: &LogCall) -> RuleResult {
    if let Some(event) = log_call.call.args.first()
        && is_format(event)
    {
        return RuleResult::new(
            "SL004",
            Status::Fail,
            ".format() used on event; use a constant string and pass data as keyword arguments"
                .to_string(),
        );
    }
    RuleResult::new("SL004", Status::Pass, String::new())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rules::test_helpers::check_first_call;

    #[test]
    fn passes_with_keyword_arguments() {
        let result = check_first_call(
            "log.info('subscription_cancelled', user_id='u_123', reason='too_expensive')",
            check_sl004,
        );
        assert_eq!(result.status, Status::Pass, "{}", result.feedback);
    }

    #[test]
    fn fails_with_positional_placeholders() {
        let result = check_first_call(
            "log.info('subscription cancelled for {}'.format('u_123'))",
            check_sl004,
        );
        assert_eq!(result.status, Status::Fail, "{}", result.feedback);
    }

    #[test]
    fn fails_with_named_placeholders() {
        let result = check_first_call(
            "log.warning('user {user_id} cancelled: {reason}'.format(user_id='u_123', reason='too_expensive'))",
            check_sl004,
        );
        assert_eq!(result.status, Status::Fail, "{}", result.feedback);
    }

    #[test]
    fn passes_with_positional_placeholders_in_value() {
        let result = check_first_call(
            "log.info('subscription_cancelled', summary='u={} r={}'.format('u_123', 'expensive'))",
            check_sl004,
        );
        assert_eq!(result.status, Status::Pass, "{}", result.feedback);
    }

    #[test]
    fn fails_with_format_on_variable() {
        let result = check_first_call("log.info(var.format(user_id))", check_sl004);
        assert_eq!(result.status, Status::Fail, "{}", result.feedback);
    }
}
