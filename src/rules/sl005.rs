use crate::models::{LogCall, ParentContext, RuleResult, Status};

use rustpython_parser::ast;

/// SL005: `log.exception()` must only be used inside an `except` block.
pub fn check_sl005(log_call: &LogCall) -> RuleResult {
    // Only `exception()` calls are subject to this check.
    let is_exception = match log_call.call.func.as_ref() {
        ast::Expr::Attribute(attr) => attr.attr.as_str() == "exception",
        _ => false,
    };

    if !is_exception {
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
}

// # OK — exception() inside except
// def send_email(user_id: str):
//     try:
//         pass
//     except Exception:
//         log.exception("email_send_failed", user_id=user_id)
//         raise

// # OK — exception() inside nested except
// def nested(user_id: str):
//     try:
//         try:
//             pass
//         except ValueError:
//             log.exception("inner_failed", user_id=user_id)
//     except Exception:
//         log.exception("outer_failed", user_id=user_id)

// # SL005 — outside any except block
// def notify_user(user_id: str):
//     log.exception("notification_failed", user_id=user_id)

// # OK — error() outside except is fine
// def archive(user_id: str):
//     log.error("archive_failed", user_id=user_id)
