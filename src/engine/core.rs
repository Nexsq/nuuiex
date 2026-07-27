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
    Output(Vec<String>, usize, usize),
    InputRequest,
}

#[inline]
pub fn is_valid_key_variant(name: &str) -> bool {
    matches!(
        name,
        "Up" | "Down"
            | "Left"
            | "Right"
            | "ShiftUp"
            | "ShiftDown"
            | "ShiftLeft"
            | "ShiftRight"
            | "CtrlUp"
            | "CtrlDown"
            | "CtrlLeft"
            | "CtrlRight"
            | "CtrlShiftUp"
            | "CtrlShiftDown"
            | "CtrlShiftLeft"
            | "CtrlShiftRight"
            | "Delete"
            | "CtrlDelete"
            | "Char"
            | "Shift"
            | "Ctrl"
            | "Alt"
            | "Esc"
            | "Enter"
            | "Tab"
            | "Backspace"
            | "CtrlBackspace"
            | "None"
            | "F"
            | "Space"
            | "CapsLock"
            | "PgUp"
            | "PgDn"
            | "Home"
            | "End"
            | "PrtScr"
            | "Insert"
            | "LMeta"
            | "RMeta"
            | "LMB"
            | "RMB"
            | "MMB"
            | "SB1"
            | "SB2"
    )
}

#[inline]
pub fn is_valid_modifier_variant(name: &str) -> bool {
    matches!(
        name,
        "None" | "Bold" | "Dim" | "Italic" | "Underline" | "Reverse" | "Strikethrough"
    )
}

pub const BUILTIN_FUNCS: &[&str] = &[
    "print",
    "println",
    "displayx",
    "displayy",
    "sleep",
    "sleepaccurate",
    "exit",
    "range",
    "random",
    "input",
    "len",
    "max",
    "min",
    "exec",
    "onlinux",
    "onwindows",
    "isdown",
    "isup",
    "isdownfocus",
    "isupfocus",
    "keydown",
    "keyup",
    "write",
    "mousex",
    "mousey",
    "mousedelta",
    "setmouse",
    "activekeys",
    "clear",
    "time",
    "setcaret",
    "scroll",
    "getpixel",
    "compixel",
    "macrodata",
    "keypress",
    "beep",
    "caretx",
    "carety",
    "screenx",
    "screeny",
    "focused",
    "interrupt",
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
                ast::Stmt::Loop(b)
                | ast::Stmt::While(_, b)
                | ast::Stmt::For(_, _, b, _)
                | ast::Stmt::Async(b, _) => {
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
    focus_token: Arc<AtomicBool>,
    display_size: Arc<std::sync::atomic::AtomicU32>,
    macro_rel_path: String,
) {
    let mut lexer = lexer::Lexer::new(source);
    let tokens = lexer.tokenize();

    let mut parser = parser::Parser::new(tokens);
    let ast = parser.parse();

    if !parser.errors.is_empty() {
        let mut res = vec!["Syntax Errors:".to_string()];
        res.extend(parser.errors);
        let _ = tx.send(EngineMessage::Output(res, 0, 0));
        return;
    }

    let mut analyzer = analyzer::Analyzer::new();
    analyzer.analyze(&ast);

    if !analyzer.errors.is_empty() {
        let mut res = vec!["Analysis Errors:".to_string()];
        res.extend(analyzer.errors);
        let _ = tx.send(EngineMessage::Output(res, 0, 0));
        return;
    }

    let mut interpreter = interpreter::Interpreter::new(
        tx,
        input_rx,
        cancel_token,
        focus_token,
        display_size,
        macro_rel_path,
    );
    interpreter.exec(&ast);
}
