use rustpython_parser::ast;

use crate::models::{LogCall, RuleResult, Status};

/// SL009 — Event string exceeds maximum length.
pub fn check_sl009(log_call: &LogCall, event_length: usize) -> RuleResult {
    let Some(first_arg) = log_call.call.args.first() else {
        return RuleResult::new("SL009", Status::Pass, String::new());
    };

    let ast::Expr::Constant(constant) = first_arg else {
        return RuleResult::new("SL009", Status::Pass, String::new());
    };

    let ast::Constant::Str(event) = &constant.value else {
        return RuleResult::new("SL009", Status::Pass, String::new());
    };

    if event.len() <= event_length {
        return RuleResult::new("SL009", Status::Pass, String::new());
    }

    RuleResult::new(
        "SL009",
        Status::Fail,
        format!(
            "event string \"{}\" exceeds maximum length of {}",
            event, event_length,
        ),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rules::test_helpers::check_first_call;

    fn checker(event_length: usize) -> impl Fn(&LogCall) -> RuleResult {
        move |log_call| check_sl009(log_call, event_length)
    }

    #[test]
    fn passes_snake_case() {
        let result = check_first_call(
            r#"log.info("profile_updated", user_id="u_123")"#,
            checker(20),
        );
        assert_eq!(result.status, Status::Pass, "{}", result.feedback);
    }

    #[test]
    fn fails_event_string_too_long() {
        let result = check_first_call(
            r#"log.info("profile_updated_with_long_event_string", user_id="u_123")"#,
            checker(20),
        );
        assert_eq!(result.status, Status::Fail, "{}", result.feedback);
    }
}
