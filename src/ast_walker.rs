use rustpython_parser::ast::{self, ExprCall, Stmt};

/// Walk the AST recursively and collect all `log.<level>(...)` call expressions.
pub fn collect_log_calls(stmt: &Stmt) -> Vec<&ExprCall> {
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
                        vec![call]
                    }
                    _ => vec![],
                },
                _ => vec![],
            },
            _ => vec![],
        },
        Stmt::If(if_stmt) => if_stmt.body.iter().flat_map(collect_log_calls).collect(),
        Stmt::FunctionDef(func) => func.body.iter().flat_map(collect_log_calls).collect(),
        Stmt::For(for_stmt) => for_stmt.body.iter().flat_map(collect_log_calls).collect(),
        Stmt::While(while_stmt) => while_stmt.body.iter().flat_map(collect_log_calls).collect(),
        Stmt::Try(try_stmt) => {
            let mut calls = vec![];
            calls.extend(try_stmt.body.iter().flat_map(collect_log_calls));
            calls.extend(try_stmt.finalbody.iter().flat_map(collect_log_calls));
            for handler in &try_stmt.handlers {
                let ast::ExceptHandler::ExceptHandler(h) = handler;
                calls.extend(h.body.iter().flat_map(collect_log_calls));
            }
            calls
        }
        Stmt::With(with_stmt) => with_stmt.body.iter().flat_map(collect_log_calls).collect(),
        Stmt::ClassDef(class_def) => class_def.body.iter().flat_map(collect_log_calls).collect(),
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
        stmts.iter().flat_map(collect_log_calls).count()
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
}
