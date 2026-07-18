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
        let mut root_scope = HashSet::new();
        root_scope.insert("Key".to_string());
        root_scope.insert("Color".to_string());
        Self {
            errors: Vec::new(),
            error_lines: HashSet::new(),
            scopes: vec![root_scope],
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

    fn define(&mut self, name: &str, line: usize) {
        if self.scopes.last().unwrap().contains(name) {
            self.error(
                line,
                format!("Variable '{}' is already defined in this scope", name),
            );
        }
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
            if let Stmt::Fn(name, _, _, line) = stmt {
                self.define(name, *line);
            }
        }
        for stmt in stmts {
            self.analyze_stmt(stmt);
        }
    }

    fn analyze_stmt(&mut self, stmt: &Stmt) {
        match stmt {
            Stmt::Expr(expr) => {
                self.analyze_expr(expr);
            }
            Stmt::Let(name, expr, line) | Stmt::Const(name, expr, line) => {
                self.analyze_expr(expr);
                self.define(name, *line);
            }
            Stmt::Assign(target, expr, line) => {
                self.analyze_expr(expr);
                self.check_lvalue(target, *line);
            }
            Stmt::AssignOp(target, _, expr, line) => {
                self.analyze_expr(expr);
                self.check_lvalue(target, *line);
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
            Stmt::While(cond, body) => {
                self.analyze_expr(cond);
                self.loop_depth += 1;
                self.push_scope();
                self.analyze(body);
                self.pop_scope();
                self.loop_depth -= 1;
            }
            Stmt::For(name, expr, body, line) => {
                self.analyze_expr(expr);
                self.loop_depth += 1;
                self.push_scope();
                self.define(name, *line);
                self.analyze(body);
                self.pop_scope();
                self.loop_depth -= 1;
            }
            Stmt::Async(body, _) => {
                self.push_scope();
                self.analyze(body);
                self.pop_scope();
            }
            Stmt::Break(line) => {
                if self.loop_depth == 0 {
                    self.error(*line, "Break statement outside of a loop".into());
                }
            }
            Stmt::Fn(_, params, body, line) => {
                for param in params {
                    if let Some(default) = &param.default {
                        self.analyze_expr(default);
                    }
                }
                self.push_scope();
                for param in params {
                    self.define(&param.name, *line);
                }
                self.analyze(body);
                self.pop_scope();
            }
            Stmt::Return(expr) => {
                if let Some(e) = expr {
                    self.analyze_expr(e);
                }
            }
        }
    }

    fn analyze_expr(&mut self, expr: &Expr) {
        match expr {
            Expr::Number(_) | Expr::String(_) | Expr::Bool(_) | Expr::Nil => {}
            Expr::FormatString(parts) => {
                for part in parts {
                    if let StringPart::Expr(e) = part {
                        self.analyze_expr(e);
                    }
                }
            }
            Expr::List(items) => {
                for item in items {
                    self.analyze_expr(item);
                }
            }
            Expr::Dict(items) => {
                for (key, val) in items {
                    self.analyze_expr(key);
                    self.analyze_expr(val);
                }
            }
            Expr::Ident(name, line) => {
                if !self.is_defined(name) {
                    self.error(*line, format!("Undefined variable '{}'", name));
                }
            }
            Expr::Index(left, index, _) => {
                self.analyze_expr(left);
                self.analyze_expr(index);
            }
            Expr::StaticAccess(left, prop, line) => {
                self.analyze_expr(left);
                if let Expr::Ident(name, _) = &**left {
                    if name == "Key" {
                        let valid_variants = [
                            "Up",
                            "Down",
                            "Left",
                            "Right",
                            "ShiftUp",
                            "ShiftDown",
                            "ShiftLeft",
                            "ShiftRight",
                            "CtrlUp",
                            "CtrlDown",
                            "CtrlLeft",
                            "CtrlRight",
                            "CtrlShiftUp",
                            "CtrlShiftDown",
                            "CtrlShiftLeft",
                            "CtrlShiftRight",
                            "Delete",
                            "CtrlDelete",
                            "Char",
                            "Shift",
                            "Ctrl",
                            "Alt",
                            "Esc",
                            "Enter",
                            "Tab",
                            "Backspace",
                            "CtrlBackspace",
                            "None",
                            "F",
                            "Space",
                            "CapsLock",
                            "PgUp",
                            "PgDn",
                            "Home",
                            "End",
                            "PrtScr",
                            "Insert",
                            "LWin",
                            "RWin",
                            "LMB",
                            "RMB",
                            "MMB",
                            "SB1",
                            "SB2",
                        ];
                        if !valid_variants.contains(&prop.as_str()) {
                            self.error(*line, format!("Invalid variant '{}' for enum 'Key'", prop));
                        }
                    } else if name == "Color" {
                        if crate::theme::themecore::parse_color(prop).is_err() {
                            self.error(*line, format!("Invalid color variant '{}'", prop));
                        }
                    }
                }
            }
            Expr::MethodCall(left, method, args, line) => {
                self.analyze_expr(left);
                for (_, arg) in args {
                    self.analyze_expr(arg);
                }
                if let Expr::Ident(name, _) = &**left {
                    if name == "Key" {
                        let valid_variants = [
                            "Up",
                            "Down",
                            "Left",
                            "Right",
                            "ShiftUp",
                            "ShiftDown",
                            "ShiftLeft",
                            "ShiftRight",
                            "CtrlUp",
                            "CtrlDown",
                            "CtrlLeft",
                            "CtrlRight",
                            "CtrlShiftUp",
                            "CtrlShiftDown",
                            "CtrlShiftLeft",
                            "CtrlShiftRight",
                            "Delete",
                            "CtrlDelete",
                            "Char",
                            "Shift",
                            "Ctrl",
                            "Alt",
                            "Esc",
                            "Enter",
                            "Tab",
                            "Backspace",
                            "CtrlBackspace",
                            "None",
                            "F",
                            "Space",
                            "CapsLock",
                            "PgUp",
                            "PgDn",
                            "Home",
                            "End",
                            "PrtScr",
                            "Insert",
                            "LWin",
                            "RWin",
                            "LMB",
                            "RMB",
                            "MMB",
                            "SB1",
                            "SB2",
                        ];
                        if !valid_variants.contains(&method.as_str()) {
                            self.error(
                                *line,
                                format!("Invalid variant '{}' for enum 'Key'", method),
                            );
                        }
                    } else if name == "Color" {
                        if crate::theme::themecore::parse_color(method).is_err() {
                            self.error(*line, format!("Invalid color variant '{}'", method));
                        }
                        if !args.is_empty() {
                            self.error(*line, "Color variant does not take arguments".to_string());
                        }
                    }
                }
            }
            Expr::Not(expr, _) => {
                self.analyze_expr(expr);
            }
            Expr::Binary(left, _, right, _) => {
                self.analyze_expr(left);
                self.analyze_expr(right);
            }
            Expr::Call(name, args, line) => {
                if !self.is_defined(name) && !super::core::BUILTIN_FUNCS.contains(&name.as_str()) {
                    self.error(*line, format!("Undefined function '{}'", name));
                } else if name == "isdown"
                    || name == "isup"
                    || name == "isdownfocus"
                    || name == "isupfocus"
                    || name == "keydown"
                    || name == "keyup"
                {
                    if args.len() != 1 {
                        self.error(*line, format!("'{}' expects exactly 1 argument", name));
                    } else if matches!(args[0].1, Expr::String(_)) {
                        self.error(*line, format!("'{}' expects a Key", name));
                    }
                } else if name == "cursorx" || name == "cursory" {
                    if args.len() != 0 {
                        self.error(*line, format!("'{}' expects exactly 0 arguments", name));
                    }
                } else if name == "setcursor" {
                    if args.len() < 2 || args.len() > 3 {
                        self.error(*line, format!("'{}' expects 2 or 3 arguments", name));
                    }
                }
                for (_, arg) in args {
                    self.analyze_expr(arg);
                }
            }
        }
    }

    fn check_lvalue(&mut self, expr: &Expr, line: usize) {
        match expr {
            Expr::Ident(name, _) => {
                if !self.is_defined(name) {
                    self.error(line, format!("Undefined variable '{}'", name));
                }
            }
            Expr::Index(left, index, _) => {
                self.check_lvalue(left, line);
                self.analyze_expr(index);
            }
            _ => self.error(line, "Invalid assignment target".to_string()),
        }
    }
}
