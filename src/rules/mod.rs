use crate::config::Config;
use crate::models::{LogCall, RuleResult, RuleSeverity, Status};

pub mod case_style;
mod expr_helpers;
mod sl001;
mod sl002;
mod sl003;
mod sl004;
mod sl005;
mod sl006;
mod sl007;
mod sl008;
mod sl009;

pub fn check_all(log_call: &LogCall, config: &Config) -> Vec<RuleResult> {
    let should_run = |rule_id: &str| -> bool {
        if let Some(ref select) = config.select
            && !select.iter().any(|s| s == rule_id)
        {
            return false;
        }
        if let Some(ref ignore) = config.ignore
            && ignore.iter().any(|s| s == rule_id)
        {
            return false;
        }
        !matches!(config.rules.get(rule_id), Some(RuleSeverity::Off))
    };

    let apply_severity = |mut result: RuleResult| -> RuleResult {
        if result.status != Status::Pass
            && let Some(severity) = config.rules.get(result.rule_id)
        {
            result.status = match severity {
                RuleSeverity::Error => Status::Fail,
                RuleSeverity::Warning => Status::Warning,
                RuleSeverity::Off => result.status,
            };
        }
        result
    };

    let mut results = Vec::with_capacity(9);

    if should_run("SL001") {
        results.push(apply_severity(sl001::check_sl001(log_call.call)));
    }
    if should_run("SL002") {
        results.push(apply_severity(sl002::check_sl002(log_call.call)));
    }
    if should_run("SL003") {
        results.push(apply_severity(sl003::check_sl003(log_call.call)));
    }
    if should_run("SL004") {
        results.push(apply_severity(sl004::check_sl004(log_call.call)));
    }
    if should_run("SL005") {
        results.push(apply_severity(sl005::check_sl005(log_call)));
    }
    if should_run("SL006") {
        results.push(apply_severity(sl006::check_sl006(log_call)));
    }
    if should_run("SL007") {
        results.push(apply_severity(sl007::check_sl007(
            log_call,
            config.min_loop_log_level,
        )));
    }
    if should_run("SL008") {
        results.push(apply_severity(sl008::check_sl008(
            log_call,
            config.case_style,
        )));
    }
    if should_run("SL009") {
        results.push(apply_severity(sl009::check_sl009(
            log_call,
            config.max_event_length,
        )));
    }

    results
}

#[cfg(test)]
mod test_helpers {
    use crate::ast_walker::{ParentContext, collect_log_calls};
    use crate::models::{LogCall, RuleResult};
    use rustpython_parser::Parse;
    use rustpython_parser::ast::Suite;

    pub fn check_first_call<F>(source: &str, check_fn: F) -> RuleResult
    where
        F: Fn(&LogCall) -> RuleResult,
    {
        let stmts = Suite::parse(source, "<test>").expect("parse failed");
        let calls: Vec<LogCall> = stmts
            .iter()
            .flat_map(|s| collect_log_calls(s, ParentContext::Module))
            .collect();
        let call = calls.first().expect("no log call found");
        check_fn(call)
    }

    pub fn check_first_call_expr(
        source: &str,
        check_fn: fn(&rustpython_parser::ast::ExprCall) -> RuleResult,
    ) -> RuleResult {
        let stmts = Suite::parse(source, "<test>").expect("parse failed");
        let calls: Vec<LogCall> = stmts
            .iter()
            .flat_map(|s| collect_log_calls(s, ParentContext::Module))
            .collect();
        let call = calls.first().expect("no log call found");
        check_fn(call.call)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast_walker::{ParentContext, collect_log_calls};
    use crate::models::RuleSeverity;
    use crate::models::Status;
    use rustpython_parser::Parse;
    use rustpython_parser::ast::Suite;

    #[test]
    fn sl009_default_rejects_event_over_30_chars() {
        let source = r#"log.info("this_event_string_is_thirty_five_chars")"#;
        let stmts = Suite::parse(source, "<test>").expect("parse failed");
        let calls: Vec<_> = stmts
            .iter()
            .flat_map(|s| collect_log_calls(s, ParentContext::Module))
            .collect();
        let call = calls.first().expect("no log call found");
        let results = check_all(call, &Config::default());
        let sl009 = results.iter().find(|r| r.rule_id == "SL009").unwrap();
        assert_eq!(
            sl009.status,
            Status::Warning,
            "a 35-char event should warn with default max of 30",
        );
    }

    #[test]
    fn sl009_default_allows_event_under_30_chars() {
        let source = r#"log.info("short_event_string")"#;
        let stmts = Suite::parse(source, "<test>").expect("parse failed");
        let calls: Vec<_> = stmts
            .iter()
            .flat_map(|s| collect_log_calls(s, ParentContext::Module))
            .collect();
        let call = calls.first().expect("no log call found");
        let results = check_all(call, &Config::default());
        let sl009 = results.iter().find(|r| r.rule_id == "SL009").unwrap();
        assert_eq!(
            sl009.status,
            Status::Pass,
            "an 18-char event should pass with default max of 30",
        );
    }

    #[test]
    fn select_runs_only_specified_rules() {
        let source = "log.info('user_logged_in', 'extra')\n";
        let stmts = Suite::parse(source, "<test>").expect("parse failed");
        let calls: Vec<_> = stmts
            .iter()
            .flat_map(|s| collect_log_calls(s, ParentContext::Module))
            .collect();
        let call = calls.first().expect("no log call found");

        let mut config = Config::default();
        config.select = Some(vec!["SL001".to_string()]);
        let results = check_all(call, &config);

        let rule_ids: Vec<&str> = results.iter().map(|r| r.rule_id).collect();
        assert_eq!(rule_ids, vec!["SL001"]);
    }

    #[test]
    fn ignore_skips_specified_rules() {
        let source = "log.info('user_logged_in', 'extra')\n";
        let stmts = Suite::parse(source, "<test>").expect("parse failed");
        let calls: Vec<_> = stmts
            .iter()
            .flat_map(|s| collect_log_calls(s, ParentContext::Module))
            .collect();
        let call = calls.first().expect("no log call found");

        let mut config = Config::default();
        config.ignore = Some(vec!["SL001".to_string()]);
        let results = check_all(call, &config);

        let sl001 = results.iter().find(|r| r.rule_id == "SL001");
        assert!(sl001.is_none(), "SL001 should be skipped entirely");
    }

    #[test]
    fn severity_off_disables_rule() {
        let source = "log.info('user_logged_in', 'extra')\n";
        let stmts = Suite::parse(source, "<test>").expect("parse failed");
        let calls: Vec<_> = stmts
            .iter()
            .flat_map(|s| collect_log_calls(s, ParentContext::Module))
            .collect();
        let call = calls.first().expect("no log call found");

        let mut config = Config::default();
        config.rules.insert("SL001".to_string(), RuleSeverity::Off);
        let results = check_all(call, &config);

        let sl001 = results.iter().find(|r| r.rule_id == "SL001");
        assert!(sl001.is_none(), "SL001 should be disabled");
    }

    #[test]
    fn severity_error_promotes_warning_to_fail() {
        let source = "log.info(\"this_event_string_is_thirty_five_chars\")\n";
        let stmts = Suite::parse(source, "<test>").expect("parse failed");
        let calls: Vec<_> = stmts
            .iter()
            .flat_map(|s| collect_log_calls(s, ParentContext::Module))
            .collect();
        let call = calls.first().expect("no log call found");

        let mut config = Config::default();
        config
            .rules
            .insert("SL009".to_string(), RuleSeverity::Error);
        let results = check_all(call, &config);

        let sl009 = results.iter().find(|r| r.rule_id == "SL009").unwrap();
        assert_eq!(
            sl009.status,
            Status::Fail,
            "SL009 should be promoted to error"
        );
    }

    #[test]
    fn select_takes_precedence_over_ignore() {
        let source = "log.info('user_logged_in', 'extra')\n";
        let stmts = Suite::parse(source, "<test>").expect("parse failed");
        let calls: Vec<_> = stmts
            .iter()
            .flat_map(|s| collect_log_calls(s, ParentContext::Module))
            .collect();
        let call = calls.first().expect("no log call found");

        let mut config = Config::default();
        config.select = Some(vec!["SL001".to_string(), "SL002".to_string()]);
        config.ignore = Some(vec!["SL001".to_string()]);
        let results = check_all(call, &config);

        let rule_ids: Vec<&str> = results.iter().map(|r| r.rule_id).collect();
        assert_eq!(rule_ids, vec!["SL002"]);
    }

    #[test]
    fn select_multiple_rules() {
        let source = "log.info('user_logged_in', 'extra')\n";
        let stmts = Suite::parse(source, "<test>").expect("parse failed");
        let calls: Vec<_> = stmts
            .iter()
            .flat_map(|s| collect_log_calls(s, ParentContext::Module))
            .collect();
        let call = calls.first().expect("no log call found");

        let mut config = Config::default();
        config.select = Some(vec!["SL001".to_string(), "SL002".to_string()]);
        let results = check_all(call, &config);

        let rule_ids: Vec<&str> = results.iter().map(|r| r.rule_id).collect();
        assert_eq!(rule_ids, vec!["SL001", "SL002"]);
    }
}
