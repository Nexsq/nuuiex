use super::analyzer;
use super::interpreter;
use super::lexer;
use super::parser;
use std::collections::HashSet;

pub fn analyze_code(source: &str) -> (usize, HashSet<usize>) {
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

    (errors_count, error_lines)
}

pub fn run_in_thread(source: &str, tx: std::sync::mpsc::Sender<Vec<String>>) {
    let mut lexer = lexer::Lexer::new(source);
    let tokens = lexer.tokenize();

    let mut parser = parser::Parser::new(tokens);
    let ast = parser.parse();

    if !parser.errors.is_empty() {
        let mut res = vec!["--- Syntax Errors ---".to_string()];
        res.extend(parser.errors);
        let _ = tx.send(res);
        return;
    }

    let mut analyzer = analyzer::Analyzer::new();
    analyzer.analyze(&ast);

    if !analyzer.errors.is_empty() {
        let mut res = vec!["--- Analysis Errors ---".to_string()];
        res.extend(analyzer.errors);
        let _ = tx.send(res);
        return;
    }

    let mut interpreter = interpreter::Interpreter::new(tx);
    interpreter.exec(&ast);
}
