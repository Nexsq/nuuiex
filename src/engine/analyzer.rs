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
                if !matches!(expr, Expr::Call(..)) {
                    self.errors.push(format!(
                        "Line {}: Standalone expression value is unused.",
                        expr.line()
                    ));
                }
                self.analyze_expr(expr);
            }
        }
    }

    fn analyze_expr(&mut self, expr: &Expr) {
        match expr {
            Expr::Number(_, _) | Expr::String(_, _) => {}
            Expr::Ident(name, line) => {
                self.errors
                    .push(format!("Line {}: Undefined variable '{}'", line, name));
            }
            Expr::Binary(left, _, right, _) => {
                self.analyze_expr(left);
                self.analyze_expr(right);
            }
            Expr::Call(name, args, line) => {
                if *name != "print" && *name != "println" {
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
