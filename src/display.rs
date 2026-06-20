use std::io::{self, Write};

use colored::Colorize;

use crate::models::{Finding, RuleResult, Status};

#[derive(Debug, Clone, Copy, PartialEq, Eq, clap::ValueEnum)]
pub enum OutputFormat {
    Full,
    Concise,
}

pub struct LineIndex {
    line_starts: Vec<usize>,
}

impl LineIndex {
    pub fn new(source: &str) -> Self {
        let mut line_starts = vec![0];
        for (i, b) in source.bytes().enumerate() {
            if b == b'\n' {
                line_starts.push(i + 1);
            }
        }
        Self { line_starts }
    }

    pub fn line_col(&self, offset: usize) -> (usize, usize) {
        let line = self.line_starts.partition_point(|&start| start <= offset) - 1;
        let col = offset - self.line_starts[line];
        (line + 1, col + 1)
    }

    pub fn line_text<'a>(&self, line: usize, source: &'a str) -> &'a str {
        let start = self.line_starts[line - 1];
        let end = if line < self.line_starts.len() {
            self.line_starts[line] - 1
        } else {
            source.len()
        };
        source[start..end].trim_end_matches('\r')
    }

    pub fn line_start_offset(&self, line: usize) -> usize {
        self.line_starts[line - 1]
    }
}

pub fn print_diagnostics(
    w: &mut impl Write,
    findings: &[Finding],
    file_path: &str,
    source_code: &str,
    format: OutputFormat,
) -> io::Result<(usize, usize)> {
    let index = LineIndex::new(source_code);
    let mut errors = 0usize;
    let mut warnings = 0usize;

    for finding in findings {
        let call = finding.statement();
        let start_offset = u32::from(call.range.start()) as usize;
        let end_offset = u32::from(call.range.end()) as usize;
        let (line, col) = index.line_col(start_offset);

        for result in &finding.results {
            match result.status {
                Status::Pass => continue,
                Status::Fail => errors += 1,
                Status::Warning => warnings += 1,
            }

            write_header(w, file_path, line, col, result)?;

            if format == OutputFormat::Full {
                let source_line = index.line_text(line, source_code);
                let line_start = index.line_start_offset(line);
                let underline_start = col - 1;
                let call_end_in_line = end_offset - line_start;
                let underline_end = call_end_in_line.min(source_line.len());
                let underline_width = underline_end.saturating_sub(underline_start).max(1);

                write_source_context(w, line, source_line, underline_start, underline_width, result)?;
            }
        }
    }

    Ok((errors, warnings))
}

pub fn print_summary(w: &mut impl Write, errors: usize, warnings: usize) -> io::Result<()> {
    if errors == 0 && warnings == 0 {
        writeln!(w, "{}", "All checks passed!".green().bold())
    } else {
        let mut parts = Vec::new();
        if errors > 0 {
            parts.push(format!("{} error(s)", errors));
        }
        if warnings > 0 {
            parts.push(format!("{} warning(s)", warnings));
        }
        writeln!(w, "{}", format!("Found {}.", parts.join(", ")).bold())
    }
}

fn write_header(
    w: &mut impl Write,
    file_path: &str,
    line: usize,
    col: usize,
    result: &RuleResult,
) -> io::Result<()> {
    let rule = color_rule_id(result);
    writeln!(
        w,
        "{}:{}:{}: {} {}",
        file_path.bold().cyan(),
        line,
        col,
        rule,
        result.feedback,
    )
}

fn write_source_context(
    w: &mut impl Write,
    line_num: usize,
    source_line: &str,
    underline_start: usize,
    underline_width: usize,
    result: &RuleResult,
) -> io::Result<()> {
    let gutter_width = line_num.to_string().len();
    let pipe = "|".blue().bold();

    writeln!(w, "{:>gutter_width$} {pipe}", "")?;

    let line_str = format!("{line_num:>gutter_width$}");
    writeln!(w, "{} {pipe} {source_line}", line_str.blue().bold())?;

    let markers = "^".repeat(underline_width);
    let (colored_markers, colored_rule) = match result.status {
        Status::Fail => (markers.red().bold(), result.rule_id.red().bold()),
        Status::Warning => (markers.yellow().bold(), result.rule_id.yellow().bold()),
        Status::Pass => unreachable!(),
    };
    writeln!(
        w,
        "{:>gutter_width$} {pipe} {}{colored_markers} {colored_rule}",
        "",
        " ".repeat(underline_start),
    )?;

    writeln!(w, "{:>gutter_width$} {pipe}", "")
}

fn color_rule_id(result: &RuleResult) -> colored::ColoredString {
    match result.status {
        Status::Fail => result.rule_id.red().bold(),
        Status::Warning => result.rule_id.yellow().bold(),
        Status::Pass => result.rule_id.normal(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::analyzer;
    use rustpython_parser::Parse;
    use rustpython_parser::ast::Suite;

    const IMPORT: &str = "import structlog\n";

    fn init() {
        colored::control::set_override(false);
    }

    fn src(code: &str) -> String {
        format!("{IMPORT}{code}")
    }

    #[test]
    fn line_index_single_line() {
        let idx = LineIndex::new("hello");
        assert_eq!(idx.line_col(0), (1, 1));
        assert_eq!(idx.line_col(4), (1, 5));
        assert_eq!(idx.line_text(1, "hello"), "hello");
    }

    #[test]
    fn line_index_multi_line() {
        let source = "abc\ndef\nghi\n";
        let idx = LineIndex::new(source);
        assert_eq!(idx.line_col(0), (1, 1));
        assert_eq!(idx.line_col(2), (1, 3));
        assert_eq!(idx.line_col(4), (2, 1));
        assert_eq!(idx.line_col(6), (2, 3));
        assert_eq!(idx.line_col(8), (3, 1));
        assert_eq!(idx.line_text(1, source), "abc");
        assert_eq!(idx.line_text(2, source), "def");
        assert_eq!(idx.line_text(3, source), "ghi");
    }

    #[test]
    fn line_index_crlf() {
        let source = "abc\r\ndef\r\n";
        let idx = LineIndex::new(source);
        assert_eq!(idx.line_col(0), (1, 1));
        assert_eq!(idx.line_col(5), (2, 1));
        assert_eq!(idx.line_text(1, source), "abc");
        assert_eq!(idx.line_text(2, source), "def");
    }

    #[test]
    fn line_index_no_trailing_newline() {
        let source = "abc\ndef";
        let idx = LineIndex::new(source);
        assert_eq!(idx.line_text(1, source), "abc");
        assert_eq!(idx.line_text(2, source), "def");
    }

    #[test]
    fn line_index_empty_lines() {
        let source = "a\n\nb\n";
        let idx = LineIndex::new(source);
        assert_eq!(idx.line_col(0), (1, 1));
        assert_eq!(idx.line_col(2), (2, 1));
        assert_eq!(idx.line_col(3), (3, 1));
        assert_eq!(idx.line_text(1, source), "a");
        assert_eq!(idx.line_text(2, source), "");
        assert_eq!(idx.line_text(3, source), "b");
    }

    #[test]
    fn line_index_start_offset() {
        let source = "abc\ndef\nghi\n";
        let idx = LineIndex::new(source);
        assert_eq!(idx.line_start_offset(1), 0);
        assert_eq!(idx.line_start_offset(2), 4);
        assert_eq!(idx.line_start_offset(3), 8);
    }

    #[test]
    fn concise_format_shows_only_failures() {
        init();
        let source = src("log.info('user_logged_in', 'extra')\n");
        let stmts = Suite::parse(&source, "<test>").expect("parse failed");
        let findings = analyzer::analyze(&stmts);

        let mut buf = Vec::new();
        let (errors, warnings) = print_diagnostics(
            &mut buf,
            &findings,
            "test.py",
            &source,
            OutputFormat::Concise,
        )
        .unwrap();

        let output = String::from_utf8(buf).unwrap();
        assert!(!output.contains("OK"));
        assert!(output.contains("SL001"));
        assert!(output.contains("test.py:2:1:"));
        assert!(errors > 0 || warnings > 0);
    }

    #[test]
    fn concise_format_each_line_has_file_path() {
        init();
        let source = src("log.info('user_logged_in', 'extra')\n");
        let stmts = Suite::parse(&source, "<test>").expect("parse failed");
        let findings = analyzer::analyze(&stmts);

        let mut buf = Vec::new();
        print_diagnostics(
            &mut buf,
            &findings,
            "test.py",
            &source,
            OutputFormat::Concise,
        )
        .unwrap();

        let output = String::from_utf8(buf).unwrap();
        for line in output.lines() {
            assert!(
                line.starts_with("test.py:"),
                "each line should start with file path, got: {line}"
            );
        }
    }

    #[test]
    fn full_format_shows_source_context() {
        init();
        let source = src("log.info('user_logged_in', 'extra')\n");
        let stmts = Suite::parse(&source, "<test>").expect("parse failed");
        let findings = analyzer::analyze(&stmts);

        let mut buf = Vec::new();
        print_diagnostics(
            &mut buf,
            &findings,
            "test.py",
            &source,
            OutputFormat::Full,
        )
        .unwrap();

        let output = String::from_utf8(buf).unwrap();
        assert!(output.contains("|"), "full format should contain gutter pipes");
        assert!(output.contains("^^^"), "full format should contain underline markers");
        assert!(
            output.contains("log.info"),
            "full format should show source line"
        );
    }

    #[test]
    fn pass_results_are_filtered() {
        init();
        let source = src("log.info('user_logged_in', user_id='u_123')\n");
        let stmts = Suite::parse(&source, "<test>").expect("parse failed");
        let findings = analyzer::analyze(&stmts);

        let mut buf = Vec::new();
        let (errors, warnings) = print_diagnostics(
            &mut buf,
            &findings,
            "test.py",
            &source,
            OutputFormat::Concise,
        )
        .unwrap();

        let output = String::from_utf8(buf).unwrap();
        assert_eq!(errors, 0);
        assert_eq!(warnings, 0);
        assert!(output.is_empty(), "no output for passing results");
    }

    #[test]
    fn counts_errors_and_warnings() {
        init();
        let source = src("log.info('user_logged_in', 'extra')\n");
        let stmts = Suite::parse(&source, "<test>").expect("parse failed");
        let findings = analyzer::analyze(&stmts);

        let mut buf = Vec::new();
        let (errors, _warnings) = print_diagnostics(
            &mut buf,
            &findings,
            "test.py",
            &source,
            OutputFormat::Concise,
        )
        .unwrap();

        assert!(errors > 0, "should count at least one error");
    }

    #[test]
    fn summary_all_passed() {
        init();
        let mut buf = Vec::new();
        print_summary(&mut buf, 0, 0).unwrap();
        let output = String::from_utf8(buf).unwrap();
        assert!(output.contains("All checks passed!"));
    }

    #[test]
    fn summary_errors_only() {
        init();
        let mut buf = Vec::new();
        print_summary(&mut buf, 3, 0).unwrap();
        let output = String::from_utf8(buf).unwrap();
        assert!(output.contains("3 error(s)"));
        assert!(!output.contains("warning"));
    }

    #[test]
    fn summary_warnings_only() {
        init();
        let mut buf = Vec::new();
        print_summary(&mut buf, 0, 2).unwrap();
        let output = String::from_utf8(buf).unwrap();
        assert!(output.contains("2 warning(s)"));
        assert!(!output.contains("error"));
    }

    #[test]
    fn summary_errors_and_warnings() {
        init();
        let mut buf = Vec::new();
        print_summary(&mut buf, 5, 3).unwrap();
        let output = String::from_utf8(buf).unwrap();
        assert!(output.contains("5 error(s)"));
        assert!(output.contains("3 warning(s)"));
    }

    #[test]
    fn full_format_gutter_alignment() {
        init();
        let source = src("log.info('user_logged_in', 'extra')\n");
        let stmts = Suite::parse(&source, "<test>").expect("parse failed");
        let findings = analyzer::analyze(&stmts);

        let mut buf = Vec::new();
        print_diagnostics(
            &mut buf,
            &findings,
            "test.py",
            &source,
            OutputFormat::Full,
        )
        .unwrap();

        let output = String::from_utf8(buf).unwrap();
        let lines: Vec<&str> = output.lines().collect();
        let pipe_lines: Vec<&&str> = lines.iter().filter(|l| l.contains('|')).collect();
        assert!(
            pipe_lines.len() >= 3,
            "full format should have at least 3 gutter lines per diagnostic"
        );
    }

    #[test]
    fn multiline_call_underlines_first_line() {
        init();
        let source = src("log.info(\n    'user_logged_in',\n    'extra'\n)\n");
        let stmts = Suite::parse(&source, "<test>").expect("parse failed");
        let findings = analyzer::analyze(&stmts);

        let mut buf = Vec::new();
        print_diagnostics(
            &mut buf,
            &findings,
            "test.py",
            &source,
            OutputFormat::Full,
        )
        .unwrap();

        let output = String::from_utf8(buf).unwrap();
        assert!(output.contains("test.py:2:1:"));
        assert!(output.contains("log.info("));
    }

    #[test]
    fn multiple_findings_in_one_file() {
        init();
        let source = src("log.info('a', 'extra')\nlog.info('b', 'extra')\n");
        let stmts = Suite::parse(&source, "<test>").expect("parse failed");
        let findings = analyzer::analyze(&stmts);

        let mut buf = Vec::new();
        let (errors, _) = print_diagnostics(
            &mut buf,
            &findings,
            "test.py",
            &source,
            OutputFormat::Concise,
        )
        .unwrap();

        let output = String::from_utf8(buf).unwrap();
        assert!(errors >= 2, "should have at least 2 errors");
        assert!(output.contains("test.py:2:"));
        assert!(output.contains("test.py:3:"));
    }
}
