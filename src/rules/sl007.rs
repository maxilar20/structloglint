use crate::ast_walker::ParentContext;
use crate::models::{LogCall, LogLevel, RuleResult, Status};

/// SL007 — Log call inside a loop body.
pub fn check_sl007(log_call: &LogCall, min_log_level: LogLevel) -> RuleResult {
    if log_call.level < min_log_level {
        return RuleResult::new("SL007", Status::Pass, String::new());
    }

    if matches!(log_call.context, ParentContext::While | ParentContext::For) {
        let feedback = format!(
            "logging at level `{min_log_level}` or above inside a loop body",
            min_log_level = min_log_level.as_str()
        );
        return RuleResult::new("SL007", Status::Fail, feedback);
    }

    RuleResult::new("SL007", Status::Pass, String::new())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rules::test_helpers::check_first_call;

    fn check_sl007_with_min_level(log_level: LogLevel) -> impl Fn(&LogCall) -> RuleResult {
        move |log_call| check_sl007(log_call, log_level)
    }

    #[test]
    fn passes_outside_loop() {
        let checker = check_sl007_with_min_level(LogLevel::Info);
        let rule_result = check_first_call(
            r#"log.info("product_import_started", count=len(products))"#,
            checker,
        );
        assert_eq!(rule_result.status, Status::Pass, "{}", rule_result.feedback);
    }

    #[test]
    fn passes_inside_for_loop_with_debug_level() {
        let checker = check_sl007_with_min_level(LogLevel::Info);
        let rule_result = check_first_call(
            r#"for product in products:
    log.debug("product_imported", product_id=product["id"])"#,
            checker,
        );
        assert_eq!(rule_result.status, Status::Pass, "{}", rule_result.feedback);
    }

    #[test]
    fn fails_inside_for_loop_with_info_level() {
        let checker = check_sl007_with_min_level(LogLevel::Info);
        let rule_result = check_first_call(
            r#"for product in products:
    log.info("product_imported", product_id=product["id"])"#,
            checker,
        );
        assert_eq!(rule_result.status, Status::Fail, "{}", rule_result.feedback);
    }

    #[test]
    fn passes_inside_while_loop_with_debug_level() {
        let checker = check_sl007_with_min_level(LogLevel::Info);
        let rule_result = check_first_call(
            r#"i = 0
while i < len(products):
    log.debug("processing_product", index=i)
    i += 1"#,
            checker,
        );
        assert_eq!(rule_result.status, Status::Pass, "{}", rule_result.feedback);
    }

    #[test]
    fn fails_inside_while_loop_with_info_level() {
        let checker = check_sl007_with_min_level(LogLevel::Info);
        let rule_result = check_first_call(
            r#"i = 0
while i < len(products):
    log.info("processing_product", index=i)
    i += 1"#,
            checker,
        );
        assert_eq!(rule_result.status, Status::Fail, "{}", rule_result.feedback);
    }

    #[test]
    fn passes_inside_for_loop_else_statement() {
        let checker = check_sl007_with_min_level(LogLevel::Info);
        let rule_result = check_first_call(
            r#"for x in []:
    pass
else:
    log.info("empty_loop_complete")"#,
            checker,
        );
        assert_eq!(rule_result.status, Status::Pass, "{}", rule_result.feedback);
    }
}
