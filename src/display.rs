use std::fmt;

use colored::Colorize;

use crate::models::{Finding, Status};

impl fmt::Display for Finding<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for result in &self.results {
            let icon = match result.status {
                Status::Pass => "OK".green(),
                Status::Warning => "WARN".yellow(),
                Status::Fail => "FAIL".red(),
            };
            write!(f, "{icon} {}  {}", result.rule_id, result.feedback)?;
        }
        Ok(())
    }
}

pub fn print_findings(findings: &[Finding], source_code: &str, verbose: bool) {
    for finding in findings {
        let stmt = finding.statement();
        let line = &source_code[stmt.range.clone()];
        if verbose {
            dbg!(stmt);
        }
        println!("\n-> {}\n  {}", line, finding);
    }
}
