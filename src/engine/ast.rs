use super::lexer::TokenKind;

#[derive(Debug, Clone)]
pub enum StringPart {
    Text(String),
    Expr(Expr),
}

#[derive(Debug, Clone)]
pub enum Expr {
    Number(f64, usize),
    String(String, usize),
    FormatString(Vec<StringPart>, usize),
    Ident(String, usize),
    Binary(Box<Expr>, BinaryOp, Box<Expr>, usize),
    Call(String, Vec<Expr>, usize),
}

impl Expr {
    pub fn line(&self) -> usize {
        match self {
            Expr::Number(_, l) => *l,
            Expr::String(_, l) => *l,
            Expr::FormatString(_, l) => *l,
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
pub enum Stmt {
    Expr(Expr),
    Let(String, Expr, usize),
    Const(String, Expr, usize),
    Assign(String, Expr, usize),
}
