#[derive(Debug, Clone, PartialEq)]
pub enum TokenKind<'a> {
    Ident(&'a str),
    Number(f64),
    String(String),
    Plus,
    Minus,
    Star,
    Slash,
    LParen,
    RParen,
    Comma,
    Newline,
    EOF,
    Error(String),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Token<'a> {
    pub kind: TokenKind<'a>,
    pub line: usize,
}

pub struct Lexer<'a> {
    source: &'a str,
    pos: usize,
    line: usize,
}

impl<'a> Lexer<'a> {
    pub fn new(source: &'a str) -> Self {
        Self {
            source,
            pos: 0,
            line: 1,
        }
    }

    pub fn tokenize(&mut self) -> Vec<Token<'a>> {
        let mut tokens = Vec::new();
        while !self.is_at_end() {
            let token = self.next_token();
            let is_eof = token.kind == TokenKind::EOF;
            tokens.push(token);
            if is_eof {
                break;
            }
        }
        tokens
    }

    fn is_at_end(&self) -> bool {
        self.pos >= self.source.len()
    }

    fn peek(&self) -> char {
        self.source[self.pos..].chars().next().unwrap_or('\0')
    }

    fn advance(&mut self) -> char {
        if let Some(c) = self.source[self.pos..].chars().next() {
            self.pos += c.len_utf8();
            c
        } else {
            '\0'
        }
    }

    fn next_token(&mut self) -> Token<'a> {
        while !self.is_at_end() {
            let c = self.peek();
            match c {
                ' ' | '\t' | '\r' => {
                    self.advance();
                }
                '#' => {
                    while !self.is_at_end() && self.peek() != '\n' {
                        self.advance();
                    }
                }
                '\n' => {
                    self.advance();
                    let tok = Token {
                        kind: TokenKind::Newline,
                        line: self.line,
                    };
                    self.line += 1;
                    return tok;
                }
                '+' => {
                    self.advance();
                    return Token {
                        kind: TokenKind::Plus,
                        line: self.line,
                    };
                }
                '-' => {
                    self.advance();
                    return Token {
                        kind: TokenKind::Minus,
                        line: self.line,
                    };
                }
                '*' => {
                    self.advance();
                    return Token {
                        kind: TokenKind::Star,
                        line: self.line,
                    };
                }
                '/' => {
                    self.advance();
                    return Token {
                        kind: TokenKind::Slash,
                        line: self.line,
                    };
                }
                '(' => {
                    self.advance();
                    return Token {
                        kind: TokenKind::LParen,
                        line: self.line,
                    };
                }
                ')' => {
                    self.advance();
                    return Token {
                        kind: TokenKind::RParen,
                        line: self.line,
                    };
                }
                ',' => {
                    self.advance();
                    return Token {
                        kind: TokenKind::Comma,
                        line: self.line,
                    };
                }
                '"' => return self.string(),
                _ if c.is_ascii_digit() => return self.number(),
                _ if c.is_alphabetic() || c == '_' => return self.identifier(),
                _ => {
                    self.advance();
                    return Token {
                        kind: TokenKind::Error(format!("Unexpected character: {}", c)),
                        line: self.line,
                    };
                }
            }
        }
        Token {
            kind: TokenKind::EOF,
            line: self.line,
        }
    }

    fn string(&mut self) -> Token<'a> {
        let start_line = self.line;
        self.advance();
        let mut val = String::new();

        while !self.is_at_end() && self.peek() != '"' {
            if self.peek() == '\n' {
                self.line += 1;
                val.push(self.advance());
            } else if self.peek() == '\\' {
                self.advance();
                if self.is_at_end() {
                    break;
                }
                match self.advance() {
                    'n' => val.push('\n'),
                    'r' => val.push('\r'),
                    '\\' => val.push('\\'),
                    '"' => val.push('"'),
                    c => {
                        val.push('\\');
                        val.push(c);
                    }
                }
            } else {
                val.push(self.advance());
            }
        }

        if self.is_at_end() {
            return Token {
                kind: TokenKind::Error("Unterminated string".into()),
                line: start_line,
            };
        }
        self.advance();

        Token {
            kind: TokenKind::String(val),
            line: start_line,
        }
    }

    fn number(&mut self) -> Token<'a> {
        let start = self.pos;
        while self.peek().is_ascii_digit() {
            self.advance();
        }
        if self.peek() == '.' {
            self.advance();
            while self.peek().is_ascii_digit() {
                self.advance();
            }
        }
        let slice = &self.source[start..self.pos];
        let num: f64 = slice.parse().unwrap_or(0.0);

        Token {
            kind: TokenKind::Number(num),
            line: self.line,
        }
    }

    fn identifier(&mut self) -> Token<'a> {
        let start = self.pos;
        while self.peek().is_alphanumeric() || self.peek() == '_' {
            self.advance();
        }
        Token {
            kind: TokenKind::Ident(&self.source[start..self.pos]),
            line: self.line,
        }
    }
}
