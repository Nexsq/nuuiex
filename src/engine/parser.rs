use super::ast::{BinaryOp, Expr, Stmt};
use super::lexer::{Token, TokenKind};

pub struct Parser<'a> {
    tokens: Vec<Token<'a>>,
    current: usize,
    pub errors: Vec<String>,
}

impl<'a> Parser<'a> {
    pub fn new(tokens: Vec<Token<'a>>) -> Self {
        Self {
            tokens,
            current: 0,
            errors: Vec::new(),
        }
    }

    pub fn parse(&mut self) -> Vec<Stmt<'a>> {
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

    fn parse_statement(&mut self) -> Option<Stmt<'a>> {
        let expr = self.parse_expression()?;
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

    fn parse_expression(&mut self) -> Option<Expr<'a>> {
        self.parse_term()
    }

    fn parse_term(&mut self) -> Option<Expr<'a>> {
        let mut expr = self.parse_factor()?;

        while self.check(&TokenKind::Plus) || self.check(&TokenKind::Minus) {
            let op = BinaryOp::from_token(&self.advance().kind).unwrap();
            let right = self.parse_factor()?;
            expr = Expr::Binary(Box::new(expr), op, Box::new(right));
        }

        Some(expr)
    }

    fn parse_factor(&mut self) -> Option<Expr<'a>> {
        let mut expr = self.parse_primary()?;

        while self.check(&TokenKind::Star) || self.check(&TokenKind::Slash) {
            let op = BinaryOp::from_token(&self.advance().kind).unwrap();
            let right = self.parse_primary()?;
            expr = Expr::Binary(Box::new(expr), op, Box::new(right));
        }

        Some(expr)
    }

    fn parse_primary(&mut self) -> Option<Expr<'a>> {
        if self.is_at_end() {
            self.error("Unexpected end of input.");
            return None;
        }

        let token = self.advance();
        match &token.kind {
            TokenKind::Number(n) => Some(Expr::Number(*n)),
            TokenKind::String(s) => Some(Expr::String(s.clone())),
            TokenKind::Ident(name) => {
                let func_name = *name;

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
                        Some(Expr::Call(func_name, args))
                    } else {
                        self.error("Expected ')' after arguments.");
                        None
                    }
                } else {
                    Some(Expr::Ident(func_name))
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
                let error_msg = msg.clone();
                self.error(&error_msg);
                None
            }
            _ => {
                let kind_str = format!("{:?}", token.kind);
                self.error(&format!("Unexpected token: {}", kind_str));
                None
            }
        }
    }

    fn peek(&self) -> &Token<'a> {
        &self.tokens[self.current]
    }

    fn previous(&self) -> &Token<'a> {
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

    fn advance(&mut self) -> &Token<'a> {
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
