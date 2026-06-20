use crate::models::Finding;

/// Print findings to stdout with source code context.
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
