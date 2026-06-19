use crate::models::Finding;

/// Print findings to stdout with source code context.
pub fn print_findings(findings: &[Finding], source_code: &str) {
    for finding in findings {
        let line = &source_code[finding.statement.range.clone()];
        // dbg!(&finding.statement);
        println!("\n-> {}\n  {}", line, finding);
    }
}
