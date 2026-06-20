use crate::expr_helpers::is_call_exception;
use crate::models::{LogCall, ParentContext, RuleResult, Status};

/// SL005: `log.exception()` must only be used inside an `except` block.
pub fn check_sl005(log_call: &LogCall) -> RuleResult {
    // Only `exception()` calls are subject to this check.

    if !is_call_exception(&log_call.call) {
        return RuleResult::new("SL005", Status::Pass, String::new());
    }

    // `exception()` must be inside an Except context.
    if log_call.context != ParentContext::Except {
        return RuleResult::new(
            "SL005",
            Status::Fail,
            "log.exception() used outside of an except block. Use log.exception() only inside except handlers."
                .to_string(),
        );
    }

    RuleResult::new("SL005", Status::Pass, String::new())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rules::test_helpers::check_first_call;

    #[test]
    fn passes_inside_except_block() {
        let source = r#"import structlog
log = structlog.get_logger()
try:
    pass
except Exception:
    log.exception("boom", user_id="u_123")
"#;
        let result = check_first_call(source, check_sl005);
        assert_eq!(result.status, Status::Pass, "{}", result.feedback);
    }

    #[test]
    fn fails_outside_except_block() {
        let source = r#"import structlog
log = structlog.get_logger()
log.exception("boom", user_id="u_123")
"#;
        let result = check_first_call(source, check_sl005);
        assert_eq!(result.status, Status::Fail, "{}", result.feedback);
    }

    #[test]
    fn passes_non_exception_call_outside_except() {
        let source = "log.error('fail', user_id='u_123')";
        let result = check_first_call(source, check_sl005);
        assert_eq!(result.status, Status::Pass, "{}", result.feedback);
    }

    #[test]
    fn passes_exception_call_inside_except() {
        let source = r#"def send_email(user_id: str):
    try:
        pass
    except Exception:
        log.exception("email_send_failed", user_id=user_id)
        raise"#;
        let result = check_first_call(source, check_sl005);
        assert_eq!(result.status, Status::Pass, "{}", result.feedback);
    }

    #[test]
    fn passes_exception_call_inside_nested_except() {
        let source = r#"def nested(user_id: str):
    try:
        try:
            pass
        except ValueError:
            log.exception("inner_failed", user_id=user_id)
    except Exception:
        log.exception("outer_failed", user_id=user_id)"#;
        let result = check_first_call(source, check_sl005);
        assert_eq!(result.status, Status::Pass, "{}", result.feedback);
    }
}
