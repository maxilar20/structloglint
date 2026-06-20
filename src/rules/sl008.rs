use rustpython_parser::ast;

use super::case_style::CaseStyle;
use crate::models::{Fix, LogCall, RuleResult, Status};

/// SL008 — Event string must match the configured case style.
pub fn check_sl008(log_call: &LogCall, case_style: CaseStyle) -> RuleResult {
    let Some(first_arg) = log_call.call.args.first() else {
        return RuleResult::new("SL008", Status::Pass, String::new());
    };

    let ast::Expr::Constant(constant) = first_arg else {
        return RuleResult::new("SL008", Status::Pass, String::new());
    };

    let ast::Constant::Str(event) = &constant.value else {
        return RuleResult::new("SL008", Status::Pass, String::new());
    };

    if case_style.is_match(event) {
        return RuleResult::new("SL008", Status::Pass, String::new());
    }

    let converted = case_style.convert(event);
    let fix = Fix {
        replacement: format!("\"{}\"", converted),
        start: u32::from(constant.range.start()) as usize,
        end: u32::from(constant.range.end()) as usize,
    };

    RuleResult::new(
        "SL008",
        Status::Fail,
        format!(
            "Event string \"{}\" does not match {} style, expected \"{}\"",
            event, case_style, converted,
        ),
    )
    .with_fix(fix)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rules::test_helpers::check_first_call;

    fn checker(case_style: CaseStyle) -> impl Fn(&LogCall) -> RuleResult {
        move |log_call| check_sl008(log_call, case_style)
    }

    #[test]
    fn passes_snake_case() {
        let result = check_first_call(
            r#"log.info("profile_updated", user_id="u_123")"#,
            checker(CaseStyle::SnakeCase),
        );
        assert_eq!(result.status, Status::Pass, "{}", result.feedback);
    }

    #[test]
    fn fails_pascal_case_when_snake_required() {
        let result = check_first_call(
            r#"log.info("ProfileUpdated", user_id="u_123")"#,
            checker(CaseStyle::SnakeCase),
        );
        assert_eq!(result.status, Status::Fail, "{}", result.feedback);
    }

    #[test]
    fn fails_camel_case_when_snake_required() {
        let result = check_first_call(
            r#"log.info("profileUpdated", user_id="u_123")"#,
            checker(CaseStyle::SnakeCase),
        );
        assert_eq!(result.status, Status::Fail, "{}", result.feedback);
    }

    #[test]
    fn fails_spaces_when_snake_required() {
        let result = check_first_call(
            r#"log.info("profile updated successfully", user_id="u_123")"#,
            checker(CaseStyle::SnakeCase),
        );
        assert_eq!(result.status, Status::Fail, "{}", result.feedback);
    }

    #[test]
    fn fails_kebab_case_when_snake_required() {
        let result = check_first_call(
            r#"log.info("profile-updated", user_id="u_123")"#,
            checker(CaseStyle::SnakeCase),
        );
        assert_eq!(result.status, Status::Fail, "{}", result.feedback);
    }

    #[test]
    fn fails_screaming_snake_when_snake_required() {
        let result = check_first_call(
            r#"log.info("PROFILE_UPDATED", user_id="u_123")"#,
            checker(CaseStyle::SnakeCase),
        );
        assert_eq!(result.status, Status::Fail, "{}", result.feedback);
    }

    #[test]
    fn passes_no_event_arg() {
        let result = check_first_call(
            r#"log.info(user_id="u_123")"#,
            checker(CaseStyle::SnakeCase),
        );
        assert_eq!(result.status, Status::Pass, "{}", result.feedback);
    }
}
