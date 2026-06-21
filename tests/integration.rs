use std::path::Path;

use rustpython_parser::Parse;
use rustpython_parser::ast::Suite;
use structloglint::config::{self, Config};
use structloglint::models::Status;
use structloglint::{analyzer, rules::case_style::CaseStyle};
use walkdir::WalkDir;

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
        "both info and warning in loops should be flagged with min=info"
    );
}

// --- rule_selection fixture ---

#[test]
fn rule_selection_select_runs_only_specified_rules() {
    let violations = analyze_fixture("rule_selection", "app.py");

    assert_eq!(
        count_rule(&violations, "SL001"),
        1,
        "SL001 should be present"
    );
    assert_eq!(
        count_rule(&violations, "SL002"),
        0,
        "SL002 should not be present because only SL001 is selected"
    );
    assert_eq!(
        count_rule(&violations, "SL003"),
        0,
        "SL003 should not be present because only SL001 is selected"
    );
    assert_eq!(
        violations.len(),
        1,
        "only SL001 violation should be reported"
    );
}

// --- rule_severity fixture ---

#[test]
fn rule_severity_off_disables_sl007() {
    let violations = analyze_fixture("rule_severity", "app.py");

    assert_eq!(
        count_rule(&violations, "SL007"),
        0,
        "SL007 should be disabled by severity=off"
    );
}

#[test]
fn rule_severity_error_promotes_sl009_to_fail() {
    let violations = analyze_fixture("rule_severity", "app.py");

    let sl009 = violations
        .iter()
        .find(|(id, _)| id == "SL009")
        .expect("SL009 should be present");
    assert_eq!(sl009.1, Status::Fail, "SL009 should be promoted to error");
}

#[test]
fn rule_severity_sl008_uses_default_fail() {
    let violations = analyze_fixture("rule_severity", "app.py");

    let sl008 = violations
        .iter()
        .find(|(id, _)| id == "SL008")
        .expect("SL008 should be present");
    assert_eq!(
        sl008.1,
        Status::Fail,
        "SL008 should retain its default error status"
    );
}

// --- File exclusion tests ---

fn walk_and_filter_fixture(fixture: &str, config: &Config) -> (Vec<String>, Vec<String>) {
    let fixture_dir = Path::new("tests/fixtures").join(fixture);
    let exclude_set = config.build_exclude_globset().unwrap();

    let mut included = Vec::new();
    let mut excluded = Vec::new();

    for entry in WalkDir::new(&fixture_dir)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file() && e.path().extension().is_some_and(|ext| ext == "py"))
    {
        let rel = entry
            .path()
            .strip_prefix(&fixture_dir)
            .unwrap_or(entry.path());
        if exclude_set.is_match(rel.to_string_lossy().as_ref()) {
            excluded.push(rel.to_string_lossy().to_string());
        } else {
            included.push(rel.to_string_lossy().to_string());
        }
    }

    (included, excluded)
}

#[test]
fn default_excludes_filters_venv_and_other_dirs() {
    let config = Config::default();
    let (_included, excluded) = walk_and_filter_fixture("file_exclusion", &config);

    let excluded_set: std::collections::HashSet<_> = excluded.iter().map(|s| s.as_str()).collect();

    assert!(
        excluded_set.contains(".venv/app.py"),
        ".venv/app.py should be excluded. excluded: {excluded:?}"
    );
    assert!(
        excluded_set.contains("venv/app.py"),
        "venv/app.py should be excluded. excluded: {excluded:?}"
    );
    assert!(
        excluded_set.contains("node_modules/some_lib/app.py"),
        "node_modules should be excluded. excluded: {excluded:?}"
    );
    assert!(
        excluded_set.contains("__pycache__/cached.py"),
        "__pycache__ should be excluded. excluded: {excluded:?}"
    );
    assert!(
        excluded_set.contains("migrations/001_init.py"),
        "migrations should be excluded. excluded: {excluded:?}"
    );
}

#[test]
fn default_excludes_lets_normal_dirs_through() {
    let config = Config::default();
    let (included, _) = walk_and_filter_fixture("file_exclusion", &config);

    let included_set: std::collections::HashSet<_> = included.iter().map(|s| s.as_str()).collect();

    assert!(
        included_set.contains("src/app.py"),
        "src/app.py should be included. included: {included:?}"
    );
    assert!(
        included_set.contains("my_module/app.py"),
        "my_module/app.py should be included (not in default excludes). included: {included:?}"
    );
}

#[test]
fn default_excludes_only_filters_matching_patterns() {
    let config = Config::default();
    let (included, excluded) = walk_and_filter_fixture("file_exclusion", &config);

    assert_eq!(
        included.len(),
        2,
        "only src/app.py and my_module/app.py should be included"
    );
    assert_eq!(
        excluded.len(),
        5,
        ".venv, venv, node_modules, __pycache__, and migrations should be excluded"
    );
}

#[test]
fn extend_exclude_adds_extra_patterns() {
    let fixture_dir = Path::new("tests/fixtures/file_exclusion_extend");
    let config = config::discover_config(fixture_dir).unwrap();

    assert_eq!(
        config.extend_exclude,
        Some(vec!["my_module/**".to_string()])
    );

    let (included, excluded) = walk_and_filter_fixture("file_exclusion_extend", &config);

    let excluded_set: std::collections::HashSet<_> = excluded.iter().map(|s| s.as_str()).collect();
    let included_set: std::collections::HashSet<_> = included.iter().map(|s| s.as_str()).collect();

    assert!(
        excluded_set.contains(".venv/app.py"),
        ".venv should still be excluded by defaults"
    );
    assert!(
        excluded_set.contains("my_module/app.py"),
        "my_module should be excluded by extend-exclude"
    );
    assert!(
        included_set.contains("src/app.py"),
        "src/app.py should be included"
    );
    assert!(
        included_set.contains("other/app.py"),
        "other/app.py should be included"
    );
}

#[test]
fn exclude_overrides_all_defaults() {
    let fixture_dir = Path::new("tests/fixtures/file_exclusion_override");
    let config = config::discover_config(fixture_dir).unwrap();

    assert_eq!(config.exclude, Some(vec![]));

    let (included, excluded) = walk_and_filter_fixture("file_exclusion_override", &config);

    assert!(
        excluded.is_empty(),
        "empty exclude list should mean no exclusions"
    );

    let included_set: std::collections::HashSet<_> = included.iter().map(|s| s.as_str()).collect();
    assert!(
        included_set.contains("src/app.py"),
        "src/app.py should be included"
    );
    assert!(
        included_set.contains(".venv/app.py"),
        ".venv/app.py should be included when exclude is explicitly empty"
    );
}
