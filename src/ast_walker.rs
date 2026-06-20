use rustpython_parser::ast::{self, Stmt};

use crate::models::{LogCall, ParentContext};

/// Collect all `log.<level>(...)` call expressions along with their parent context.
///
/// The `parent` parameter indicates the immediate enclosing block type.
/// When recursing into nested bodies, the caller passes the appropriate context.
pub fn collect_log_calls(stmt: &Stmt, parent: ParentContext) -> Vec<LogCall<'_>> {
    match stmt {
        Stmt::Expr(expr_stmt) => match expr_stmt.value.as_ref() {
            ast::Expr::Call(call) => match call.func.as_ref() {
                ast::Expr::Attribute(attr) => match attr.value.as_ref() {
                    ast::Expr::Name(name)
                        if name.id.as_str() == "log"
                            && matches!(
                                attr.attr.as_str(),
                                "debug" | "info" | "warning" | "error" | "exception" | "critical"
                            ) =>
                    {
                        vec![LogCall::new(call, parent)]
                    }
                    _ => vec![],
                },
                _ => vec![],
            },
            _ => vec![],
        },
        Stmt::If(if_stmt) => {
            let mut calls: Vec<LogCall> = if_stmt
                .body
                .iter()
                .flat_map(|s| collect_log_calls(s, ParentContext::If))
                .collect();
            calls.extend(
                if_stmt
                    .orelse
                    .iter()
                    .flat_map(|s| collect_log_calls(s, parent)),
            );
            calls
        }
        Stmt::FunctionDef(func) => func
            .body
            .iter()
            .flat_map(|s| collect_log_calls(s, ParentContext::Function))
            .collect(),
        Stmt::For(for_stmt) => for_stmt
            .body
            .iter()
            .flat_map(|s| collect_log_calls(s, ParentContext::For))
            .chain(
                for_stmt
                    .orelse
                    .iter()
                    .flat_map(|s| collect_log_calls(s, parent)),
            )
            .collect(),
        Stmt::While(while_stmt) => while_stmt
            .body
            .iter()
            .flat_map(|s| collect_log_calls(s, ParentContext::While))
            .chain(
                while_stmt
                    .orelse
                    .iter()
                    .flat_map(|s| collect_log_calls(s, parent)),
            )
            .collect(),
        Stmt::Try(try_stmt) => {
            let mut calls: Vec<LogCall> = vec![];
            calls.extend(
                try_stmt
                    .body
                    .iter()
                    .flat_map(|s| collect_log_calls(s, parent)),
            );
            calls.extend(
                try_stmt
                    .finalbody
                    .iter()
                    .flat_map(|s| collect_log_calls(s, parent)),
            );
            for handler in &try_stmt.handlers {
                let ast::ExceptHandler::ExceptHandler(h) = handler;
                calls.extend(
                    h.body
                        .iter()
                        .flat_map(|s| collect_log_calls(s, ParentContext::Except)),
                );
            }
            calls
        }
        Stmt::With(with_stmt) => with_stmt
            .body
            .iter()
            .flat_map(|s| collect_log_calls(s, ParentContext::With))
            .collect(),
        Stmt::ClassDef(class_def) => class_def
            .body
            .iter()
            .flat_map(|s| collect_log_calls(s, ParentContext::Class))
            .collect(),
        _ => vec![],
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rustpython_parser::Parse;
    use rustpython_parser::ast::Suite;

    fn find_log_calls(source: &str) -> usize {
        let stmts = Suite::parse(source, "<test>").expect("parse failed");
        stmts
            .iter()
            .flat_map(|s| collect_log_calls(s, ParentContext::Module))
            .count()
    }

    #[test]
    fn direct_log_call() {
        let source =
            "import structlog\nlog = structlog.get_logger()\nlog.info('payment_complete')\n";
        assert_eq!(find_log_calls(source), 1);
    }

    #[test]
    fn direct_log_call_multiple_levels() {
        let source = r#"import structlog
log = structlog.get_logger()
log.info("a")
log.debug("b")
log.warning("c")
log.error("d")
log.exception("e")
log.critical("f")
"#;
        assert_eq!(find_log_calls(source), 6);
    }

    #[test]
    fn ignore_non_structlog_print() {
        let source = "print('hello')\n";
        assert_eq!(find_log_calls(source), 0);
    }

    #[test]
    fn ignore_non_log_attribute() {
        let source = "obj = object()\nobj.attr()\n";
        assert_eq!(find_log_calls(source), 0);
    }

    #[test]
    fn inside_if_statement() {
        let source = r#"import structlog
log = structlog.get_logger()
if True:
    log.info("a")
"#;
        assert_eq!(find_log_calls(source), 1);
    }

    #[test]
    fn inside_nested_if() {
        let source = r#"import structlog
log = structlog.get_logger()
if True:
    if True:
        log.info("nested")
"#;
        assert_eq!(find_log_calls(source), 1);
    }

    #[test]
    fn inside_function_def() {
        let source = r#"import structlog
log = structlog.get_logger()
def foo():
    log.info("bar")
"#;
        assert_eq!(find_log_calls(source), 1);
    }

    #[test]
    fn inside_for_loop() {
        let source = r#"import structlog
log = structlog.get_logger()
for i in []:
    log.debug("iter")
"#;
        assert_eq!(find_log_calls(source), 1);
    }

    #[test]
    fn inside_for_orelse() {
        let source = r#"import structlog
log = structlog.get_logger()
for i in []:
    var = 1+1
else:
    log.info("else")
"#;
        assert_eq!(find_log_calls(source), 1);
    }

    #[test]
    fn inside_while_loop() {
        let source = r#"import structlog
log = structlog.get_logger()
while False:
    log.warning("never")
"#;
        assert_eq!(find_log_calls(source), 1);
    }

    #[test]
    fn inside_try_and_except_and_finally() {
        let source = r#"import structlog
log = structlog.get_logger()
try:
    log.info("a")
except:
    log.error("b")
finally:
    log.debug("c")
"#;
        assert_eq!(find_log_calls(source), 3);
    }

    #[test]
    fn inside_with_statement() {
        let source = r#"import structlog
log = structlog.get_logger()
with open(""):
    log.info("inside")
"#;
        assert_eq!(find_log_calls(source), 1);
    }

    #[test]
    fn inside_class_method() {
        let source = r#"import structlog
log = structlog.get_logger()
class Foo:
    def method(self):
        log.info("inside")
"#;
        assert_eq!(find_log_calls(source), 1);
    }

    #[test]
    fn deep_nesting() {
        let source = r#"import structlog
log = structlog.get_logger()
def f():
    for i in []:
        if True:
            try:
                log.info("a")
            except:
                log.error("b")
            finally:
                log.debug("c")
"#;
        assert_eq!(find_log_calls(source), 3);
    }

    #[test]
    fn context_except_block() {
        let source = r#"import structlog
log = structlog.get_logger()
try:
    pass
except:
    log.exception("boom")
"#;
        let stmts = Suite::parse(source, "<test>").expect("parse failed");
        let calls: Vec<LogCall> = stmts
            .iter()
            .flat_map(|s| collect_log_calls(s, ParentContext::Module))
            .collect();
        assert_eq!(calls.len(), 1);
        assert_eq!(calls[0].context, ParentContext::Except);
    }

    #[test]
    fn context_for_loop() {
        let source = r#"import structlog
log = structlog.get_logger()
for i in []:
    log.info("x")
"#;
        let stmts = Suite::parse(source, "<test>").expect("parse failed");
        let calls: Vec<LogCall> = stmts
            .iter()
            .flat_map(|s| collect_log_calls(s, ParentContext::Module))
            .collect();
        assert_eq!(calls.len(), 1);
        assert_eq!(calls[0].context, ParentContext::For);
    }

    #[test]
    fn context_while_loop() {
        let source = r#"import structlog
log = structlog.get_logger()
while True:
    log.info("x")
"#;
        let stmts = Suite::parse(source, "<test>").expect("parse failed");
        let calls: Vec<LogCall> = stmts
            .iter()
            .flat_map(|s| collect_log_calls(s, ParentContext::Module))
            .collect();
        assert_eq!(calls.len(), 1);
        assert_eq!(calls[0].context, ParentContext::While);
    }

    #[test]
    fn context_module_level() {
        let source = "import structlog\nlog = structlog.get_logger()\nlog.info('hi')\n";
        let stmts = Suite::parse(source, "<test>").expect("parse failed");
        let calls: Vec<LogCall> = stmts
            .iter()
            .flat_map(|s| collect_log_calls(s, ParentContext::Module))
            .collect();
        assert_eq!(calls.len(), 1);
        assert_eq!(calls[0].context, ParentContext::Module);
    }

    #[test]
    fn context_nested_retains_deepest() {
        // Inside for -> if -> try -> except, context should be Except (most specific).
        let source = r#"import structlog
log = structlog.get_logger()
for i in []:
    if True:
        try:
            pass
        except:
            log.exception("inside except")
"#;
        let stmts = Suite::parse(source, "<test>").expect("parse failed");
        let calls: Vec<LogCall> = stmts
            .iter()
            .flat_map(|s| collect_log_calls(s, ParentContext::Module))
            .collect();
        assert_eq!(calls.len(), 1);
        assert_eq!(calls[0].context, ParentContext::Except);
    }
}
