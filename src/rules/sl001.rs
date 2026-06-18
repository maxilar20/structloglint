use rustpython_parser::ast;

use crate::models::RuleResult;

pub fn check_sl001(call: &ast::ExprCall) -> RuleResult {
    if call.args.len() > 1 {
        return RuleResult::new(
            "SL001",
            false,
            "Too many positional arguments. Only one positional argument should be provided."
                .to_string(),
        );
    }

    RuleResult::new("SL001", true, String::new())
}
