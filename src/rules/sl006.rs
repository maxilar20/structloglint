use rustpython_parser::ast;
use rustpython_parser::ast::Keyword;

use crate::models::{LogCall, LogLevel, ParentContext, RuleResult, Status};

fn keyword_is_exc_info(keyword: &Keyword) -> bool {
    if let Some(keyword_id) = &keyword.arg {
        keyword_id.as_str() == "exc_info"
    } else {
        false
    }
}

fn any_keyword_is_exc_info(log_call: &ast::ExprCall) -> bool {
    log_call
        .keywords
        .iter()
        .any(|keyword| keyword_is_exc_info(keyword))
}

/// SL006 — error() inside except block without exc_info=True.
pub fn check_sl006(log_call: &LogCall) -> RuleResult {
    if log_call.level != LogLevel::Error {
        return RuleResult::new("SL006", Status::Pass, String::new());
    }

    if log_call.context == ParentContext::Except {
        if any_keyword_is_exc_info(log_call.call) {
            return RuleResult::new("SL006", Status::Pass, String::new());
        }
        return RuleResult::new("SL006", Status::Fail, String::new());
    }

    RuleResult::new("SL006", Status::Pass, String::new())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rules::test_helpers::check_first_call;

    #[test]
    fn passes_with_explicit_exc_info() {
        let log_call = check_first_call(
            r#"def refund_order(order_id: str):
    try:
        pass
    except ValueError:
        log.error("refund_validation_failed", order_id=order_id, exc_info=True)
        raise"#,
            check_sl006,
        );
        assert_eq!(log_call.status, Status::Pass);
    }

    #[test]
    fn fails_with_no_explicit_exc_info() {
        let log_call = check_first_call(
            r#"def charge_card(order_id: str):
    try:
        pass
    except TimeoutError as e:
        log.error("card_charge_timed_out", order_id=order_id, error=str(e))
        raise"#,
            check_sl006,
        );
        assert_eq!(log_call.status, Status::Fail);
    }

    #[test]
    fn passes_with_log_exception_and_no_exc_info() {
        let log_call = check_first_call(
            r#"def cancel_order(order_id: str):
    try:
        pass
    except Exception:
        log.exception("cancel_failed", order_id=order_id)
        raise"#,
            check_sl006,
        );
        assert_eq!(log_call.status, Status::Pass);
    }
}
