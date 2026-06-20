use rustpython_parser::ast;

use crate::expr_helpers::is_format;
use crate::models::{RuleResult, Status};

pub fn check_sl005(call: &ast::ExprCall) -> RuleResult {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rules::test_helpers::check_first_call;

    #[test]
    fn passes_with_keyword_arguments() {
        let result = check_first_call("", check_sl005);
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
