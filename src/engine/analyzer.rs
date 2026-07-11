use super::ast::{Expr, Stmt, StringPart};
use std::collections::HashSet;

pub struct Analyzer {
    pub errors: Vec<String>,
    pub error_lines: HashSet<usize>,
    pub scopes: Vec<HashSet<String>>,
    pub loop_depth: usize,
}

impl Analyzer {
    pub fn new() -> Self {
        Self {
            errors: Vec::new(),
            error_lines: HashSet::new(),
            scopes: vec![HashSet::new()],
            loop_depth: 0,
        }
    }

    fn error(&mut self, line: usize, msg: String) {
        self.error_lines.insert(line);
        self.errors.push(format!("Line {}: {}", line, msg));
    }

    fn push_scope(&mut self) {
        self.scopes.push(HashSet::new());
    }

    fn pop_scope(&mut self) {
        self.scopes.pop();
    }

    fn define(&mut self, name: &str) {
        self.scopes.last_mut().unwrap().insert(name.to_string());
    }

    fn is_defined(&self, name: &str) -> bool {
        for scope in self.scopes.iter().rev() {
            if scope.contains(name) {
                return true;
            }
        }
        false
    }

    pub fn analyze(&mut self, stmts: &[Stmt]) {
        for stmt in stmts {
            self.analyze_stmt(stmt);
        }
    }

    fn analyze_stmt(&mut self, stmt: &Stmt) {
        match stmt {
            Stmt::Expr(expr) => {
                if !matches!(expr, Expr::Call(..)) {
                    self.error(expr.line(), "Standalone expression value is unused.".into());
                }
                self.analyze_expr(expr);
            }
            Stmt::Let(name, expr, _) | Stmt::Const(name, expr, _) => {
                self.analyze_expr(expr);
                self.define(name);
            }
            Stmt::Assign(name, expr, line) => {
                self.analyze_expr(expr);
                if !self.is_defined(name) {
                    self.error(*line, format!("Undefined variable '{}'", name));
                }
            }
            Stmt::AssignOp(name, _, expr, line) => {
                self.analyze_expr(expr);
                if !self.is_defined(name) {
                    self.error(*line, format!("Undefined variable '{}'", name));
                }
            }
            Stmt::If(cond, then_b, elifs, else_b) => {
                self.analyze_expr(cond);
                self.push_scope();
                self.analyze(then_b);
                self.pop_scope();

                for (elif_cond, elif_b) in elifs {
                    self.analyze_expr(elif_cond);
                    self.push_scope();
                    self.analyze(elif_b);
                    self.pop_scope();
                }
                if let Some(e_b) = else_b {
                    self.push_scope();
                    self.analyze(e_b);
                    self.pop_scope();
                }
            }
            Stmt::Loop(body) => {
                self.loop_depth += 1;
                self.push_scope();
                self.analyze(body);
                self.pop_scope();
                self.loop_depth -= 1;
            }
            Stmt::Break(line) => {
                if self.loop_depth == 0 {
                    self.error(*line, "Break statement outside of a loop".into());
                }
            }
        }
    }

    fn analyze_expr(&mut self, expr: &Expr) {
        match expr {
            Expr::Number(_, _) | Expr::String(_, _) | Expr::Bool(_, _) => {}
            Expr::FormatString(parts, _) => {
                for part in parts {
                    if let StringPart::Expr(e) = part {
                        self.analyze_expr(e);
                    }
                }
            }
            Expr::Ident(name, line) => {
                if !self.is_defined(name) {
                    self.error(*line, format!("Undefined variable '{}'", name));
                }
            }
            Expr::Binary(left, _, right, _) => {
                self.analyze_expr(left);
                self.analyze_expr(right);
            }
            Expr::Call(name, args, line) => {
                if name != "print" && name != "println" && name != "sleep" && name != "exit" {
                    self.error(*line, format!("Undefined function '{}'", name));
                }

                for arg in args {
                    self.analyze_expr(arg);
                }
            }
        }
    }
}
