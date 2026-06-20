use std::fs;

use clap::Parser;
use structlog_linter::{analyzer, display};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    file: String,

    #[arg(short, long)]
    verbose: bool,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    println!("\n-> Reading {}", args.file);
    let python_code = fs::read_to_string(&args.file)?;

    if args.verbose {
        println!("------------------ START ------------------");
        println!("{}", &python_code[..python_code.len().min(200)]);
        println!("------------------- END -------------------");
    }

    let findings = analyzer::analyze(&python_code, &args.file)?;
    println!("Found {} calls", findings.len());

    display::print_findings(&findings, &python_code, args.verbose);

    Ok(())
}
