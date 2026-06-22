use std::fs;
use std::io;
use std::process;

use clap::Parser;
use globset::Candidate;
use ignore::WalkBuilder;
use rustpython_parser::Parse;
use structloglint::config;
use structloglint::display::OutputFormat;
use structloglint::models::LogLevel;
use structloglint::rules::case_style::CaseStyle;
use structloglint::{analyzer, display};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(short, long, default_value = "./")]
    path: String,

    #[arg(short, long)]
    verbose: bool,

    #[arg(short = 'f', long, default_value = "full", value_enum)]
    output_format: OutputFormat,

    #[arg(long, value_name = "STYLE")]
    event_case_style: Option<String>,

    #[arg(long, value_name = "N")]
    max_event_length: Option<usize>,

    #[arg(long, value_name = "LEVEL")]
    loop_log_level: Option<String>,

    #[arg(long, value_name = "RULES", value_delimiter = ',')]
    select: Option<Vec<String>>,

    #[arg(long, value_name = "RULES", value_delimiter = ',')]
    ignore: Option<Vec<String>>,

    #[arg(long, value_name = "PATTERN", value_delimiter = ',')]
    exclude: Option<Vec<String>>,

    #[arg(long, value_name = "PATTERN", value_delimiter = ',')]
    extend_exclude: Option<Vec<String>>,

    #[arg(
        long = "check-imports",
        overrides_with = "no_check_imports",
        default_value_t = false
    )]
    check_imports: bool,

    #[arg(
        long = "no-check-imports",
        overrides_with = "check_imports",
        default_value_t = false
    )]
    no_check_imports: bool,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    let mut config = config::discover_config(std::path::Path::new(&args.path))?;

    if let Some(style) = &args.event_case_style {
        config.case_style = style
            .parse::<CaseStyle>()
            .map_err(|_| format!("invalid case style: {style}"))?;
    }
    if let Some(n) = args.max_event_length {
        config.max_event_length = n;
    }
    if let Some(level) = &args.loop_log_level {
        config.min_loop_log_level = level
            .parse::<LogLevel>()
            .map_err(|_| format!("invalid log level: {level}"))?;
    }
    if args.select.is_some() {
        config.select = args.select;
    }
    if args.ignore.is_some() {
        config.ignore = args.ignore;
    }
    if args.exclude.is_some() {
        config.exclude = args.exclude;
    }
    if args.extend_exclude.is_some() {
        config.extend_exclude = args.extend_exclude;
    }
    if args.check_imports {
        config.check_imports = true;
    } else if args.no_check_imports {
        config.check_imports = false;
    }
    let mut stdout = io::stdout().lock();
    let mut total_errors = 0usize;
    let mut total_warnings = 0usize;

    let start_path = std::path::PathBuf::from(&args.path);
    let exclude_set = config.build_exclude_globset()?;

    let files: Vec<_> = WalkBuilder::new(&start_path)
        .standard_filters(false)
        .hidden(false)
        .filter_entry(move |entry| {
            let rel = entry
                .path()
                .strip_prefix(&start_path)
                .unwrap_or(entry.path());
            let file_path = Candidate::new(rel);
            let file_basename = rel.file_name().map(Candidate::new);
            !exclude_set.is_match_candidate(&file_path)
                && !file_basename.is_some_and(|b| exclude_set.is_match_candidate(&b))
        })
        .build()
        .filter_map(|e| e.ok())
        .filter(|e| {
            e.file_type().is_some_and(|ft| ft.is_file())
                && e.path().extension().is_some_and(|ext| ext == "py")
        })
        .map(|e| e.into_path())
        .collect();

    for file in &files {
        let file_path = file.to_string_lossy().to_string();
        let python_code = fs::read_to_string(file)?;
        let stmts = rustpython_parser::ast::Suite::parse(&python_code, &file_path)?;
        let findings = analyzer::analyze(&stmts, &config);

        let (errors, warnings) = display::print_diagnostics(
            &mut stdout,
            &findings,
            &file_path,
            &python_code,
            args.output_format,
        )?;
        total_errors += errors;
        total_warnings += warnings;
    }

    display::print_summary(&mut stdout, total_errors, total_warnings)?;

    if total_errors > 0 || total_warnings > 0 {
        process::exit(1);
    }

    Ok(())
}
