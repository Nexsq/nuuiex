use super::lexer::TokenKind;

#[derive(Debug, Clone)]
pub enum Expr<'a> {
    Number(f64, usize),
    String(String, usize),
    Ident(&'a str, usize),
    Binary(Box<Expr<'a>>, BinaryOp, Box<Expr<'a>>, usize),
    Call(&'a str, Vec<Expr<'a>>, usize),
}

impl<'a> Expr<'a> {
    pub fn line(&self) -> usize {
        match self {
            Expr::Number(_, l) => *l,
            Expr::String(_, l) => *l,
            Expr::Ident(_, l) => *l,
            Expr::Binary(_, _, _, l) => *l,
            Expr::Call(_, _, l) => *l,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BinaryOp {
    Add,
    Sub,
    Mul,
    Div,
}

impl BinaryOp {
    pub fn from_token(kind: &TokenKind) -> Option<Self> {
        match kind {
            TokenKind::Plus => Some(Self::Add),
            TokenKind::Minus => Some(Self::Sub),
            TokenKind::Star => Some(Self::Mul),
            TokenKind::Slash => Some(Self::Div),
            _ => None,
        }
    }
}

#[derive(Debug, Clone)]
pub enum Stmt<'a> {
    Expr(Expr<'a>),
}
