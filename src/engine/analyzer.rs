use super::ast::{Expr, Stmt, StringPart};
use std::collections::HashSet;

pub struct Analyzer {
    pub errors: Vec<String>,
    pub defined_vars: HashSet<String>,
}

impl Analyzer {
    pub fn new() -> Self {
        Self {
            errors: Vec::new(),
            defined_vars: HashSet::new(),
        }
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
                    self.errors.push(format!(
                        "Line {}: Standalone expression value is unused.",
                        expr.line()
                    ));
                }
                self.analyze_expr(expr);
            }
            Stmt::Let(name, expr, _) | Stmt::Const(name, expr, _) => {
                self.analyze_expr(expr);
                self.defined_vars.insert(name.clone());
            }
            Stmt::Assign(name, expr, line) => {
                self.analyze_expr(expr);
                if !self.defined_vars.contains(name) {
                    self.errors
                        .push(format!("Line {}: Undefined variable '{}'", line, name));
                }
            }
        }
    }

    fn analyze_expr(&mut self, expr: &Expr) {
        match expr {
            Expr::Number(_, _) | Expr::String(_, _) => {}
            Expr::FormatString(parts, _) => {
                for part in parts {
                    if let StringPart::Expr(e) = part {
                        self.analyze_expr(e);
                    }
                }
            }
            Expr::Ident(name, line) => {
                if !self.defined_vars.contains(name) {
                    self.errors
                        .push(format!("Line {}: Undefined variable '{}'", line, name));
                }
            }
            Expr::Binary(left, _, right, _) => {
                self.analyze_expr(left);
                self.analyze_expr(right);
            }
            Expr::Call(name, args, line) => {
                if name != "print" && name != "println" && name != "sleep" && name != "exit" {
                    self.errors
                        .push(format!("Line {}: Undefined function '{}'", line, name));
                }

                for arg in args {
                    self.analyze_expr(arg);
                }
            }
        }
    }
}
