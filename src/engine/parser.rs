use super::ast::{BinaryOp, Expr, Param, Stmt, StringPart};
use super::lexer::{Token, TokenKind};
use std::collections::HashSet;

pub struct Parser {
    tokens: Vec<Token>,
    current: usize,
    pub errors: Vec<String>,
    pub error_lines: HashSet<usize>,
    in_dict_key: bool,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self {
            tokens,
            current: 0,
            errors: Vec::new(),
            error_lines: HashSet::new(),
            in_dict_key: false,
        }
    }

    fn peek_next_kind(&self) -> Option<&TokenKind> {
        self.tokens.get(self.current + 1).map(|t| &t.kind)
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

    fn check_statement_end(&self) -> bool {
        self.check(&TokenKind::Newline) || self.is_at_end() || self.check(&TokenKind::Dedent)
    }

    fn consume_statement_end(&mut self) {
        if self.check(&TokenKind::Newline) {
            self.advance();
        }
    }

    fn parse_statement(&mut self) -> Option<Stmt> {
        if self.check(&TokenKind::Let) {
            return self.parse_variable_declaration(false);
        }
        if self.check(&TokenKind::Const) {
            return self.parse_variable_declaration(true);
        }
        if self.check(&TokenKind::Fn) {
            return self.parse_fn();
        }
        if self.check(&TokenKind::Return) {
            return self.parse_return();
        }
        if self.check(&TokenKind::Loop) {
            return self.parse_loop();
        }
        if self.check(&TokenKind::While) {
            return self.parse_while();
        }
        if self.check(&TokenKind::For) {
            return self.parse_for();
        }
        if self.check(&TokenKind::If) {
            return self.parse_if();
        }
        if self.check(&TokenKind::Async) {
            return self.parse_async();
        }
        if self.check(&TokenKind::Break) {
            let token = self.advance().clone();
            if self.check_statement_end() {
                self.consume_statement_end();
                return Some(Stmt::Break(token.line));
            } else {
                self.error("Expected newline after break");
                return None;
            }
        }
        if self.check(&TokenKind::Continue) {
            let token = self.advance().clone();
            if self.check_statement_end() {
                self.consume_statement_end();
                return Some(Stmt::Continue(token.line));
            } else {
                self.error("Expected newline after continue");
                return None;
            }
        }
        if self.check(&TokenKind::Pass) {
            let token = self.advance().clone();
            if self.check_statement_end() {
                self.consume_statement_end();
                return Some(Stmt::Pass(token.line));
            } else {
                self.error("Expected newline after pass");
                return None;
            }
        }

        let expr = self.parse_expression()?;

        if self.check(&TokenKind::Eq)
            || self.check(&TokenKind::PlusEq)
            || self.check(&TokenKind::MinusEq)
            || self.check(&TokenKind::StarEq)
            || self.check(&TokenKind::SlashEq)
            || self.check(&TokenKind::PercentEq)
        {
            let op_token = self.advance().clone();
            if matches!(expr, Expr::Ident(..) | Expr::Index(..)) {
                let value = self.parse_expression()?;
                if self.check_statement_end() {
                    self.consume_statement_end();
                    if op_token.kind == TokenKind::Eq {
                        return Some(Stmt::Assign(expr, value, op_token.line));
                    } else {
                        let bin_op = match op_token.kind {
                            TokenKind::PlusEq => BinaryOp::Add,
                            TokenKind::MinusEq => BinaryOp::Sub,
                            TokenKind::StarEq => BinaryOp::Mul,
                            TokenKind::SlashEq => BinaryOp::Div,
                            TokenKind::PercentEq => BinaryOp::Mod,
                            _ => unreachable!(),
                        };
                        return Some(Stmt::AssignOp(expr, bin_op, value, op_token.line));
                    }
                } else {
                    self.error("Expected newline after assignment");
                    return None;
                }
            } else {
                self.error("Invalid assignment target.");
                return None;
            }
        }

        if self.check_statement_end() {
            self.consume_statement_end();
            Some(Stmt::Expr(expr))
        } else {
            self.error("Expected newline after expression");
            None
        }
    }

    fn parse_fn(&mut self) -> Option<Stmt> {
        let line = self.advance().line;
        let name = if let TokenKind::Ident(ref n) = self.peek().kind {
            let name_str = n.clone();
            self.advance();
            name_str
        } else {
            self.error("Expected function name");
            return None;
        };

        if !self.check(&TokenKind::LParen) {
            self.error("Expected '(' after function name");
            return None;
        }
        self.advance();

        let mut params = Vec::new();
        if !self.check(&TokenKind::RParen) {
            loop {
                let p_name = if let TokenKind::Ident(ref n) = self.peek().kind {
                    let name_str = n.clone();
                    self.advance();
                    name_str
                } else {
                    self.error("Expected parameter name");
                    return None;
                };

                let mut default = None;
                if self.check(&TokenKind::Eq) {
                    self.advance();
                    default = self.parse_expression();
                }

                params.push(Param {
                    name: p_name,
                    default,
                });

                if self.check(&TokenKind::Comma) {
                    self.advance();
                    if self.check(&TokenKind::RParen) {
                        break;
                    }
                } else {
                    break;
                }
            }
        }

        if self.check(&TokenKind::RParen) {
            self.advance();
        } else {
            self.error("Expected ')' after parameters");
            return None;
        }

        if !self.check(&TokenKind::Colon) {
            self.error("Expected ':' after function signature");
            return None;
        }
        self.advance();
        if !self.check_statement_end() {
            self.error("Expected newline after ':'");
            return None;
        }
        self.consume_statement_end();

        let body = self.parse_block()?;
        Some(Stmt::Fn(name, params, body, line))
    }

    fn parse_return(&mut self) -> Option<Stmt> {
        self.advance();
        let mut value = None;
        if !self.check_statement_end() {
            value = self.parse_expression();
        }
        if self.check_statement_end() {
            self.consume_statement_end();
            Some(Stmt::Return(value))
        } else {
            self.error("Expected newline after return value");
            None
        }
    }

    fn parse_block(&mut self) -> Option<Vec<Stmt>> {
        while self.check(&TokenKind::Newline) {
            self.advance();
        }

        if !self.check(&TokenKind::Indent) {
            self.error("Expected indentation block");
            return None;
        }
        self.advance();

        let mut stmts = Vec::new();
        while !self.is_at_end() && !self.check(&TokenKind::Dedent) {
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

        if self.check(&TokenKind::Dedent) {
            self.advance();
        } else {
            self.error("Expected dedent");
        }
        Some(stmts)
    }

    fn parse_loop(&mut self) -> Option<Stmt> {
        let token = self.advance().clone();

        let mut count_expr = None;
        if !self.check(&TokenKind::Colon) {
            count_expr = self.parse_expression();
        }

        if !self.check(&TokenKind::Colon) {
            self.error("Expected ':' after loop");
            return None;
        }
        self.advance();
        if !self.check_statement_end() {
            self.error("Expected newline after ':'");
            return None;
        }
        self.consume_statement_end();

        let body = self.parse_block()?;
        Some(Stmt::Loop(count_expr, body, token.line))
    }

    fn parse_while(&mut self) -> Option<Stmt> {
        self.advance();
        let cond = self.parse_expression()?;
        if !self.check(&TokenKind::Colon) {
            self.error("Expected ':' after while condition");
            return None;
        }
        self.advance();
        if !self.check_statement_end() {
            self.error("Expected newline after ':'");
            return None;
        }
        self.consume_statement_end();

        let body = self.parse_block()?;
        Some(Stmt::While(cond, body))
    }

    fn parse_for(&mut self) -> Option<Stmt> {
        let token = self.advance().clone();
        let name = if let TokenKind::Ident(ref n) = self.peek().kind {
            let name_str = n.clone();
            self.advance();
            name_str
        } else {
            self.error("Expected variable name after 'for'");
            return None;
        };

        if !self.check(&TokenKind::In) {
            self.error("Expected 'in' after for loop variable");
            return None;
        }
        self.advance();

        let iterable = self.parse_expression()?;

        if !self.check(&TokenKind::Colon) {
            self.error("Expected ':' after for loop iterable");
            return None;
        }
        self.advance();
        if !self.check_statement_end() {
            self.error("Expected newline after ':'");
            return None;
        }
        self.consume_statement_end();

        let body = self.parse_block()?;
        Some(Stmt::For(name, iterable, body, token.line))
    }

    fn parse_async(&mut self) -> Option<Stmt> {
        let token = self.advance().clone();
        if !self.check(&TokenKind::Colon) {
            self.error("Expected ':' after 'async'");
            return None;
        }
        self.advance();
        if !self.check_statement_end() {
            self.error("Expected newline after ':'");
            return None;
        }
        self.consume_statement_end();
        let body = self.parse_block()?;
        Some(Stmt::Async(body, token.line))
    }

    fn parse_if(&mut self) -> Option<Stmt> {
        self.advance();
        let cond = self.parse_expression()?;
        if !self.check(&TokenKind::Colon) {
            self.error("Expected ':' after if condition");
            return None;
        }
        self.advance();
        if !self.check_statement_end() {
            self.error("Expected newline after ':'");
            return None;
        }
        self.consume_statement_end();

        let then_branch = self.parse_block()?;

        let mut elifs = Vec::new();
        while self.check(&TokenKind::Elif) {
            self.advance();
            let elif_cond = self.parse_expression()?;
            if !self.check(&TokenKind::Colon) {
                self.error("Expected ':' after elif condition");
                return None;
            }
            self.advance();
            if !self.check_statement_end() {
                self.error("Expected newline after ':'");
                return None;
            }
            self.consume_statement_end();

            let elif_branch = self.parse_block()?;
            elifs.push((elif_cond, elif_branch));
        }

        let mut else_branch = None;
        if self.check(&TokenKind::Else) {
            self.advance();
            if !self.check(&TokenKind::Colon) {
                self.error("Expected ':' after else");
                return None;
            }
            self.advance();
            if !self.check_statement_end() {
                self.error("Expected newline after ':'");
                return None;
            }
            self.consume_statement_end();

            else_branch = Some(self.parse_block()?);
        }

        Some(Stmt::If(cond, then_branch, elifs, else_branch))
    }

    fn parse_variable_declaration(&mut self, is_const: bool) -> Option<Stmt> {
        let token = self.advance().clone();
        let keyword = if is_const { "const" } else { "let" };
        let name = if let TokenKind::Ident(ref n) = self.peek().kind {
            let name_str = n.clone();
            self.advance();
            name_str
        } else {
            self.error(&format!("Expected variable name after '{}'", keyword));
            return None;
        };

        if !self.check(&TokenKind::Eq) {
            self.error("Expected '=' after variable name");
            return None;
        }
        self.advance();

        let value = self.parse_expression()?;

        if self.check_statement_end() {
            self.consume_statement_end();
            if is_const {
                Some(Stmt::Const(name, value, token.line))
            } else {
                Some(Stmt::Let(name, value, token.line))
            }
        } else {
            self.error("Expected newline after variable declaration");
            None
        }
    }

    fn parse_expression(&mut self) -> Option<Expr> {
        self.parse_or()
    }

    fn parse_or(&mut self) -> Option<Expr> {
        let mut expr = self.parse_and()?;
        while self.check(&TokenKind::Or) {
            let token = self.advance().clone();
            let op = BinaryOp::from_token(&token.kind).unwrap();
            let right = self.parse_and()?;
            expr = Expr::Binary(Box::new(expr), op, Box::new(right), token.line);
        }
        Some(expr)
    }

    fn parse_and(&mut self) -> Option<Expr> {
        let mut expr = self.parse_equality()?;
        while self.check(&TokenKind::And) {
            let token = self.advance().clone();
            let op = BinaryOp::from_token(&token.kind).unwrap();
            let right = self.parse_equality()?;
            expr = Expr::Binary(Box::new(expr), op, Box::new(right), token.line);
        }
        Some(expr)
    }

    fn parse_equality(&mut self) -> Option<Expr> {
        let mut expr = self.parse_relational()?;
        while self.check(&TokenKind::EqEq) || self.check(&TokenKind::NotEq) {
            let token = self.advance().clone();
            let op = BinaryOp::from_token(&token.kind).unwrap();
            let right = self.parse_relational()?;
            expr = Expr::Binary(Box::new(expr), op, Box::new(right), token.line);
        }
        Some(expr)
    }

    fn parse_relational(&mut self) -> Option<Expr> {
        let mut expr = self.parse_term()?;
        while self.check(&TokenKind::Less)
            || self.check(&TokenKind::Greater)
            || self.check(&TokenKind::LessEq)
            || self.check(&TokenKind::GreaterEq)
        {
            let token = self.advance().clone();
            let op = BinaryOp::from_token(&token.kind).unwrap();
            let right = self.parse_term()?;
            expr = Expr::Binary(Box::new(expr), op, Box::new(right), token.line);
        }
        Some(expr)
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
        let mut expr = self.parse_unary()?;

        while self.check(&TokenKind::Star)
            || self.check(&TokenKind::Slash)
            || self.check(&TokenKind::Percent)
        {
            let token = self.advance().clone();
            let op = BinaryOp::from_token(&token.kind).unwrap();
            let right = self.parse_unary()?;
            expr = Expr::Binary(Box::new(expr), op, Box::new(right), token.line);
        }

        Some(expr)
    }

    fn parse_unary(&mut self) -> Option<Expr> {
        if self.check(&TokenKind::Not) {
            let token = self.advance().clone();
            let right = self.parse_unary()?;
            return Some(Expr::Not(Box::new(right), token.line));
        }
        if self.check(&TokenKind::Minus) || self.check(&TokenKind::Plus) {
            let token = self.advance().clone();
            let op = BinaryOp::from_token(&token.kind).unwrap();
            let right = self.parse_unary()?;
            return Some(Expr::Binary(
                Box::new(Expr::Number(0.0)),
                op,
                Box::new(right),
                token.line,
            ));
        }
        self.parse_call()
    }

    fn parse_call(&mut self) -> Option<Expr> {
        let mut expr = self.parse_primary()?;

        loop {
            if self.check(&TokenKind::LBracket) {
                let line = self.advance().line;
                let index = self.parse_expression()?;
                if self.check(&TokenKind::RBracket) {
                    self.advance();
                    expr = Expr::Index(Box::new(expr), Box::new(index), line);
                } else {
                    self.error("Expected ']' after index");
                    return None;
                }
            } else if !self.in_dict_key
                && (self.check(&TokenKind::DoubleColon) || self.check(&TokenKind::Colon))
                && (matches!(self.peek_next_kind(), Some(TokenKind::Ident(_)))
                    || matches!(self.peek_next_kind(), Some(TokenKind::Number(_)))
                    || matches!(self.peek_next_kind(), Some(TokenKind::NoneValue))
                    || matches!(self.peek_next_kind(), Some(TokenKind::True))
                    || matches!(self.peek_next_kind(), Some(TokenKind::False)))
            {
                let is_double = self.check(&TokenKind::DoubleColon);
                let line = self.advance().line;
                let method = match self.peek().kind.clone() {
                    TokenKind::Ident(m) => m,
                    TokenKind::Number(n) => {
                        if n.fract() == 0.0 && n >= 0.0 && n <= 999999.0 {
                            format!("{:06}", n as u64)
                        } else {
                            n.to_string()
                        }
                    }
                    TokenKind::NoneValue => "None".to_string(),
                    TokenKind::True => "True".to_string(),
                    TokenKind::False => "False".to_string(),
                    _ => unreachable!(),
                };
                self.advance();

                if self.check(&TokenKind::LParen) {
                    if !is_double {
                        let is_enum = if let Expr::Ident(ref name, _) = expr {
                            name == "Key"
                                || name == "Color"
                                || name == "Background"
                                || name == "Modifier"
                        } else {
                            false
                        };
                        if !is_enum {
                            self.error("Use '::' for method calls, not ':'.");
                            return None;
                        }
                    }
                    self.advance();
                    let mut args = Vec::new();

                    if !self.check(&TokenKind::RParen) {
                        loop {
                            let mut kw_name = None;
                            if let TokenKind::Ident(ref n) = self.peek().kind {
                                if self.tokens.get(self.current + 1).map(|t| &t.kind)
                                    == Some(&TokenKind::Eq)
                                {
                                    kw_name = Some(n.clone());
                                    self.advance();
                                    self.advance();
                                }
                            }

                            if let Some(arg) = self.parse_expression() {
                                args.push((kw_name, arg));
                            } else {
                                return None;
                            }
                            if self.check(&TokenKind::Comma) {
                                self.advance();
                                if self.check(&TokenKind::RParen) {
                                    break;
                                }
                            } else {
                                break;
                            }
                        }
                    }
                    if self.check(&TokenKind::RParen) {
                        self.advance();
                        expr = Expr::MethodCall(Box::new(expr), method, args, line);
                    } else {
                        self.error("Expected ')' after arguments");
                        return None;
                    }
                } else {
                    if is_double {
                        self.error("Expected '(' after method name");
                        return None;
                    }
                    expr = Expr::StaticAccess(Box::new(expr), method, line);
                }
            } else {
                break;
            }
        }

        Some(expr)
    }

    fn parse_primary(&mut self) -> Option<Expr> {
        if self.is_at_end() {
            self.error("Unexpected end of input");
            return None;
        }

        let token = self.advance().clone();
        let line = token.line;
        match token.kind {
            TokenKind::Number(n) => Some(Expr::Number(n)),
            TokenKind::True => Some(Expr::Bool(true)),
            TokenKind::False => Some(Expr::Bool(false)),
            TokenKind::NoneValue => Some(Expr::Nil),
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

                        let mut lexer = crate::engine::lexer::Lexer::new_with_line(&expr_str, line);
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
                            self.error_lines.extend(parser.error_lines);
                        } else {
                            self.error(&format!(
                                "Invalid expression in interpolation: {}",
                                expr_str
                            ));
                            self.errors.extend(parser.errors);
                            self.error_lines.extend(parser.error_lines);
                        }
                    } else {
                        current_text.push(c);
                    }
                }

                if !current_text.is_empty() {
                    parts.push(StringPart::Text(current_text));
                }

                if has_expr {
                    Some(Expr::FormatString(parts))
                } else {
                    let mut final_str = String::new();
                    for part in parts {
                        if let StringPart::Text(t) = part {
                            final_str.push_str(&t);
                        }
                    }
                    Some(Expr::String(final_str))
                }
            }
            TokenKind::ImageVariant(ref base64) => Some(Expr::StaticAccess(
                Box::new(Expr::Ident("Image".to_string(), line)),
                base64.clone(),
                line,
            )),
            TokenKind::Ident(name) => {
                let func_name = name;

                if self.check(&TokenKind::LParen) {
                    self.advance();
                    let mut args = Vec::new();

                    if !self.check(&TokenKind::RParen) {
                        loop {
                            let mut kw_name = None;
                            if let TokenKind::Ident(ref n) = self.peek().kind {
                                if self.tokens.get(self.current + 1).map(|t| &t.kind)
                                    == Some(&TokenKind::Eq)
                                {
                                    kw_name = Some(n.clone());
                                    self.advance();
                                    self.advance();
                                }
                            }

                            if let Some(arg) = self.parse_expression() {
                                args.push((kw_name, arg));
                            } else {
                                return None;
                            }
                            if self.check(&TokenKind::Comma) {
                                self.advance();
                                if self.check(&TokenKind::RParen) {
                                    break;
                                }
                            } else {
                                break;
                            }
                        }
                    }
                    if self.check(&TokenKind::RParen) {
                        self.advance();
                        Some(Expr::Call(func_name, args, line))
                    } else {
                        self.error("Expected ')' after arguments");
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
                    self.error("Expected ')' after expression");
                    None
                }
            }
            TokenKind::LBrace => {
                let mut items = Vec::new();
                if !self.check(&TokenKind::RBrace) {
                    loop {
                        let prev_in_dict = self.in_dict_key;
                        self.in_dict_key = true;
                        let key_opt = self.parse_expression();
                        self.in_dict_key = prev_in_dict;

                        if let Some(key) = key_opt {
                            if self.check(&TokenKind::Colon) {
                                self.advance();
                                if let Some(val) = self.parse_expression() {
                                    items.push((key, val));
                                } else {
                                    return None;
                                }
                            } else {
                                self.error("Expected ':' after dictionary key");
                                return None;
                            }
                        } else {
                            return None;
                        }
                        if self.check(&TokenKind::Comma) {
                            self.advance();
                            if self.check(&TokenKind::RBrace) {
                                break;
                            }
                        } else {
                            break;
                        }
                    }
                }
                if self.check(&TokenKind::RBrace) {
                    self.advance();
                    Some(Expr::Dict(items))
                } else {
                    self.error("Expected '}' after dictionary");
                    None
                }
            }
            TokenKind::LBracket => {
                let mut items = Vec::new();
                if !self.check(&TokenKind::RBracket) {
                    loop {
                        if let Some(item) = self.parse_expression() {
                            items.push(item);
                        } else {
                            return None;
                        }
                        if self.check(&TokenKind::Comma) {
                            self.advance();
                            if self.check(&TokenKind::RBracket) {
                                break;
                            }
                        } else {
                            break;
                        }
                    }
                }
                if self.check(&TokenKind::RBracket) {
                    self.advance();
                    Some(Expr::List(items))
                } else {
                    self.error("Expected ']' after list items");
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
        self.error_lines.insert(line);
        self.errors.push(format!("Line {}: {}", line, msg));
    }

    fn synchronize(&mut self) {
        while !self.is_at_end() {
            if self.peek().kind == TokenKind::Newline {
                self.advance();
                return;
            }
            if matches!(self.peek().kind, TokenKind::Dedent | TokenKind::EOF) {
                return;
            }
            self.advance();
        }
    }
}
