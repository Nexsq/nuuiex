use super::ast::{Expr, Stmt, StringPart};
use std::collections::HashSet;

pub struct Analyzer {
    pub errors: Vec<String>,
    pub error_lines: HashSet<usize>,
    pub defined_vars: HashSet<String>,
    pub loop_depth: usize,
}

impl Analyzer {
    pub fn new() -> Self {
        Self {
            errors: Vec::new(),
            error_lines: HashSet::new(),
            defined_vars: HashSet::new(),
            loop_depth: 0,
        }
    }

    fn error(&mut self, line: usize, msg: String) {
        self.error_lines.insert(line);
        self.errors.push(format!("Line {}: {}", line, msg));
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
                self.defined_vars.insert(name.clone());
            }
            Stmt::Assign(name, expr, line) => {
                self.analyze_expr(expr);
                if !self.defined_vars.contains(name) {
                    self.error(*line, format!("Undefined variable '{}'", name));
                }
            }
            Stmt::AssignOp(name, _, expr, line) => {
                self.analyze_expr(expr);
                if !self.defined_vars.contains(name) {
                    self.error(*line, format!("Undefined variable '{}'", name));
                }
            }
            Stmt::If(cond, then_b, elifs, else_b) => {
                self.analyze_expr(cond);
                self.analyze(then_b);
                for (elif_cond, elif_b) in elifs {
                    self.analyze_expr(elif_cond);
                    self.analyze(elif_b);
                }
                if let Some(e_b) = else_b {
                    self.analyze(e_b);
                }
            }
            Stmt::Loop(body) => {
                self.loop_depth += 1;
                self.analyze(body);
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
