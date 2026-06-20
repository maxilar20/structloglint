use crate::models::Finding;

/// Print findings to stdout with source code context.
pub fn print_findings(findings: &[Finding], source_code: &str, verbose: bool) {
    for finding in findings {
        let line = &source_code[finding.statement.range.clone()];
        if verbose {
            dbg!(&finding.statement);
        }
        println!("\n-> {}\n  {}", line, finding);
    }
}
