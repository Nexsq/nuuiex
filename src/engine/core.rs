use super::analyzer;
use super::interpreter;
use super::lexer;
use super::parser;

pub fn run(source: &str) -> Vec<String> {
    let mut lexer = lexer::Lexer::new(source);
    let tokens = lexer.tokenize();

    let mut parser = parser::Parser::new(tokens);
    let ast = parser.parse();

    if !parser.errors.is_empty() {
        let mut res = vec!["--- Syntax Errors ---".to_string()];
        res.extend(parser.errors);
        return res;
    }

    let mut analyzer = analyzer::Analyzer::new();
    analyzer.analyze(&ast);

    if !analyzer.errors.is_empty() {
        let mut res = vec!["--- Analysis Errors ---".to_string()];
        res.extend(analyzer.errors);
        return res;
    }

    let mut interpreter = interpreter::Interpreter::new();
    interpreter.exec(&ast);

    let mut res = interpreter.output;

    if !interpreter.errors.is_empty() {
        if !res.is_empty() {
            res.push("".to_string());
        }
        res.push("--- Runtime Errors ---".to_string());
        res.extend(interpreter.errors);
    }

    if res.is_empty() {
        res.push("Execution finished with no output.".to_string());
    }

    res
}
