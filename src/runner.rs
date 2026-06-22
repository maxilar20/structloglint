use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use globset::{Candidate, GlobSet};
use ignore::WalkBuilder;
use rayon::prelude::*;
use rustpython_parser::Parse;

use crate::analyzer;
use crate::config::Config;
use crate::display::{self, OutputFormat};

#[derive(Debug)]
pub struct FileResult {
    pub buf: Vec<u8>,
    pub errors: usize,
    pub warnings: usize,
    pub statements: usize,
}

pub struct LintResults {
    pub succeeded: Vec<FileResult>,
    pub failed: Vec<(String, String)>,
}

pub fn process_source(
    source: &str,
    file_path: &str,
    config: &Config,
    output_format: OutputFormat,
) -> Result<FileResult, String> {
    let stmts = rustpython_parser::ast::Suite::parse(source, file_path)
        .map_err(|e| format!("failed to parse: {e}"))?;
    let findings = analyzer::analyze(&stmts, config);
    let mut buf = Vec::new();
    let (errors, warnings) =
        display::print_diagnostics(&mut buf, &findings, file_path, source, output_format)
            .map_err(|e| format!("failed to write diagnostics: {e}"))?;
    Ok(FileResult {
        buf,
        errors,
        warnings,
        statements: findings.len(),
    })
}

pub fn process_file(
    path: &Path,
    config: &Config,
    output_format: OutputFormat,
) -> Result<FileResult, String> {
    let file_path = path.to_string_lossy().to_string();
    let source = fs::read_to_string(path).map_err(|e| format!("failed to read file: {e}"))?;
    process_source(&source, &file_path, config, output_format)
}

pub fn discover_files(start_path: &Path, exclude_set: &GlobSet) -> Vec<PathBuf> {
    WalkBuilder::new(start_path)
        .standard_filters(false)
        .hidden(false)
        .filter_entry({
            let start = start_path.to_path_buf();
            let set = exclude_set.clone();
            move |entry| {
                let rel = entry.path().strip_prefix(&start).unwrap_or(entry.path());
                let file_path = Candidate::new(rel);
                let file_basename = rel.file_name().map(Candidate::new);
                !set.is_match_candidate(&file_path)
                    && !file_basename.is_some_and(|b| set.is_match_candidate(&b))
            }
        })
        .build()
        .filter_map(|e| e.ok())
        .filter(|e| {
            e.file_type().is_some_and(|ft| ft.is_file())
                && e.path().extension().is_some_and(|ext| ext == "py")
        })
        .map(|e| e.into_path())
        .collect()
}

pub fn run(files: &[PathBuf], config: &Config, output_format: OutputFormat) -> LintResults {
    let mut succeeded = Vec::new();
    let mut failed = Vec::new();

    for result in files
        .par_iter()
        .map(|file| {
            let file_path = file.to_string_lossy().to_string();
            match process_file(file, config, output_format) {
                Ok(res) => (file_path, Ok(res)),
                Err(msg) => (file_path, Err(msg)),
            }
        })
        .collect::<Vec<_>>()
    {
        match result {
            (_, Ok(res)) => succeeded.push(res),
            (file_path, Err(msg)) => failed.push((file_path, msg)),
        }
    }

    LintResults { succeeded, failed }
}

pub fn write_outputs_and_accumulate(
    results: &[FileResult],
    stdout: &mut impl io::Write,
) -> io::Result<(usize, usize, usize)> {
    let mut total_errors = 0;
    let mut total_warnings = 0;
    let mut total_statements = 0;

    for res in results {
        stdout.write_all(&res.buf)?;
        total_errors += res.errors;
        total_warnings += res.warnings;
        total_statements += res.statements;
    }

    Ok((total_errors, total_warnings, total_statements))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config;
    use crate::display::OutputFormat;
    use tempfile::TempDir;

    fn init() {
        colored::control::set_override(false);
    }

    const IMPORT: &str = "import structlog\n";

    #[test]
    fn process_source_finds_violations() {
        init();
        let source = format!("{IMPORT}log.info('user_logged_in', 'extra')\n");
        let config = Config::default();
        let result = process_source(&source, "test.py", &config, OutputFormat::Concise).unwrap();

        assert!(result.errors > 0, "should have errors");
        assert!(result.statements > 0, "should have statements");
        let output = String::from_utf8(result.buf).unwrap();
        assert!(
            output.contains("SL001"),
            "should report SL001, got: {output}"
        );
        assert!(output.contains("test.py:"), "should contain file path");
    }

    #[test]
    fn process_source_concise_format() {
        init();
        let source = format!("{IMPORT}log.info('user_logged_in', 'extra')\n");
        let config = Config::default();
        let result = process_source(&source, "test.py", &config, OutputFormat::Concise).unwrap();

        let output = String::from_utf8(result.buf).unwrap();
        for line in output.lines() {
            assert!(
                line.starts_with("test.py:"),
                "concise output should have file path prefix, got: {line}"
            );
        }
    }

    #[test]
    fn process_source_full_format_contains_source_context() {
        init();
        let source = format!("{IMPORT}log.info('user_logged_in', 'extra')\n");
        let config = Config::default();
        let result = process_source(&source, "test.py", &config, OutputFormat::Full).unwrap();

        let output = String::from_utf8(result.buf).unwrap();
        assert!(output.contains("|"), "full format should have gutter pipes");
        assert!(
            output.contains("log.info"),
            "full format should show source line"
        );
    }

    #[test]
    fn process_source_clean_code_has_no_violations() {
        init();
        let source = format!("{IMPORT}log.info('user_logged_in', user_id='u_123')\n");
        let config = Config::default();
        let result = process_source(&source, "test.py", &config, OutputFormat::Concise).unwrap();

        assert_eq!(result.errors, 0);
        assert_eq!(result.warnings, 0);
        assert!(result.buf.is_empty(), "clean code should have no output");
    }

    #[test]
    fn process_source_parse_error_returns_err() {
        init();
        let source = "this is not valid python {{{";
        let config = Config::default();
        let result = process_source(source, "test.py", &config, OutputFormat::Concise);

        assert!(result.is_err(), "invalid Python should return error");
        assert!(
            result.unwrap_err().contains("parse"),
            "error should mention parse"
        );
    }

    #[test]
    fn process_source_no_structlog_import_skips_file() {
        init();
        let source = "log.info('user_logged_in', 'extra')\n";
        let config = Config::default();
        let result = process_source(source, "test.py", &config, OutputFormat::Concise).unwrap();

        assert_eq!(result.errors, 0);
        assert_eq!(result.warnings, 0);
        assert_eq!(result.statements, 0);
    }

    #[test]
    fn process_source_respects_select() {
        init();
        let source = format!("{IMPORT}log.info('user_logged_in', 'extra')\n");
        let mut config = Config::default();
        config.select = Some(vec!["SL001".to_string()]);
        let result = process_source(&source, "test.py", &config, OutputFormat::Concise).unwrap();

        let output = String::from_utf8(result.buf).unwrap();
        assert!(output.contains("SL001"), "SL001 should be reported");
        assert!(
            !output.contains("SL002"),
            "SL002 should not be reported when only SL001 is selected"
        );
    }

    #[test]
    fn process_file_reads_and_analyzes() {
        init();
        let dir = TempDir::new().unwrap();
        let file_path = dir.path().join("app.py");
        let source = format!("{IMPORT}log.info('user_logged_in', 'extra')\n");
        fs::write(&file_path, &source).unwrap();

        let config = Config::default();
        let result = process_file(&file_path, &config, OutputFormat::Concise).unwrap();

        assert!(result.errors > 0);
        let output = String::from_utf8(result.buf).unwrap();
        assert!(output.contains("SL001"));
    }

    #[test]
    fn process_file_nonexistent_returns_err() {
        init();
        let path = Path::new("/nonexistent/path/file.py");
        let config = Config::default();
        let result = process_file(path, &config, OutputFormat::Concise);

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("read"));
    }

    #[test]
    fn discover_files_finds_python_files() {
        let dir = TempDir::new().unwrap();
        fs::write(dir.path().join("app.py"), "x=1\n").unwrap();
        fs::write(dir.path().join("lib.py"), "y=2\n").unwrap();
        fs::write(dir.path().join("readme.md"), "# readme\n").unwrap();

        let exclude_set = GlobSet::empty();
        let files = discover_files(dir.path(), &exclude_set);

        let names: Vec<_> = files
            .iter()
            .map(|p| p.file_name().unwrap().to_string_lossy().to_string())
            .collect();
        assert!(names.contains(&"app.py".to_string()));
        assert!(names.contains(&"lib.py".to_string()));
        assert!(!names.contains(&"readme.md".to_string()));
    }

    #[test]
    fn discover_files_respects_excludes() {
        let dir = TempDir::new().unwrap();
        fs::create_dir_all(dir.path().join("src")).unwrap();
        fs::create_dir_all(dir.path().join(".venv")).unwrap();
        fs::write(dir.path().join("src/app.py"), "x=1\n").unwrap();
        fs::write(dir.path().join(".venv/lib.py"), "y=2\n").unwrap();

        let config = Config::default();
        let exclude_set = config.build_exclude_globset().unwrap();
        let files = discover_files(dir.path(), &exclude_set);

        let paths: Vec<String> = files
            .iter()
            .map(|p| {
                p.strip_prefix(dir.path())
                    .unwrap()
                    .to_string_lossy()
                    .to_string()
            })
            .collect();
        assert!(
            paths.iter().any(|p| p.contains("src/app.py")),
            "src/app.py should be included"
        );
        assert!(
            !paths.iter().any(|p| p.contains(".venv/")),
            ".venv should be excluded"
        );
    }

    #[test]
    fn run_parallel_processes_multiple_files() {
        init();
        let dir = TempDir::new().unwrap();
        let source = format!("{IMPORT}log.info('user_logged_in', 'extra')\n");
        fs::write(dir.path().join("a.py"), &source).unwrap();
        fs::write(dir.path().join("b.py"), &source).unwrap();

        let files = vec![dir.path().join("a.py"), dir.path().join("b.py")];
        let config = Config::default();
        let results = run(&files, &config, OutputFormat::Concise);

        assert_eq!(results.succeeded.len(), 2, "both files should succeed");
        assert!(results.failed.is_empty(), "no files should fail");
        for res in &results.succeeded {
            assert!(res.errors > 0, "each file should have errors");
            let output = String::from_utf8(res.buf.clone()).unwrap();
            assert!(output.contains("SL001"));
        }
    }

    #[test]
    fn run_parallel_handles_parse_errors() {
        init();
        let dir = TempDir::new().unwrap();
        let valid = format!("{IMPORT}log.info('ok', user_id='1')\n");
        let invalid = "not python {{{";
        fs::write(dir.path().join("valid.py"), &valid).unwrap();
        fs::write(dir.path().join("invalid.py"), invalid).unwrap();

        let files = vec![dir.path().join("valid.py"), dir.path().join("invalid.py")];
        let config = Config::default();
        let results = run(&files, &config, OutputFormat::Concise);

        assert_eq!(results.succeeded.len(), 1, "one file should succeed");
        assert_eq!(results.failed.len(), 1, "one file should fail");
        assert!(
            results.failed[0].1.contains("parse"),
            "should report parse error"
        );
    }

    #[test]
    fn run_parallel_handles_missing_files() {
        init();
        let dir = TempDir::new().unwrap();
        let source = format!("{IMPORT}log.info('ok', user_id='1')\n");
        fs::write(dir.path().join("exists.py"), &source).unwrap();

        let files = vec![dir.path().join("exists.py"), dir.path().join("missing.py")];
        let config = Config::default();
        let results = run(&files, &config, OutputFormat::Concise);

        assert_eq!(results.succeeded.len(), 1, "one file should succeed");
        assert_eq!(results.failed.len(), 1, "one file should fail");
        assert!(
            results.failed[0].1.contains("read"),
            "should report read error"
        );
    }

    #[test]
    fn run_parallel_empty_input() {
        init();
        let config = Config::default();
        let results = run(&[], &config, OutputFormat::Concise);

        assert!(results.succeeded.is_empty());
        assert!(results.failed.is_empty());
    }

    #[test]
    fn write_outputs_accumulates_counts() {
        init();
        let source = format!("{IMPORT}log.info('user_logged_in', 'extra')\n");
        let config = Config::default();
        let a = process_source(&source, "a.py", &config, OutputFormat::Concise).unwrap();
        let b = process_source(&source, "b.py", &config, OutputFormat::Concise).unwrap();
        let results = vec![a, b];

        let mut buf = Vec::new();
        let (errors, _warnings, statements) =
            write_outputs_and_accumulate(&results, &mut buf).unwrap();

        assert!(errors >= 2, "should accumulate errors from both files");
        assert!(
            statements >= 2,
            "should accumulate statements from both files"
        );
        let output = String::from_utf8(buf).unwrap();
        assert!(output.contains("a.py:"), "should contain output from a.py");
        assert!(output.contains("b.py:"), "should contain output from b.py");
    }
}
