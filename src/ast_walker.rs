use rustpython_parser::ast::{self, Stmt};

use crate::models::{LogCall, LogLevel, ParentContext};

pub fn collect_log_calls(stmt: &Stmt, parent: ParentContext) -> Vec<LogCall<'_>> {
    match stmt {
        Stmt::Expr(expr_stmt) => extract_log_call(expr_stmt, parent),
        Stmt::If(if_stmt) => walk_if(if_stmt),
        Stmt::For(for_stmt) => walk_for(for_stmt),
        Stmt::AsyncFor(for_stmt) => walk_async_for(for_stmt),
        Stmt::While(while_stmt) => walk_while(while_stmt),
        Stmt::Try(try_stmt) => walk_try(try_stmt),
        Stmt::TryStar(try_stmt) => walk_try_star(try_stmt),
        Stmt::With(with_stmt) => walk_with(with_stmt),
        Stmt::AsyncWith(with_stmt) => walk_async_with(with_stmt),
        Stmt::FunctionDef(func) => walk_function(func),
        Stmt::AsyncFunctionDef(func) => walk_async_function(func),
        Stmt::ClassDef(class_def) => walk_class(class_def),
        Stmt::Match(match_stmt) => walk_match(match_stmt),
        _ => vec![],
    }
}

fn walk_body<'a>(body: &'a [Stmt], context: ParentContext) -> Vec<LogCall<'a>> {
    body.iter()
        .flat_map(|s| collect_log_calls(s, context))
        .collect()
}

fn extract_log_call<'a>(
    expr_stmt: &'a ast::StmtExpr,
    parent: ParentContext,
) -> Vec<LogCall<'a>> {
    let ast::Expr::Call(call) = expr_stmt.value.as_ref() else {
        return vec![];
    };
    let ast::Expr::Attribute(attr) = call.func.as_ref() else {
        return vec![];
    };
    let ast::Expr::Name(name) = attr.value.as_ref() else {
        return vec![];
    };
    if name.id.as_str() == "log" {
        if let Ok(level) = attr.attr.as_str().parse::<LogLevel>() {
            return vec![LogCall::new(call, parent, level)];
        }
    }
    vec![]
}

fn walk_if<'a>(if_stmt: &'a ast::StmtIf) -> Vec<LogCall<'a>> {
    let mut calls = walk_body(&if_stmt.body, ParentContext::If);
    calls.extend(walk_body(&if_stmt.orelse, ParentContext::Else));
    calls
}

fn walk_for<'a>(for_stmt: &'a ast::StmtFor) -> Vec<LogCall<'a>> {
    let mut calls = walk_body(&for_stmt.body, ParentContext::For);
    calls.extend(walk_body(&for_stmt.orelse, ParentContext::ForElse));
    calls
}

fn walk_async_for<'a>(for_stmt: &'a ast::StmtAsyncFor) -> Vec<LogCall<'a>> {
    let mut calls = walk_body(&for_stmt.body, ParentContext::AsyncFor);
    calls.extend(walk_body(&for_stmt.orelse, ParentContext::AsyncForElse));
    calls
}

fn walk_while<'a>(while_stmt: &'a ast::StmtWhile) -> Vec<LogCall<'a>> {
    let mut calls = walk_body(&while_stmt.body, ParentContext::While);
    calls.extend(walk_body(&while_stmt.orelse, ParentContext::WhileElse));
    calls
}

fn walk_try<'a>(try_stmt: &'a ast::StmtTry) -> Vec<LogCall<'a>> {
    let mut calls = walk_body(&try_stmt.body, ParentContext::Try);
    calls.extend(walk_except_handlers(&try_stmt.handlers));
    calls.extend(walk_body(&try_stmt.orelse, ParentContext::TryElse));
    calls.extend(walk_body(&try_stmt.finalbody, ParentContext::Finally));
    calls
}

fn walk_try_star<'a>(try_stmt: &'a ast::StmtTryStar) -> Vec<LogCall<'a>> {
    let mut calls = walk_body(&try_stmt.body, ParentContext::Try);
    calls.extend(walk_except_handlers(&try_stmt.handlers));
    calls.extend(walk_body(&try_stmt.orelse, ParentContext::TryElse));
    calls.extend(walk_body(&try_stmt.finalbody, ParentContext::Finally));
    calls
}

fn walk_except_handlers<'a>(handlers: &'a [ast::ExceptHandler]) -> Vec<LogCall<'a>> {
    handlers
        .iter()
        .flat_map(|handler| {
            let ast::ExceptHandler::ExceptHandler(h) = handler;
            walk_body(&h.body, ParentContext::Except)
        })
        .collect()
}

fn walk_with<'a>(with_stmt: &'a ast::StmtWith) -> Vec<LogCall<'a>> {
    walk_body(&with_stmt.body, ParentContext::With)
}

fn walk_async_with<'a>(with_stmt: &'a ast::StmtAsyncWith) -> Vec<LogCall<'a>> {
    walk_body(&with_stmt.body, ParentContext::AsyncWith)
}

fn walk_function<'a>(func: &'a ast::StmtFunctionDef) -> Vec<LogCall<'a>> {
    walk_body(&func.body, ParentContext::Function)
}

fn walk_async_function<'a>(func: &'a ast::StmtAsyncFunctionDef) -> Vec<LogCall<'a>> {
    walk_body(&func.body, ParentContext::AsyncFunction)
}

fn walk_class<'a>(class_def: &'a ast::StmtClassDef) -> Vec<LogCall<'a>> {
    walk_body(&class_def.body, ParentContext::Class)
}

fn walk_match<'a>(match_stmt: &'a ast::StmtMatch) -> Vec<LogCall<'a>> {
    match_stmt
        .cases
        .iter()
        .flat_map(|case| walk_body(&case.body, ParentContext::Match))
        .collect()
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

    fn find_log_calls_with_context(source: &str) -> Vec<ParentContext> {
        let stmts = Suite::parse(source, "<test>").expect("parse failed");
        stmts
            .iter()
            .flat_map(|s| collect_log_calls(s, ParentContext::Module))
            .map(|c| c.context)
            .collect()
    }

    // ── Detection ───────────────────────────────────────────────────

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

    // ── Block traversal (count) ─────────────────────────────────────

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
    fn inside_async_for() {
        let source = r#"import structlog
log = structlog.get_logger()
async def f():
    async for i in aiter():
        log.info("async_iter")
"#;
        assert_eq!(find_log_calls(source), 1);
    }

    #[test]
    fn inside_async_with() {
        let source = r#"import structlog
log = structlog.get_logger()
async def f():
    async with ctx():
        log.info("async_with")
"#;
        assert_eq!(find_log_calls(source), 1);
    }

    #[test]
    fn inside_async_function() {
        let source = r#"import structlog
log = structlog.get_logger()
async def f():
    log.info("async_fn")
"#;
        assert_eq!(find_log_calls(source), 1);
    }

    #[test]
    fn inside_match_case() {
        let source = r#"import structlog
log = structlog.get_logger()
match cmd:
    case "start":
        log.info("starting")
    case "stop":
        log.info("stopping")
"#;
        assert_eq!(find_log_calls(source), 2);
    }

    #[test]
    fn inside_try_else() {
        let source = r#"import structlog
log = structlog.get_logger()
try:
    pass
except:
    pass
else:
    log.info("no_error")
"#;
        assert_eq!(find_log_calls(source), 1);
    }

    #[test]
    fn inside_try_star() {
        let source = r#"import structlog
log = structlog.get_logger()
try:
    log.info("body")
except* ValueError:
    log.error("val_err")
except* TypeError:
    log.error("type_err")
"#;
        assert_eq!(find_log_calls(source), 3);
    }

    #[test]
    fn inside_while_orelse() {
        let source = r#"import structlog
log = structlog.get_logger()
while False:
    pass
else:
    log.info("done")
"#;
        assert_eq!(find_log_calls(source), 1);
    }

    #[test]
    fn inside_if_else() {
        let source = r#"import structlog
log = structlog.get_logger()
if False:
    pass
else:
    log.info("else_branch")
"#;
        assert_eq!(find_log_calls(source), 1);
    }

    #[test]
    fn inside_elif() {
        let source = r#"import structlog
log = structlog.get_logger()
if False:
    log.info("if_branch")
elif True:
    log.info("elif_branch")
else:
    log.info("else_branch")
"#;
        assert_eq!(find_log_calls(source), 3);
    }

    #[test]
    fn inside_async_for_orelse() {
        let source = r#"import structlog
log = structlog.get_logger()
async def f():
    async for i in aiter():
        pass
    else:
        log.info("async_for_else")
"#;
        assert_eq!(find_log_calls(source), 1);
    }

    // ── Context assignment ──────────────────────────────────────────

    #[test]
    fn context_module_level() {
        let source = "import structlog\nlog = structlog.get_logger()\nlog.info('hi')\n";
        assert_eq!(find_log_calls_with_context(source), vec![ParentContext::Module]);
    }

    #[test]
    fn context_if_block() {
        let source = r#"import structlog
log = structlog.get_logger()
if True:
    log.info("x")
"#;
        assert_eq!(find_log_calls_with_context(source), vec![ParentContext::If]);
    }

    #[test]
    fn context_else_block() {
        let source = r#"import structlog
log = structlog.get_logger()
if False:
    pass
else:
    log.info("x")
"#;
        assert_eq!(find_log_calls_with_context(source), vec![ParentContext::Else]);
    }

    #[test]
    fn context_elif_body_is_if() {
        let source = r#"import structlog
log = structlog.get_logger()
if False:
    pass
elif True:
    log.info("x")
"#;
        assert_eq!(find_log_calls_with_context(source), vec![ParentContext::If]);
    }

    #[test]
    fn context_elif_else_is_else() {
        let source = r#"import structlog
log = structlog.get_logger()
if False:
    pass
elif False:
    pass
else:
    log.info("x")
"#;
        assert_eq!(find_log_calls_with_context(source), vec![ParentContext::Else]);
    }

    #[test]
    fn context_for_loop() {
        let source = r#"import structlog
log = structlog.get_logger()
for i in []:
    log.info("x")
"#;
        assert_eq!(find_log_calls_with_context(source), vec![ParentContext::For]);
    }

    #[test]
    fn context_for_else() {
        let source = r#"import structlog
log = structlog.get_logger()
for i in []:
    pass
else:
    log.info("x")
"#;
        assert_eq!(find_log_calls_with_context(source), vec![ParentContext::ForElse]);
    }

    #[test]
    fn context_while_loop() {
        let source = r#"import structlog
log = structlog.get_logger()
while True:
    log.info("x")
"#;
        assert_eq!(find_log_calls_with_context(source), vec![ParentContext::While]);
    }

    #[test]
    fn context_while_else() {
        let source = r#"import structlog
log = structlog.get_logger()
while False:
    pass
else:
    log.info("x")
"#;
        assert_eq!(find_log_calls_with_context(source), vec![ParentContext::WhileElse]);
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
        assert_eq!(find_log_calls_with_context(source), vec![ParentContext::Except]);
    }

    #[test]
    fn context_try_body() {
        let source = r#"import structlog
log = structlog.get_logger()
try:
    log.info("x")
except:
    pass
"#;
        assert_eq!(find_log_calls_with_context(source), vec![ParentContext::Try]);
    }

    #[test]
    fn context_try_else() {
        let source = r#"import structlog
log = structlog.get_logger()
try:
    pass
except:
    pass
else:
    log.info("x")
"#;
        assert_eq!(find_log_calls_with_context(source), vec![ParentContext::TryElse]);
    }

    #[test]
    fn context_finally() {
        let source = r#"import structlog
log = structlog.get_logger()
try:
    pass
except:
    pass
finally:
    log.info("x")
"#;
        assert_eq!(find_log_calls_with_context(source), vec![ParentContext::Finally]);
    }

    #[test]
    fn context_function() {
        let source = r#"import structlog
log = structlog.get_logger()
def foo():
    log.info("x")
"#;
        assert_eq!(find_log_calls_with_context(source), vec![ParentContext::Function]);
    }

    #[test]
    fn context_async_function() {
        let source = r#"import structlog
log = structlog.get_logger()
async def foo():
    log.info("x")
"#;
        assert_eq!(find_log_calls_with_context(source), vec![ParentContext::AsyncFunction]);
    }

    #[test]
    fn context_with() {
        let source = r#"import structlog
log = structlog.get_logger()
with open(""):
    log.info("x")
"#;
        assert_eq!(find_log_calls_with_context(source), vec![ParentContext::With]);
    }

    #[test]
    fn context_async_with() {
        let source = r#"import structlog
log = structlog.get_logger()
async def f():
    async with ctx():
        log.info("x")
"#;
        assert_eq!(find_log_calls_with_context(source), vec![ParentContext::AsyncWith]);
    }

    #[test]
    fn context_class() {
        let source = r#"import structlog
log = structlog.get_logger()
class Foo:
    log.info("x")
"#;
        assert_eq!(find_log_calls_with_context(source), vec![ParentContext::Class]);
    }

    #[test]
    fn context_async_for() {
        let source = r#"import structlog
log = structlog.get_logger()
async def f():
    async for i in aiter():
        log.info("x")
"#;
        assert_eq!(find_log_calls_with_context(source), vec![ParentContext::AsyncFor]);
    }

    #[test]
    fn context_async_for_else() {
        let source = r#"import structlog
log = structlog.get_logger()
async def f():
    async for i in aiter():
        pass
    else:
        log.info("x")
"#;
        assert_eq!(find_log_calls_with_context(source), vec![ParentContext::AsyncForElse]);
    }

    #[test]
    fn context_match() {
        let source = r#"import structlog
log = structlog.get_logger()
match cmd:
    case "start":
        log.info("x")
"#;
        assert_eq!(find_log_calls_with_context(source), vec![ParentContext::Match]);
    }

    #[test]
    fn context_nested_retains_deepest() {
        let source = r#"import structlog
log = structlog.get_logger()
for i in []:
    if True:
        try:
            pass
        except:
            log.exception("inside except")
"#;
        assert_eq!(find_log_calls_with_context(source), vec![ParentContext::Except]);
    }

    #[test]
    fn context_full_try_all_branches() {
        let source = r#"import structlog
log = structlog.get_logger()
try:
    log.info("try_body")
except:
    log.error("except_body")
else:
    log.info("try_else")
finally:
    log.debug("finally_body")
"#;
        assert_eq!(
            find_log_calls_with_context(source),
            vec![
                ParentContext::Try,
                ParentContext::Except,
                ParentContext::TryElse,
                ParentContext::Finally,
            ]
        );
    }

    #[test]
    fn context_if_elif_else_all_branches() {
        let source = r#"import structlog
log = structlog.get_logger()
if True:
    log.info("if_body")
elif True:
    log.info("elif_body")
else:
    log.info("else_body")
"#;
        assert_eq!(
            find_log_calls_with_context(source),
            vec![ParentContext::If, ParentContext::If, ParentContext::Else]
        );
    }

    #[test]
    fn context_for_body_and_else() {
        let source = r#"import structlog
log = structlog.get_logger()
for i in []:
    log.info("body")
else:
    log.info("else")
"#;
        assert_eq!(
            find_log_calls_with_context(source),
            vec![ParentContext::For, ParentContext::ForElse]
        );
    }

    #[test]
    fn context_while_body_and_else() {
        let source = r#"import structlog
log = structlog.get_logger()
while True:
    log.info("body")
else:
    log.info("else")
"#;
        assert_eq!(
            find_log_calls_with_context(source),
            vec![ParentContext::While, ParentContext::WhileElse]
        );
    }
}
