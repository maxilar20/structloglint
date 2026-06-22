use std::io;
use std::process;

use clap::Parser;
use structloglint::config;
use structloglint::display::OutputFormat;
use structloglint::models::LogLevel;
use structloglint::rules::case_style::CaseStyle;
use structloglint::{display, runner};

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

fn apply_args_overrides(args: &Args, config: &mut config::Config) -> Result<(), String> {
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
        config.select = args.select.clone();
    }
    if args.ignore.is_some() {
        config.ignore = args.ignore.clone();
    }
    if args.exclude.is_some() {
        config.exclude = args.exclude.clone();
    }
    if args.extend_exclude.is_some() {
        config.extend_exclude = args.extend_exclude.clone();
    }
    if args.check_imports {
        config.check_imports = true;
    } else if args.no_check_imports {
        config.check_imports = false;
    }
    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    let mut config = config::discover_config(std::path::Path::new(&args.path))?;

    apply_args_overrides(&args, &mut config).map_err(Box::<dyn std::error::Error>::from)?;

    let start_path = std::path::PathBuf::from(&args.path);
    let exclude_set = config.build_exclude_globset()?;
    let files = runner::discover_files(&start_path, &exclude_set);

    let results = runner::run(&files, &config, args.output_format);

    let mut stdout = io::stdout().lock();
    let (total_errors, total_warnings, total_statements) =
        runner::write_outputs_and_accumulate(&results.succeeded, &mut stdout)?;

    for (file_path, msg) in &results.failed {
        eprintln!("structloglint: {file_path}: {msg}");
    }

    display::print_summary(
        &mut io::stdout().lock(),
        total_errors,
        total_warnings,
        total_statements,
    )?;

    if total_errors > 0 || total_warnings > 0 {
        process::exit(1);
    }

    Ok(())
}
