use rustpython_parser::ast::{self, Expr::Constant, Expr::Name};

use crate::models::{RuleResult, Status};

pub fn check_sl002(call: &ast::ExprCall) -> RuleResult {
    if call.args.len() > 0 {
        let first_arg = &call.args[0];
        match first_arg {
            Constant(_) => {
                return RuleResult::new("SL002", Status::Pass, String::new());
            }
            Name(_) => {
                return RuleResult::new(
                    "SL002",
                    Status::Warning,
                    "Passing a variable as the event argument in structlog, consider using a constant string instead".to_string(),
                );
            }
            _ => {
                return RuleResult::new(
                    "SL002",
                    Status::Fail,
                    "Passing an f-string as the event argument in structlog, consider using a constant string instead".to_string(),
                );
            }
        }
    }

    RuleResult::new("SL002", Status::Pass, String::new())
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
        check_sl002(call)
    }

    #[test]
    fn passes_with_plain_string_event() {
        let result = check_first_call("log.info('user_logged_in')");
        assert_eq!(result.status, Status::Pass, "{}", result.feedback);
    }

    #[test]
    fn passes_with_keyword_args_only() {
        let result = check_first_call("log.info('user_logged_in', user_id='u_123', ip='1.2.3.4')");
        assert_eq!(result.status, Status::Pass, "{}", result.feedback);
    }

    #[test]
    fn fails_with_f_string_but_no_interpolation() {
        let result = check_first_call("log.info(f'user logged in')");
        assert_eq!(result.status, Status::Fail, "{}", result.feedback);
    }

    #[test]
    fn fails_with_f_string_with_interpolation() {
        let result = check_first_call(r#"log.warning(f"rate limit exceeded for {'u_123'}")"#);
        assert_eq!(result.status, Status::Fail, "{}", result.feedback)
    }

    #[test]
    fn fails_with_f_string_with_interpolation_log_error() {
        let result =
            check_first_call(r#"log.error(f"login failed for u_123 from 1.2.3.4", exc_info=True)"#);
        assert_eq!(result.status, Status::Fail, "{}", result.feedback)
    }

    #[test]
    fn warns_when_passing_variable_as_event_arg() {
        let result = check_first_call("log.info(var)");
        assert_eq!(result.status, Status::Warning, "{}", result.feedback)
    }

    #[test]
    fn passes_when_constant_string_event_and_variable_kwarg() {
        let result = check_first_call("log.info('user_logged_in', user_id=var)");
        assert_eq!(result.status, Status::Pass, "{}", result.feedback)
    }

    #[test]
    fn passes_when_constant_string_event_and_f_string_kwarg() {
        let result = check_first_call("log.info('user_logged_in', user_id=f'user_{var}')");
        assert_eq!(result.status, Status::Pass, "{}", result.feedback)
    }
}
