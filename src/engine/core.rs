use super::analyzer;
use super::interpreter;
use super::lexer;
use super::parser;

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
