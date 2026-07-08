use super::ast::{Expr, Stmt};

pub struct Analyzer {
    pub errors: Vec<String>,
}

impl Analyzer {
    pub fn new() -> Self {
        Self { errors: Vec::new() }
    }

    pub fn analyze(&mut self, stmts: &[Stmt]) {
        for stmt in stmts {
            self.analyze_stmt(stmt);
        }
    }

    fn analyze_stmt(&mut self, stmt: &Stmt) {
        match stmt {
            Stmt::Expr(expr) => {
                if !matches!(expr, Expr::Call(_, _)) {
                    self.errors
                        .push("Standalone expression value is unused.".to_string());
                }
                self.analyze_expr(expr);
            }
        }
    }

    fn analyze_expr(&mut self, expr: &Expr) {
        match expr {
            Expr::Number(_) | Expr::String(_) => {}
            Expr::Ident(name) => {
                self.errors.push(format!("Undefined variable '{}'", name));
            }
            Expr::Binary(left, _, right) => {
                self.analyze_expr(left);
                self.analyze_expr(right);
            }
            Expr::Call(name, args) => {
                if *name != "print" && *name != "println" {
                    self.errors.push(format!("Undefined function '{}'", name));
                }

                for arg in args {
                    self.analyze_expr(arg);
                }
            }
        }
    }
}
