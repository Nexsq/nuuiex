use super::lexer::TokenKind;

#[derive(Debug, Clone)]
pub struct Param {
    pub name: String,
    pub default: Option<Expr>,
}

#[derive(Debug, Clone)]
pub struct FunctionDef {
    pub name: String,
    pub params: Vec<Param>,
    pub body: Vec<Stmt>,
}

#[derive(Debug, Clone)]
pub enum StringPart {
    Text(String),
    Expr(Expr),
}

#[derive(Debug, Clone)]
pub enum Expr {
    Number(f64),
    String(String),
    Bool(bool),
    Nil,
    FormatString(Vec<StringPart>),
    List(Vec<Expr>),
    Dict(Vec<(Expr, Expr)>),
    Ident(String, usize),
    Index(Box<Expr>, Box<Expr>, usize),
    MethodCall(Box<Expr>, String, Vec<(Option<String>, Expr)>, usize),
    StaticAccess(Box<Expr>, String, usize),
    Binary(Box<Expr>, BinaryOp, Box<Expr>, usize),
    Not(Box<Expr>, usize),
    Call(String, Vec<(Option<String>, Expr)>, usize),
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
    And,
    Or,
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
            TokenKind::And => Some(Self::And),
            TokenKind::Or => Some(Self::Or),
            _ => None,
        }
    }
}

#[derive(Debug, Clone)]
pub enum Stmt {
    Expr(Expr),
    Let(String, Expr, usize),
    Const(String, Expr, usize),
    Assign(Expr, Expr, usize),
    AssignOp(Expr, BinaryOp, Expr, usize),
    If(Expr, Vec<Stmt>, Vec<(Expr, Vec<Stmt>)>, Option<Vec<Stmt>>),
    Loop(Option<Expr>, Vec<Stmt>, usize),
    While(Expr, Vec<Stmt>),
    For(String, Expr, Vec<Stmt>, usize),
    Break(usize),
    Continue(usize),
    Fn(String, Vec<Param>, Vec<Stmt>, usize),
    Return(Option<Expr>),
    Async(Vec<Stmt>, usize),
}
