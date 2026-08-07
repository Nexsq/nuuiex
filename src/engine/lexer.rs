#[derive(Debug, Clone, PartialEq)]
pub enum TokenKind {
    Ident(String),
    Number(f64),
    String(String),
    True,
    False,
    NoneValue,
    Plus,
    Minus,
    Star,
    Slash,
    Percent,
    Eq,
    PlusEq,
    MinusEq,
    StarEq,
    SlashEq,
    PercentEq,
    EqEq,
    NotEq,
    Less,
    Greater,
    LessEq,
    GreaterEq,
    Let,
    Const,
    Fn,
    Return,
    Loop,
    While,
    For,
    In,
    If,
    Elif,
    Else,
    Break,
    Continue,
    Pass,
    And,
    Or,
    Not,
    Async,
    Colon,
    DoubleColon,
    Indent,
    Dedent,
    LParen,
    RParen,
    LBracket,
    RBracket,
    LBrace,
    RBrace,
    Comma,
    Newline,
    EOF,
    Error(String),
    ImageVariant(String),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    pub kind: TokenKind,
    pub line: usize,
}

pub struct Lexer<'a> {
    source: &'a str,
    chars: std::iter::Peekable<std::str::CharIndices<'a>>,
    line: usize,
    nesting_level: usize,
}

impl<'a> Lexer<'a> {
    pub fn new(source: &'a str) -> Self {
        Self {
            source,
            chars: source.char_indices().peekable(),
            line: 1,
            nesting_level: 0,
        }
    }

    pub fn new_with_line(source: &'a str, start_line: usize) -> Self {
        Self {
            source,
            chars: source.char_indices().peekable(),
            line: start_line,
            nesting_level: 0,
        }
    }

    pub fn tokenize(&mut self) -> Vec<Token> {
        let mut indents = vec![0];
        let mut tokens = Vec::new();
        let mut is_bol = true;

        while !self.is_at_end() {
            if is_bol {
                let mut spaces = 0;
                let mut has_tabs = false;
                while self.peek() == ' ' || self.peek() == '\t' {
                    if self.peek() == '\t' {
                        has_tabs = true;
                    }
                    spaces += 1;
                    self.advance();
                }

                let peek = self.peek();
                if !(peek == '\n' || peek == '\r' || peek == '#' || self.is_at_end()) {
                    if has_tabs {
                        tokens.push(Token {
                            kind: TokenKind::Error("Use spaces, not tabs, for indentation".into()),
                            line: self.line,
                        });
                    }
                    let current = *indents.last().unwrap();
                    if spaces > current {
                        if spaces % 4 != 0 {
                            tokens.push(Token {
                                kind: TokenKind::Error(
                                    "Indentation must be a multiple of 4 spaces".into(),
                                ),
                                line: self.line,
                            });
                        }
                        indents.push(spaces);
                        tokens.push(Token {
                            kind: TokenKind::Indent,
                            line: self.line,
                        });
                    } else if spaces < current {
                        while spaces < *indents.last().unwrap() {
                            indents.pop();
                            tokens.push(Token {
                                kind: TokenKind::Dedent,
                                line: self.line,
                            });
                        }
                        if spaces != *indents.last().unwrap() {
                            tokens.push(Token {
                                kind: TokenKind::Error("Inconsistent indentation".into()),
                                line: self.line,
                            });
                        }
                    }
                    is_bol = false;
                }
            }

            if self.is_at_end() {
                break;
            }

            let token = self.next_token(&mut is_bol);
            if token.kind == TokenKind::EOF {
                break;
            }
            tokens.push(token);
        }

        while indents.len() > 1 {
            indents.pop();
            tokens.push(Token {
                kind: TokenKind::Dedent,
                line: self.line,
            });
        }

        if tokens.last().map(|t| &t.kind) != Some(&TokenKind::EOF) {
            tokens.push(Token {
                kind: TokenKind::EOF,
                line: self.line,
            });
        }
        tokens
    }

    fn is_at_end(&mut self) -> bool {
        self.chars.peek().is_none()
    }

    fn peek(&mut self) -> char {
        self.chars.peek().map(|&(_, c)| c).unwrap_or('\0')
    }

    fn advance(&mut self) -> char {
        self.chars.next().map(|(_, c)| c).unwrap_or('\0')
    }

    fn current_byte_pos(&mut self) -> usize {
        self.chars
            .peek()
            .map(|&(i, _)| i)
            .unwrap_or(self.source.len())
    }

    fn next_token(&mut self, is_bol: &mut bool) -> Token {
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
                    let current_line = self.line;
                    self.line += 1;

                    if self.nesting_level > 0 {
                        continue;
                    }

                    *is_bol = true;
                    return Token {
                        kind: TokenKind::Newline,
                        line: current_line,
                    };
                }
                '+' => {
                    self.advance();
                    if self.peek() == '=' {
                        self.advance();
                        return Token {
                            kind: TokenKind::PlusEq,
                            line: self.line,
                        };
                    }
                    return Token {
                        kind: TokenKind::Plus,
                        line: self.line,
                    };
                }
                '-' => {
                    self.advance();
                    if self.peek() == '=' {
                        self.advance();
                        return Token {
                            kind: TokenKind::MinusEq,
                            line: self.line,
                        };
                    }
                    return Token {
                        kind: TokenKind::Minus,
                        line: self.line,
                    };
                }
                '*' => {
                    self.advance();
                    if self.peek() == '=' {
                        self.advance();
                        return Token {
                            kind: TokenKind::StarEq,
                            line: self.line,
                        };
                    }
                    return Token {
                        kind: TokenKind::Star,
                        line: self.line,
                    };
                }
                '/' => {
                    self.advance();
                    if self.peek() == '=' {
                        self.advance();
                        return Token {
                            kind: TokenKind::SlashEq,
                            line: self.line,
                        };
                    }
                    return Token {
                        kind: TokenKind::Slash,
                        line: self.line,
                    };
                }
                '%' => {
                    self.advance();
                    if self.peek() == '=' {
                        self.advance();
                        return Token {
                            kind: TokenKind::PercentEq,
                            line: self.line,
                        };
                    }
                    return Token {
                        kind: TokenKind::Percent,
                        line: self.line,
                    };
                }
                '=' => {
                    self.advance();
                    if self.peek() == '=' {
                        self.advance();
                        return Token {
                            kind: TokenKind::EqEq,
                            line: self.line,
                        };
                    }
                    return Token {
                        kind: TokenKind::Eq,
                        line: self.line,
                    };
                }
                '<' => {
                    self.advance();
                    if self.peek() == '=' {
                        self.advance();
                        return Token {
                            kind: TokenKind::LessEq,
                            line: self.line,
                        };
                    }
                    return Token {
                        kind: TokenKind::Less,
                        line: self.line,
                    };
                }
                '>' => {
                    self.advance();
                    if self.peek() == '=' {
                        self.advance();
                        return Token {
                            kind: TokenKind::GreaterEq,
                            line: self.line,
                        };
                    }
                    return Token {
                        kind: TokenKind::Greater,
                        line: self.line,
                    };
                }
                '!' => {
                    self.advance();
                    if self.peek() == '=' {
                        self.advance();
                        return Token {
                            kind: TokenKind::NotEq,
                            line: self.line,
                        };
                    }
                    return Token {
                        kind: TokenKind::Not,
                        line: self.line,
                    };
                }
                ':' => {
                    self.advance();
                    if self.peek() == ':' {
                        self.advance();
                        return Token {
                            kind: TokenKind::DoubleColon,
                            line: self.line,
                        };
                    }
                    return Token {
                        kind: TokenKind::Colon,
                        line: self.line,
                    };
                }
                '(' => {
                    self.advance();
                    self.nesting_level += 1;
                    return Token {
                        kind: TokenKind::LParen,
                        line: self.line,
                    };
                }
                ')' => {
                    self.advance();
                    self.nesting_level = self.nesting_level.saturating_sub(1);
                    return Token {
                        kind: TokenKind::RParen,
                        line: self.line,
                    };
                }
                '[' => {
                    self.advance();
                    self.nesting_level += 1;
                    return Token {
                        kind: TokenKind::LBracket,
                        line: self.line,
                    };
                }
                ']' => {
                    self.advance();
                    self.nesting_level = self.nesting_level.saturating_sub(1);
                    return Token {
                        kind: TokenKind::RBracket,
                        line: self.line,
                    };
                }
                '{' => {
                    self.advance();
                    self.nesting_level += 1;
                    return Token {
                        kind: TokenKind::LBrace,
                        line: self.line,
                    };
                }
                '}' => {
                    self.advance();
                    self.nesting_level = self.nesting_level.saturating_sub(1);
                    return Token {
                        kind: TokenKind::RBrace,
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
                '\'' => return self.single_quote_string(),
                '`' => return self.multiline_string(),
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

    fn string(&mut self) -> Token {
        let start_line = self.line;
        self.advance();
        let mut val = String::new();

        while !self.is_at_end() && self.peek() != '"' {
            if self.peek() == '\n' {
                return Token {
                    kind: TokenKind::Error("Unterminated string (newlines not allowed)".into()),
                    line: start_line,
                };
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

    fn single_quote_string(&mut self) -> Token {
        let start_line = self.line;
        self.advance();
        let mut val = String::new();

        while !self.is_at_end() && self.peek() != '\'' {
            if self.peek() == '\n' {
                return Token {
                    kind: TokenKind::Error("Unterminated string (newlines not allowed)".into()),
                    line: start_line,
                };
            } else if self.peek() == '\\' {
                self.advance();
                if self.is_at_end() {
                    break;
                }
                match self.advance() {
                    'n' => val.push('\n'),
                    'r' => val.push('\r'),
                    '\\' => val.push('\\'),
                    '\'' => val.push('\''),
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

    fn multiline_string(&mut self) -> Token {
        let start_line = self.line;
        self.advance();
        let mut val = String::new();

        while !self.is_at_end() && self.peek() != '`' {
            if self.peek() == '\n' {
                val.push(self.advance());
                self.line += 1;
            } else if self.peek() == '\\' {
                self.advance();
                if self.is_at_end() {
                    break;
                }
                match self.advance() {
                    'n' => val.push('\n'),
                    'r' => val.push('\r'),
                    '\\' => val.push('\\'),
                    '`' => val.push('`'),
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
                kind: TokenKind::Error("Unterminated multiline string".into()),
                line: start_line,
            };
        }
        self.advance();

        let mut slice = val.as_str();
        if slice.starts_with("\r\n") {
            slice = &slice[2..];
        } else if slice.starts_with('\n') {
            slice = &slice[1..];
        }

        if slice.ends_with("\r\n") {
            slice = &slice[..slice.len() - 2];
        } else if slice.ends_with('\n') {
            slice = &slice[..slice.len() - 1];
        }

        Token {
            kind: TokenKind::String(slice.to_string()),
            line: start_line,
        }
    }

    fn number(&mut self) -> Token {
        let start = self.current_byte_pos();
        let mut has_dot = false;

        while self.peek().is_ascii_digit() {
            self.advance();
        }
        if self.peek() == '.' {
            has_dot = true;
            self.advance();
            while self.peek().is_ascii_digit() {
                self.advance();
            }
        }

        if !has_dot && (self.peek().is_alphabetic() || self.peek() == '_') {
            while self.peek().is_alphanumeric() || self.peek() == '_' {
                self.advance();
            }
            let end = self.current_byte_pos();
            let text = &self.source[start..end];
            return Token {
                kind: TokenKind::Ident(text.to_string()),
                line: self.line,
            };
        }

        let end = self.current_byte_pos();
        let slice = &self.source[start..end];
        let num: f64 = slice.parse().unwrap_or(0.0);

        Token {
            kind: TokenKind::Number(num),
            line: self.line,
        }
    }

    fn identifier(&mut self) -> Token {
        let start = self.current_byte_pos();

        while self.peek().is_alphanumeric() || self.peek() == '_' {
            self.advance();
        }

        let end = self.current_byte_pos();
        let text = &self.source[start..end];

        if text == "Image" && self.peek() == ':' {
            self.advance();
            if self.peek() == ':' {
                self.advance();
            }
            let mut base64 = String::new();
            while self.peek().is_alphanumeric()
                || self.peek() == '+'
                || self.peek() == '/'
                || self.peek() == '='
            {
                base64.push(self.advance());
            }
            return Token {
                kind: TokenKind::ImageVariant(base64),
                line: self.line,
            };
        }

        let kind = match text {
            "let" => TokenKind::Let,
            "const" => TokenKind::Const,
            "fn" => TokenKind::Fn,
            "return" => TokenKind::Return,
            "loop" => TokenKind::Loop,
            "while" => TokenKind::While,
            "for" => TokenKind::For,
            "in" => TokenKind::In,
            "if" => TokenKind::If,
            "elif" => TokenKind::Elif,
            "else" => TokenKind::Else,
            "break" => TokenKind::Break,
            "continue" => TokenKind::Continue,
            "pass" => TokenKind::Pass,
            "and" => TokenKind::And,
            "or" => TokenKind::Or,
            "not" => TokenKind::Not,
            "True" => TokenKind::True,
            "False" => TokenKind::False,
            "None" => TokenKind::NoneValue,
            "async" => TokenKind::Async,
            _ => TokenKind::Ident(text.to_string()),
        };

        Token {
            kind,
            line: self.line,
        }
    }
}
