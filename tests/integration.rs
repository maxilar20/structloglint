use std::path::Path;

use rustpython_parser::Parse;
use rustpython_parser::ast::Suite;
use structloglint::config::{self, Config};
use structloglint::models::Status;
use structloglint::{analyzer, rules::case_style::CaseStyle};

fn analyze_fixture(fixture: &str, file: &str) -> Vec<(String, Status)> {
    let fixture_dir = Path::new("tests/fixtures").join(fixture);
    let config = config::discover_config(&fixture_dir).unwrap();
    let full_path = fixture_dir.join(file);
    let source = std::fs::read_to_string(&full_path).unwrap();
    let stmts = Suite::parse(&source, &full_path.to_string_lossy()).unwrap();
    let findings = analyzer::analyze(&stmts, &config);

    findings
        .into_iter()
        .flat_map(|f| f.results)
        .filter(|r| r.status != Status::Pass)
        .map(|r| (r.rule_id.to_string(), r.status))
        .collect()
}

fn count_rule(violations: &[(String, Status)], rule: &str) -> usize {
    violations.iter().filter(|(id, _)| id == rule).count()
}

fn fixture_config(fixture: &str) -> Config {
    let fixture_dir = Path::new("tests/fixtures").join(fixture);
    config::discover_config(&fixture_dir).unwrap()
}

// --- Config discovery tests ---

#[test]
fn default_config_uses_defaults_when_no_section() {
    let config = fixture_config("default_config");
    assert_eq!(config.case_style, CaseStyle::SnakeCase);
    assert_eq!(config.max_event_length, 30);
    assert_eq!(
        config.min_loop_log_level,
        structloglint::models::LogLevel::Info
    );
}

#[test]
fn custom_config_reads_pyproject_toml() {
    let config = fixture_config("custom_config");
    assert_eq!(config.case_style, CaseStyle::CamelCase);
    assert_eq!(config.max_event_length, 50);
    assert_eq!(
        config.min_loop_log_level,
        structloglint::models::LogLevel::Warning
    );
}

#[test]
fn standalone_config_reads_structloglint_toml() {
    let config = fixture_config("standalone_config");
    assert_eq!(config.case_style, CaseStyle::KebabCase);
    assert_eq!(config.max_event_length, 40);
    assert_eq!(
        config.min_loop_log_level,
        structloglint::models::LogLevel::Info
    );
}

#[test]
fn clean_project_reads_empty_section_as_defaults() {
    let config = fixture_config("clean_project");
    assert_eq!(config.case_style, CaseStyle::SnakeCase);
    assert_eq!(config.max_event_length, 30);
}

// --- default_config fixture ---

#[test]
fn default_config_detects_all_rule_violations() {
    let violations = analyze_fixture("default_config", "src/app.py");

    assert_eq!(count_rule(&violations, "SL001"), 1);
    assert_eq!(count_rule(&violations, "SL002"), 1);
    assert_eq!(count_rule(&violations, "SL003"), 1);
    assert_eq!(count_rule(&violations, "SL004"), 1);
    assert_eq!(count_rule(&violations, "SL005"), 1);
    assert_eq!(count_rule(&violations, "SL006"), 1);
    assert_eq!(count_rule(&violations, "SL007"), 1);
    assert_eq!(count_rule(&violations, "SL008"), 2);
    assert_eq!(count_rule(&violations, "SL009"), 1);
}

#[test]
fn default_config_total_violation_count() {
    let violations = analyze_fixture("default_config", "src/app.py");
    assert_eq!(violations.len(), 10);
}

// --- custom_config fixture ---

#[test]
fn custom_config_camel_case_passes() {
    let violations = analyze_fixture("custom_config", "app.py");
    let camel_violations: Vec<_> = violations.iter().filter(|(id, _)| id == "SL008").collect();
    assert_eq!(
        camel_violations.len(),
        1,
        "only snake_case event should fail SL008"
    );
}

#[test]
fn custom_config_max_length_50() {
    let violations = analyze_fixture("custom_config", "app.py");
    assert_eq!(
        count_rule(&violations, "SL009"),
        1,
        "only event over 50 should fail"
    );
}

#[test]
fn custom_config_info_in_loop_allowed() {
    let violations = analyze_fixture("custom_config", "app.py");
    assert_eq!(
        count_rule(&violations, "SL007"),
        1,
        "only warning-level in loop should fail with min=warning"
    );
}

// --- clean_project fixture ---

#[test]
fn clean_project_has_no_violations() {
    let violations = analyze_fixture("clean_project", "app.py");
    assert!(
        violations.is_empty(),
        "clean project should have zero violations: {violations:?}"
    );
}

// --- no_structlog fixture ---

#[test]
fn no_structlog_import_produces_no_findings() {
    let fixture_dir = Path::new("tests/fixtures/no_structlog");
    let config = Config::default();
    let source = std::fs::read_to_string(fixture_dir.join("app.py")).unwrap();
    let stmts = Suite::parse(&source, "app.py").unwrap();
    let findings = analyzer::analyze(&stmts, &config);
    assert!(findings.is_empty());
}

// --- standalone_config fixture ---

#[test]
fn standalone_config_kebab_case_enforced() {
    let violations = analyze_fixture("standalone_config", "app.py");
    assert_eq!(
        count_rule(&violations, "SL008"),
        1,
        "snake_case event should fail when kebab-case required"
    );
}

#[test]
fn standalone_config_max_length_40() {
    let violations = analyze_fixture("standalone_config", "app.py");
    assert_eq!(count_rule(&violations, "SL009"), 1);
}

#[test]
fn standalone_config_loop_log_level_info() {
    let violations = analyze_fixture("standalone_config", "app.py");
    assert_eq!(
        count_rule(&violations, "SL007"),
        2,
        "both info and warning in loops should fail with min=info"
    );
}
