use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use globset::{GlobBuilder, GlobSet, GlobSetBuilder};
use serde::Deserialize;

use crate::models::{LogLevel, RuleSeverity};
use crate::rules::case_style::CaseStyle;

#[derive(Debug, Clone)]
pub struct Config {
    pub case_style: CaseStyle,
    pub max_event_length: usize,
    pub min_loop_log_level: LogLevel,
    pub check_imports: bool,
    pub select: Option<Vec<String>>,
    pub ignore: Option<Vec<String>>,
    pub rules: HashMap<String, RuleSeverity>,
    pub exclude: Option<Vec<String>>,
    pub extend_exclude: Option<Vec<String>>,
    pub include: Option<Vec<String>>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            case_style: CaseStyle::SnakeCase,
            max_event_length: 30,
            min_loop_log_level: LogLevel::Info,
            check_imports: true,
            select: None,
            ignore: None,
            rules: HashMap::new(),
            exclude: None,
            extend_exclude: None,
            include: None,
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("failed to read config file: {0}")]
    Io(#[from] std::io::Error),
    #[error("failed to parse config file: {0}")]
    Toml(#[from] toml::de::Error),
    #[error("invalid case style: {0}")]
    InvalidCaseStyle(String),
    #[error("invalid log level: {0}")]
    InvalidLogLevel(String),
    #[error("invalid rule severity: {0}")]
    InvalidRuleSeverity(String),
}

#[derive(Debug, Deserialize, Default)]
#[serde(rename_all = "kebab-case")]
struct RawConfig {
    event_case_style: Option<String>,
    max_event_length: Option<usize>,
    loop_log_level: Option<String>,
    #[serde(default)]
    check_imports: Option<bool>,
    #[serde(default)]
    select: Option<Vec<String>>,
    #[serde(default)]
    ignore: Option<Vec<String>>,
    #[serde(default)]
    rules: Option<HashMap<String, String>>,
    #[serde(default)]
    exclude: Option<Vec<String>>,
    #[serde(default)]
    extend_exclude: Option<Vec<String>>,
    #[serde(default)]
    include: Option<Vec<String>>,
}

#[derive(Debug, Deserialize)]
struct PyProject {
    tool: Option<ToolTable>,
}

#[derive(Debug, Deserialize)]
struct ToolTable {
    structloglint: Option<RawConfig>,
}

impl RawConfig {
    fn into_config(self) -> Result<Config, ConfigError> {
        let defaults = Config::default();

        let case_style = match self.event_case_style {
            Some(s) => s
                .parse::<CaseStyle>()
                .map_err(|_| ConfigError::InvalidCaseStyle(s))?,
            None => defaults.case_style,
        };

        let min_loop_log_level = match self.loop_log_level {
            Some(s) => s
                .parse::<LogLevel>()
                .map_err(|_| ConfigError::InvalidLogLevel(s))?,
            None => defaults.min_loop_log_level,
        };

        let rules = match self.rules {
            Some(map) => {
                let mut parsed = HashMap::new();
                for (rule_id, severity_str) in map {
                    let severity = severity_str
                        .parse::<RuleSeverity>()
                        .map_err(|_| ConfigError::InvalidRuleSeverity(severity_str))?;
                    parsed.insert(rule_id, severity);
                }
                parsed
            }
            None => HashMap::new(),
        };

        Ok(Config {
            case_style,
            max_event_length: self.max_event_length.unwrap_or(defaults.max_event_length),
            min_loop_log_level,
            check_imports: self.check_imports.unwrap_or(defaults.check_imports),
            select: self.select,
            ignore: self.ignore,
            rules,
            exclude: self.exclude,
            extend_exclude: self.extend_exclude,
            include: self.include,
        })
    }
}

pub fn discover_config(start: &Path) -> Result<Config, ConfigError> {
    let start = if start.is_file() {
        start
            .parent()
            .map(Path::to_path_buf)
            .unwrap_or_else(|| PathBuf::from("."))
    } else {
        start.to_path_buf()
    };

    let start = fs::canonicalize(&start).unwrap_or(start);

    let mut dir = start.as_path();
    loop {
        let standalone = dir.join("structloglint.toml");
        if standalone.is_file() {
            return parse_standalone(&standalone);
        }

        let pyproject = dir.join("pyproject.toml");
        if pyproject.is_file()
            && let Some(config) = parse_pyproject(&pyproject)?
        {
            return Ok(config);
        }

        match dir.parent() {
            Some(parent) => dir = parent,
            None => break,
        }
    }

    Ok(Config::default())
}

fn parse_standalone(path: &Path) -> Result<Config, ConfigError> {
    let contents = fs::read_to_string(path)?;
    let raw: RawConfig = toml::from_str(&contents)?;
    raw.into_config()
}

fn parse_pyproject(path: &Path) -> Result<Option<Config>, ConfigError> {
    let contents = fs::read_to_string(path)?;
    let pyproject: PyProject = toml::from_str(&contents)?;

    match pyproject.tool.and_then(|t| t.structloglint) {
        Some(raw) => Ok(Some(raw.into_config()?)),
        None => Ok(None),
    }
}

pub const DEFAULT_EXCLUDES: &[&str] = &[
    "**/.venv/**",
    "**/venv/**",
    "**/node_modules/**",
    "**/__pycache__/**",
    "**/*.pyc",
    "**/*.pyo",
    "**/.git/**",
    "**/.tox/**",
    "**/.nox/**",
    "**/site-packages/**",
    "**/__pypackages__/**",
    "**/.eggs/**",
    "**/dist/**",
    "**/build/**",
    "**/.mypy_cache/**",
    "**/.pytest_cache/**",
    "**/.ruff_cache/**",
    "**/.direnv/**",
    "**/migrations/**",
];

impl Config {
    pub fn build_exclude_globset(&self) -> Result<GlobSet, globset::Error> {
        let mut builder = GlobSetBuilder::new();

        let patterns: Vec<String> = if let Some(ref exclude) = self.exclude {
            if exclude.is_empty() {
                return Ok(GlobSet::empty());
            }
            exclude.clone()
        } else {
            let mut pats: Vec<String> = DEFAULT_EXCLUDES.iter().map(|s| s.to_string()).collect();
            if let Some(ref extend) = self.extend_exclude {
                pats.extend(extend.iter().cloned());
            }
            pats
        };

        for pat in &patterns {
            let glob = GlobBuilder::new(pat).literal_separator(true).build()?;
            builder.add(glob);
        }

        builder.build()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn write_temp_file(dir: &Path, name: &str, content: &str) -> PathBuf {
        let path = dir.join(name);
        fs::write(&path, content).unwrap();
        path
    }

    #[test]
    fn default_config_values() {
        let config = Config::default();
        assert_eq!(config.case_style, CaseStyle::SnakeCase);
        assert_eq!(config.max_event_length, 30);
        assert_eq!(config.min_loop_log_level, LogLevel::Info);
    }

    #[test]
    fn parse_pyproject_with_all_fields() {
        let dir = tempfile::tempdir().unwrap();
        write_temp_file(
            dir.path(),
            "pyproject.toml",
            r#"
[tool.structloglint]
event-case-style = "camel_case"
max-event-length = 50
loop-log-level = "debug"
"#,
        );
        let config = discover_config(dir.path()).unwrap();
        assert_eq!(config.case_style, CaseStyle::CamelCase);
        assert_eq!(config.max_event_length, 50);
        assert_eq!(config.min_loop_log_level, LogLevel::Debug);
    }

    #[test]
    fn parse_pyproject_with_partial_fields() {
        let dir = tempfile::tempdir().unwrap();
        write_temp_file(
            dir.path(),
            "pyproject.toml",
            r#"
[tool.structloglint]
max-event-length = 42
"#,
        );
        let config = discover_config(dir.path()).unwrap();
        assert_eq!(config.case_style, CaseStyle::SnakeCase);
        assert_eq!(config.max_event_length, 42);
        assert_eq!(config.min_loop_log_level, LogLevel::Info);
    }

    #[test]
    fn parse_pyproject_without_structloglint_section() {
        let dir = tempfile::tempdir().unwrap();
        write_temp_file(
            dir.path(),
            "pyproject.toml",
            r#"
[tool.ruff]
line-length = 120
"#,
        );
        let config = discover_config(dir.path()).unwrap();
        assert_eq!(config.case_style, CaseStyle::SnakeCase);
        assert_eq!(config.max_event_length, 30);
    }

    #[test]
    fn parse_standalone_config() {
        let dir = tempfile::tempdir().unwrap();
        write_temp_file(
            dir.path(),
            "structloglint.toml",
            r#"
event-case-style = "kebab_case"
max-event-length = 60
"#,
        );
        let config = discover_config(dir.path()).unwrap();
        assert_eq!(config.case_style, CaseStyle::KebabCase);
        assert_eq!(config.max_event_length, 60);
    }

    #[test]
    fn standalone_takes_precedence_over_pyproject() {
        let dir = tempfile::tempdir().unwrap();
        write_temp_file(
            dir.path(),
            "structloglint.toml",
            r#"
max-event-length = 99
"#,
        );
        write_temp_file(
            dir.path(),
            "pyproject.toml",
            r#"
[tool.structloglint]
max-event-length = 10
"#,
        );
        let config = discover_config(dir.path()).unwrap();
        assert_eq!(config.max_event_length, 99);
    }

    #[test]
    fn walks_up_directory_tree() {
        let parent = tempfile::tempdir().unwrap();
        let child = parent.path().join("sub");
        fs::create_dir(&child).unwrap();

        write_temp_file(
            parent.path(),
            "pyproject.toml",
            r#"
[tool.structloglint]
max-event-length = 77
"#,
        );

        let config = discover_config(&child).unwrap();
        assert_eq!(config.max_event_length, 77);
    }

    #[test]
    fn returns_defaults_when_no_config_found() {
        let dir = tempfile::tempdir().unwrap();
        let config = discover_config(dir.path()).unwrap();
        assert_eq!(config.case_style, CaseStyle::SnakeCase);
        assert_eq!(config.max_event_length, 30);
        assert_eq!(config.min_loop_log_level, LogLevel::Info);
    }

    #[test]
    fn invalid_case_style_returns_error() {
        let dir = tempfile::tempdir().unwrap();
        write_temp_file(
            dir.path(),
            "structloglint.toml",
            r#"
event-case-style = "YELLING_CASE"
"#,
        );
        let result = discover_config(dir.path());
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("invalid case style"), "{err}");
    }

    #[test]
    fn invalid_log_level_returns_error() {
        let dir = tempfile::tempdir().unwrap();
        write_temp_file(
            dir.path(),
            "structloglint.toml",
            r#"
loop-log-level = "verbose"
"#,
        );
        let result = discover_config(dir.path());
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("invalid log level"), "{err}");
    }

    #[test]
    fn discovers_config_from_file_path() {
        let dir = tempfile::tempdir().unwrap();
        write_temp_file(
            dir.path(),
            "pyproject.toml",
            r#"
[tool.structloglint]
max-event-length = 55
"#,
        );
        let file = write_temp_file(dir.path(), "app.py", "import structlog\n");

        let config = discover_config(&file).unwrap();
        assert_eq!(config.max_event_length, 55);
    }

    #[test]
    fn pyproject_with_empty_structloglint_uses_defaults() {
        let dir = tempfile::tempdir().unwrap();
        write_temp_file(
            dir.path(),
            "pyproject.toml",
            r#"
[tool.structloglint]
"#,
        );
        let config = discover_config(dir.path()).unwrap();
        assert_eq!(config.case_style, CaseStyle::SnakeCase);
        assert_eq!(config.max_event_length, 30);
        assert_eq!(config.min_loop_log_level, LogLevel::Info);
    }

    #[test]
    fn pyproject_with_select_and_ignore() {
        let dir = tempfile::tempdir().unwrap();
        write_temp_file(
            dir.path(),
            "pyproject.toml",
            r#"
[tool.structloglint]
select = ["SL001", "SL002"]
ignore = ["SL007"]
"#,
        );
        let config = discover_config(dir.path()).unwrap();
        assert_eq!(
            config.select,
            Some(vec!["SL001".to_string(), "SL002".to_string()])
        );
        assert_eq!(config.ignore, Some(vec!["SL007".to_string()]));
        assert!(config.rules.is_empty());
    }

    #[test]
    fn pyproject_with_per_rule_severity() {
        let dir = tempfile::tempdir().unwrap();
        write_temp_file(
            dir.path(),
            "pyproject.toml",
            r#"
[tool.structloglint.rules]
SL006 = "error"
SL007 = "off"
SL009 = "error"
"#,
        );
        let config = discover_config(dir.path()).unwrap();
        assert_eq!(config.rules.len(), 3);
        assert_eq!(config.rules.get("SL006"), Some(&RuleSeverity::Error));
        assert_eq!(config.rules.get("SL007"), Some(&RuleSeverity::Off));
        assert_eq!(config.rules.get("SL009"), Some(&RuleSeverity::Error));
    }

    #[test]
    fn invalid_rule_severity_returns_error() {
        let dir = tempfile::tempdir().unwrap();
        write_temp_file(
            dir.path(),
            "structloglint.toml",
            r#"
[rules]
SL001 = "fatal"
"#,
        );
        let result = discover_config(dir.path());
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("invalid rule severity"), "{err}");
    }

    #[test]
    fn standalone_config_with_rules_and_select() {
        let dir = tempfile::tempdir().unwrap();
        write_temp_file(
            dir.path(),
            "structloglint.toml",
            r#"
select = ["SL001", "SL008", "SL009"]
ignore = ["SL009"]

[rules]
SL008 = "warning"
"#,
        );
        let config = discover_config(dir.path()).unwrap();
        assert_eq!(
            config.select,
            Some(vec![
                "SL001".to_string(),
                "SL008".to_string(),
                "SL009".to_string()
            ])
        );
        assert_eq!(config.ignore, Some(vec!["SL009".to_string()]));
        assert_eq!(config.rules.get("SL008"), Some(&RuleSeverity::Warning));
    }

    fn is_excluded(set: &GlobSet, path: &str) -> bool {
        let candidate = globset::Candidate::new(path);
        set.is_match_candidate(&candidate)
    }

    #[test]
    fn default_excludes_venv_directories() {
        let config = Config::default();
        let ov = config.build_exclude_globset().unwrap();

        assert!(is_excluded(&ov, ".venv/app.py"));
        assert!(is_excluded(&ov, "venv/app.py"));
        assert!(is_excluded(&ov, "some/deep/path/.venv/lib/module.py"));
        assert!(is_excluded(&ov, "another/venv/something.py"));
    }

    #[test]
    fn default_excludes_node_modules_and_cache() {
        let config = Config::default();
        let ov = config.build_exclude_globset().unwrap();

        assert!(is_excluded(&ov, "node_modules/some_lib/file.py"));
        assert!(is_excluded(&ov, "project/node_modules/pkg/index.py"));
        assert!(is_excluded(&ov, "__pycache__/cached.py"));
        assert!(is_excluded(&ov, "migrations/001_init.py"));
    }

    #[test]
    fn default_excludes_dot_directories() {
        let config = Config::default();
        let ov = config.build_exclude_globset().unwrap();

        assert!(is_excluded(&ov, ".git/index.py"));
        assert!(is_excluded(&ov, ".tox/some.py"));
        assert!(is_excluded(&ov, ".nox/some.py"));
        assert!(is_excluded(&ov, ".mypy_cache/something.py"));
        assert!(is_excluded(&ov, ".pytest_cache/v/cache/lastfailed"));
        assert!(is_excluded(&ov, ".ruff_cache/0.0.0/1234567890.py"));
    }

    #[test]
    fn default_excludes_pyc_and_pyo_files() {
        let config = Config::default();
        let ov = config.build_exclude_globset().unwrap();

        assert!(is_excluded(&ov, "foo.pyc"));
        assert!(is_excluded(&ov, "bar.pyo"));
        assert!(is_excluded(&ov, "some/deep/path.pyc"));
    }

    #[test]
    fn default_excludes_allows_normal_python_files() {
        let config = Config::default();
        let ov = config.build_exclude_globset().unwrap();

        assert!(!is_excluded(&ov, "src/main.py"));
        assert!(!is_excluded(&ov, "app.py"));
        assert!(!is_excluded(&ov, "tests/test_app.py"));
        assert!(!is_excluded(&ov, "my_module/utils.py"));
        assert!(!is_excluded(&ov, "some_venv/app.py"));
        assert!(!is_excluded(&ov, "environment/app.py"));
    }

    #[test]
    fn extend_exclude_adds_patterns() {
        let mut config = Config::default();
        config.extend_exclude = Some(vec!["my_internal/**".to_string()]);

        let ov = config.build_exclude_globset().unwrap();

        assert!(is_excluded(&ov, ".venv/app.py"));
        assert!(is_excluded(&ov, "my_internal/secret.py"));
        assert!(!is_excluded(&ov, "src/main.py"));
    }

    #[test]
    fn exclude_overrides_defaults() {
        let mut config = Config::default();
        config.exclude = Some(vec!["src/**".to_string(), "tests/test_*.py".to_string()]);

        let ov = config.build_exclude_globset().unwrap();

        assert!(is_excluded(&ov, "src/main.py"));
        assert!(is_excluded(&ov, "tests/test_app.py"));
        assert!(is_excluded(&ov, "tests/test_utils.py"));
        assert!(
            !is_excluded(&ov, ".venv/app.py"),
            ".venv should NOT be excluded when explicitly overriding excludes"
        );
        assert!(!is_excluded(&ov, "app.py"));
    }

    #[test]
    fn empty_exclude_disables_all_excludes() {
        let mut config = Config::default();
        config.exclude = Some(vec![]);

        let ov = config.build_exclude_globset().unwrap();

        assert!(!is_excluded(&ov, ".venv/app.py"));
        assert!(!is_excluded(&ov, "node_modules/pkg.py"));
        assert!(!is_excluded(&ov, "src/main.py"));
    }

    #[test]
    fn empty_extend_exclude_does_not_affect_defaults() {
        let mut config = Config::default();
        config.extend_exclude = Some(vec![]);

        let ov = config.build_exclude_globset().unwrap();

        assert!(is_excluded(&ov, ".venv/app.py"));
        assert!(is_excluded(&ov, "node_modules/pkg.py"));
        assert!(!is_excluded(&ov, "src/main.py"));
    }

    #[test]
    fn pyproject_parses_exclude() {
        let dir = tempfile::tempdir().unwrap();
        write_temp_file(
            dir.path(),
            "pyproject.toml",
            r#"
[tool.structloglint]
exclude = ["generated/**", "third_party/**"]
"#,
        );
        let config = discover_config(dir.path()).unwrap();

        assert_eq!(
            config.exclude,
            Some(vec![
                "generated/**".to_string(),
                "third_party/**".to_string()
            ])
        );
        assert!(config.extend_exclude.is_none());

        let ov = config.build_exclude_globset().unwrap();
        assert!(is_excluded(&ov, "generated/code.py"));
        assert!(is_excluded(&ov, "third_party/lib.py"));
        assert!(
            !is_excluded(&ov, ".venv/app.py"),
            "default excludes should be overridden"
        );
        assert!(!is_excluded(&ov, "src/app.py"));
    }

    #[test]
    fn pyproject_parses_extend_exclude() {
        let dir = tempfile::tempdir().unwrap();
        write_temp_file(
            dir.path(),
            "pyproject.toml",
            r#"
[tool.structloglint]
extend-exclude = ["generated/**"]
"#,
        );
        let config = discover_config(dir.path()).unwrap();

        assert_eq!(
            config.extend_exclude,
            Some(vec!["generated/**".to_string()])
        );
        assert!(config.exclude.is_none());

        let ov = config.build_exclude_globset().unwrap();
        assert!(is_excluded(&ov, "generated/code.py"));
        assert!(
            is_excluded(&ov, ".venv/app.py"),
            "default excludes should still apply"
        );
        assert!(!is_excluded(&ov, "src/app.py"));
    }

    #[test]
    fn pyproject_parses_include() {
        let dir = tempfile::tempdir().unwrap();
        write_temp_file(
            dir.path(),
            "pyproject.toml",
            r#"
[tool.structloglint]
include = ["src/**"]
"#,
        );
        let config = discover_config(dir.path()).unwrap();

        assert_eq!(config.include, Some(vec!["src/**".to_string()]));
    }
}
