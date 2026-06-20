use std::fmt;
use std::fs;
use std::path::{Path, PathBuf};

use serde::Deserialize;

use crate::models::LogLevel;
use crate::rules::case_style::CaseStyle;

#[derive(Debug, Clone)]
pub struct Config {
    pub case_style: CaseStyle,
    pub max_event_length: usize,
    pub min_loop_log_level: LogLevel,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            case_style: CaseStyle::SnakeCase,
            max_event_length: 30,
            min_loop_log_level: LogLevel::Info,
        }
    }
}

#[derive(Debug)]
pub enum ConfigError {
    Io(std::io::Error),
    Toml(toml::de::Error),
    InvalidCaseStyle(String),
    InvalidLogLevel(String),
}

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Io(e) => write!(f, "failed to read config file: {e}"),
            Self::Toml(e) => write!(f, "failed to parse config file: {e}"),
            Self::InvalidCaseStyle(s) => write!(f, "invalid case style: {s}"),
            Self::InvalidLogLevel(s) => write!(f, "invalid log level: {s}"),
        }
    }
}

impl std::error::Error for ConfigError {}

impl From<std::io::Error> for ConfigError {
    fn from(e: std::io::Error) -> Self {
        Self::Io(e)
    }
}

impl From<toml::de::Error> for ConfigError {
    fn from(e: toml::de::Error) -> Self {
        Self::Toml(e)
    }
}

#[derive(Debug, Deserialize, Default)]
#[serde(rename_all = "kebab-case")]
struct RawConfig {
    event_case_style: Option<String>,
    max_event_length: Option<usize>,
    loop_log_level: Option<String>,
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
                .map_err(|()| ConfigError::InvalidCaseStyle(s))?,
            None => defaults.case_style,
        };

        let min_loop_log_level = match self.loop_log_level {
            Some(s) => s
                .parse::<LogLevel>()
                .map_err(|()| ConfigError::InvalidLogLevel(s))?,
            None => defaults.min_loop_log_level,
        };

        Ok(Config {
            case_style,
            max_event_length: self.max_event_length.unwrap_or(defaults.max_event_length),
            min_loop_log_level,
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
event-case-style = "camelCase"
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
event-case-style = "kebab-case"
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
}
