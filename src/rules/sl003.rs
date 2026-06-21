use super::expr_helpers::is_substitution;
use crate::models::{LogCall, RuleResult, Status};

pub fn check_sl003(log_call: &LogCall) -> RuleResult {
    if let Some(event) = log_call.call.args.first()
        && is_substitution(event)
    {
        return RuleResult::new(
            "SL003",
            Status::Fail,
            "%-formatting used in event; use a constant string and pass data as keyword arguments"
                .to_string(),
        );
    }
    RuleResult::new("SL003", Status::Pass, String::new())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rules::test_helpers::check_first_call;

    #[test]
    fn passes_with_constant_string_event() {
        let result = check_first_call("log.info('user_signed_up')", check_sl003);
        assert_eq!(result.status, Status::Pass, "{}", result.feedback);
    }

    #[test]
    fn passes_with_variable_event() {
        let result = check_first_call("log.info(event)", check_sl003);
        assert_eq!(result.status, Status::Pass, "{}", result.feedback);
    }

    #[test]
    fn fails_with_single_substitution() {
        let result = check_first_call("log.info('user %s signed up' % username)", check_sl003);
        assert_eq!(result.status, Status::Fail, "{}", result.feedback);
    }

    #[test]
    fn fails_with_multiple_substitutions() {
        let result = check_first_call(
            "log.info('user %s on plan %s' % (username, plan))",
            check_sl003,
        );
        assert_eq!(result.status, Status::Fail, "{}", result.feedback);
    }

    #[test]
    fn fails_with_tuple_substitution() {
        let result = check_first_call(
            "log.info('user %s on plan %s' % (username, plan))",
            check_sl003,
        );
        assert_eq!(result.status, Status::Fail, "{}", result.feedback);
    }

    #[test]
    fn passes_with_substitution_not_in_event() {
        let result = check_first_call(
            "log.info('user_signed_up', detail='%s/%s' % ('alice', 'pro'))",
            check_sl003,
        );
        assert_eq!(result.status, Status::Pass, "{}", result.feedback);
    }
}
