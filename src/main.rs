use std::fs;

use clap::Parser;
use rustpython_parser::Parse;
use structlog_linter::{analyzer, display};
use walkdir::WalkDir;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(short, long, default_value = "./")]
    path: String,

    #[arg(short, long)]
    verbose: bool,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    let files: Vec<_> = WalkDir::new(&args.path)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| {
            e.file_type().is_file()
                && e.path().extension().is_some_and(|ext| ext == "py")
        })
        .collect();

    println!("\n-> Scanning {} ({} Python file(s))", args.path, files.len());

    for file in &files {
        let file_path = file.path().to_string_lossy().to_string();
        let python_code = fs::read_to_string(file.path())?;
        let stmts = rustpython_parser::ast::Suite::parse(&python_code, &file_path)?;
        let findings = analyzer::analyze(&stmts);

        if !findings.is_empty() || args.verbose {
            println!("\n  {} — {} finding(s)", file_path, findings.len());
            display::print_findings(&findings, &python_code, args.verbose);
        }
    }

    Ok(())
}
