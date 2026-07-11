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
    Bool(bool, usize),
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
            Expr::Bool(_, l) => *l,
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
    Mod,
    EqEq,
    NotEq,
    Less,
    Greater,
    LessEq,
    GreaterEq,
}

impl BinaryOp {
    pub fn from_token(kind: &TokenKind) -> Option<Self> {
        match kind {
            TokenKind::Plus => Some(Self::Add),
            TokenKind::Minus => Some(Self::Sub),
            TokenKind::Star => Some(Self::Mul),
            TokenKind::Slash => Some(Self::Div),
            TokenKind::Percent => Some(Self::Mod),
            TokenKind::EqEq => Some(Self::EqEq),
            TokenKind::NotEq => Some(Self::NotEq),
            TokenKind::Less => Some(Self::Less),
            TokenKind::Greater => Some(Self::Greater),
            TokenKind::LessEq => Some(Self::LessEq),
            TokenKind::GreaterEq => Some(Self::GreaterEq),
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
    AssignOp(String, BinaryOp, Expr, usize),
    If(Expr, Vec<Stmt>, Vec<(Expr, Vec<Stmt>)>, Option<Vec<Stmt>>),
    Loop(Vec<Stmt>),
    Break(usize),
}
