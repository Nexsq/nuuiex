use super::ast::{BinaryOp, Expr, Stmt, StringPart};
use super::lexer::{Token, TokenKind};

pub struct Parser {
    tokens: Vec<Token>,
    current: usize,
    pub errors: Vec<String>,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self {
            tokens,
            current: 0,
            errors: Vec::new(),
        }
    }

    pub fn parse(&mut self) -> Vec<Stmt> {
        let mut stmts = Vec::new();
        while !self.is_at_end() {
            if self.check(&TokenKind::Newline) {
                self.advance();
                continue;
            }
            if let Some(stmt) = self.parse_statement() {
                stmts.push(stmt);
            } else {
                self.synchronize();
            }
        }
        stmts
    }

    fn parse_statement(&mut self) -> Option<Stmt> {
        if self.check(&TokenKind::Let) {
            return self.parse_let_declaration();
        }
        if self.check(&TokenKind::Const) {
            return self.parse_const_declaration();
        }

        let expr = self.parse_expression()?;

        if self.check(&TokenKind::Eq) {
            let eq_token = self.advance().clone();
            if let Expr::Ident(name, _) = expr {
                let value = self.parse_expression()?;
                if self.check(&TokenKind::Newline) || self.is_at_end() {
                    if !self.is_at_end() {
                        self.advance();
                    }
                    return Some(Stmt::Assign(name, value, eq_token.line));
                } else {
                    self.error("Expected newline after assignment.");
                    return None;
                }
            } else {
                self.error("Invalid assignment target.");
                return None;
            }
        }

        if self.check(&TokenKind::Newline) || self.is_at_end() {
            if !self.is_at_end() {
                self.advance();
            }
            Some(Stmt::Expr(expr))
        } else {
            self.error("Expected newline after expression.");
            None
        }
    }

    fn parse_let_declaration(&mut self) -> Option<Stmt> {
        let token = self.advance().clone();
        let name = if let TokenKind::Ident(ref n) = self.peek().kind {
            let name_str = n.clone();
            self.advance();
            name_str
        } else {
            self.error("Expected variable name after 'let'.");
            return None;
        };

        if !self.check(&TokenKind::Eq) {
            self.error("Expected '=' after variable name.");
            return None;
        }
        self.advance();

        let value = self.parse_expression()?;

        if self.check(&TokenKind::Newline) || self.is_at_end() {
            if !self.is_at_end() {
                self.advance();
            }
            Some(Stmt::Let(name, value, token.line))
        } else {
            self.error("Expected newline after variable declaration.");
            None
        }
    }

    fn parse_const_declaration(&mut self) -> Option<Stmt> {
        let token = self.advance().clone();
        let name = if let TokenKind::Ident(ref n) = self.peek().kind {
            let name_str = n.clone();
            self.advance();
            name_str
        } else {
            self.error("Expected variable name after 'const'.");
            return None;
        };

        if !self.check(&TokenKind::Eq) {
            self.error("Expected '=' after variable name.");
            return None;
        }
        self.advance();

        let value = self.parse_expression()?;

        if self.check(&TokenKind::Newline) || self.is_at_end() {
            if !self.is_at_end() {
                self.advance();
            }
            Some(Stmt::Const(name, value, token.line))
        } else {
            self.error("Expected newline after variable declaration.");
            None
        }
    }

    fn parse_expression(&mut self) -> Option<Expr> {
        self.parse_term()
    }

    fn parse_term(&mut self) -> Option<Expr> {
        let mut expr = self.parse_factor()?;

        while self.check(&TokenKind::Plus) || self.check(&TokenKind::Minus) {
            let token = self.advance().clone();
            let op = BinaryOp::from_token(&token.kind).unwrap();
            let right = self.parse_factor()?;
            expr = Expr::Binary(Box::new(expr), op, Box::new(right), token.line);
        }

        Some(expr)
    }

    fn parse_factor(&mut self) -> Option<Expr> {
        let mut expr = self.parse_primary()?;

        while self.check(&TokenKind::Star) || self.check(&TokenKind::Slash) {
            let token = self.advance().clone();
            let op = BinaryOp::from_token(&token.kind).unwrap();
            let right = self.parse_primary()?;
            expr = Expr::Binary(Box::new(expr), op, Box::new(right), token.line);
        }

        Some(expr)
    }

    fn parse_primary(&mut self) -> Option<Expr> {
        if self.is_at_end() {
            self.error("Unexpected end of input.");
            return None;
        }

        let token = self.advance().clone();
        let line = token.line;
        match token.kind {
            TokenKind::Number(n) => Some(Expr::Number(n, line)),
            TokenKind::String(s) => {
                let mut parts = Vec::new();
                let mut current_text = String::new();
                let mut chars = s.chars().peekable();
                let mut has_expr = false;

                while let Some(c) = chars.next() {
                    if c == '\\' {
                        if let Some(&next) = chars.peek() {
                            if next == '{' {
                                current_text.push('{');
                                chars.next();
                                continue;
                            } else if next == '\\' {
                                current_text.push('\\');
                                chars.next();
                                continue;
                            }
                        }
                        current_text.push('\\');
                    } else if c == '{' {
                        if !current_text.is_empty() {
                            parts.push(StringPart::Text(current_text.clone()));
                            current_text.clear();
                        }
                        let mut expr_str = String::new();
                        let mut depth = 1;
                        while let Some(ec) = chars.next() {
                            if ec == '{' {
                                depth += 1;
                                expr_str.push(ec);
                            } else if ec == '}' {
                                depth -= 1;
                                if depth == 0 {
                                    break;
                                } else {
                                    expr_str.push(ec);
                                }
                            } else {
                                expr_str.push(ec);
                            }
                        }

                        let mut lexer = crate::engine::lexer::Lexer::new(&expr_str);
                        let tokens = lexer.tokenize();
                        let mut parser = Parser::new(tokens);

                        if let Some(expr) = parser.parse_expression() {
                            while parser.check(&TokenKind::Newline) {
                                parser.advance();
                            }
                            if !parser.is_at_end() {
                                self.error(&format!(
                                    "Unexpected extra tokens in interpolation: {}",
                                    expr_str
                                ));
                            }
                            parts.push(StringPart::Expr(expr));
                            has_expr = true;
                            self.errors.extend(parser.errors);
                        } else {
                            self.error(&format!(
                                "Invalid expression in interpolation: {}",
                                expr_str
                            ));
                            self.errors.extend(parser.errors);
                        }
                    } else {
                        current_text.push(c);
                    }
                }

                if !current_text.is_empty() {
                    parts.push(StringPart::Text(current_text));
                }

                if has_expr {
                    Some(Expr::FormatString(parts, line))
                } else {
                    let mut final_str = String::new();
                    for part in parts {
                        if let StringPart::Text(t) = part {
                            final_str.push_str(&t);
                        }
                    }
                    Some(Expr::String(final_str, line))
                }
            }
            TokenKind::Ident(name) => {
                let func_name = name;

                if self.check(&TokenKind::LParen) {
                    self.advance();
                    let mut args = Vec::new();

                    if !self.check(&TokenKind::RParen) {
                        loop {
                            if let Some(arg) = self.parse_expression() {
                                args.push(arg);
                            } else {
                                return None;
                            }
                            if self.check(&TokenKind::Comma) {
                                self.advance();
                            } else {
                                break;
                            }
                        }
                    }
                    if self.check(&TokenKind::RParen) {
                        self.advance();
                        Some(Expr::Call(func_name, args, line))
                    } else {
                        self.error("Expected ')' after arguments.");
                        None
                    }
                } else {
                    Some(Expr::Ident(func_name, line))
                }
            }
            TokenKind::LParen => {
                let expr = self.parse_expression()?;
                if self.check(&TokenKind::RParen) {
                    self.advance();
                    Some(expr)
                } else {
                    self.error("Expected ')' after expression.");
                    None
                }
            }
            TokenKind::Error(msg) => {
                self.error(&msg);
                None
            }
            _ => {
                let kind_str = format!("{:?}", token.kind);
                self.error(&format!("Unexpected token: {}", kind_str));
                None
            }
        }
    }

    fn peek(&self) -> &Token {
        &self.tokens[self.current]
    }

    fn previous(&self) -> &Token {
        &self.tokens[self.current - 1]
    }

    fn is_at_end(&self) -> bool {
        self.current >= self.tokens.len() || self.peek().kind == TokenKind::EOF
    }

    fn check(&self, kind: &TokenKind) -> bool {
        if self.is_at_end() {
            return false;
        }
        std::mem::discriminant(&self.peek().kind) == std::mem::discriminant(kind)
    }

    fn advance(&mut self) -> &Token {
        if !self.is_at_end() {
            self.current += 1;
        }
        self.previous()
    }

    fn error(&mut self, msg: &str) {
        let line = if self.is_at_end() {
            if self.tokens.is_empty() {
                0
            } else {
                self.tokens.last().unwrap().line
            }
        } else {
            self.peek().line
        };
        self.errors.push(format!("Line {}: {}", line, msg));
    }

    fn synchronize(&mut self) {
        while !self.is_at_end() {
            if self.peek().kind == TokenKind::Newline {
                self.advance();
                return;
            }
            self.advance();
        }
    }
}
