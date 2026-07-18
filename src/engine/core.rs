use super::analyzer;
use super::ast;
use super::interpreter;
use super::lexer;
use super::parser;
use std::collections::HashSet;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use std::sync::mpsc::{Receiver, SyncSender};

pub enum EngineMessage {
    Output(Vec<String>),
    InputRequest,
}

pub const BUILTIN_FUNCS: &[&str] = &[
    "print",
    "println",
    "sleep",
    "exit",
    "range",
    "input",
    "len",
    "max",
    "min",
    "exec",
    "onlinux",
    "onwindows",
    "isdown",
    "isup",
];

pub fn analyze_code(source: &str) -> (usize, HashSet<usize>, HashSet<String>) {
    let mut lexer = lexer::Lexer::new(source);
    let tokens = lexer.tokenize();

    let mut parser = parser::Parser::new(tokens);
    let ast = parser.parse();

    let mut errors_count = parser.errors.len();
    let mut error_lines = parser.error_lines.clone();

    let mut analyzer = analyzer::Analyzer::new();
    analyzer.analyze(&ast);

    errors_count += analyzer.errors.len();
    error_lines.extend(analyzer.error_lines);

    let mut funcs = HashSet::new();
    for f in BUILTIN_FUNCS {
        funcs.insert(f.to_string());
    }

    fn extract_funcs(stmts: &[ast::Stmt], funcs: &mut HashSet<String>) {
        for stmt in stmts {
            match stmt {
                ast::Stmt::Fn(name, _, body, _) => {
                    funcs.insert(name.clone());
                    extract_funcs(body, funcs);
                }
                ast::Stmt::If(_, then_b, elifs, else_b) => {
                    extract_funcs(then_b, funcs);
                    for (_, elif_b) in elifs {
                        extract_funcs(elif_b, funcs);
                    }
                    if let Some(e) = else_b {
                        extract_funcs(e, funcs);
                    }
                }
                ast::Stmt::Loop(b) | ast::Stmt::While(_, b) | ast::Stmt::For(_, _, b, _) => {
                    extract_funcs(b, funcs);
                }
                _ => {}
            }
        }
    }
    extract_funcs(&ast, &mut funcs);

    (errors_count, error_lines, funcs)
}

pub fn run_in_thread(
    source: &str,
    tx: SyncSender<EngineMessage>,
    input_rx: Receiver<String>,
    cancel_token: Arc<AtomicBool>,
) {
    let mut lexer = lexer::Lexer::new(source);
    let tokens = lexer.tokenize();

    let mut parser = parser::Parser::new(tokens);
    let ast = parser.parse();

    if !parser.errors.is_empty() {
        let mut res = vec!["Syntax Errors:".to_string()];
        res.extend(parser.errors);
        let _ = tx.send(EngineMessage::Output(res));
        return;
    }

    let mut analyzer = analyzer::Analyzer::new();
    analyzer.analyze(&ast);

    if !analyzer.errors.is_empty() {
        let mut res = vec!["Analysis Errors:".to_string()];
        res.extend(analyzer.errors);
        let _ = tx.send(EngineMessage::Output(res));
        return;
    }

    let mut interpreter = interpreter::Interpreter::new(tx, input_rx, cancel_token);
    interpreter.exec(&ast);
}
