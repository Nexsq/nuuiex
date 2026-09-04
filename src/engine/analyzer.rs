use super::ast::{Expr, Stmt, StringPart};
use std::collections::{HashMap, HashSet};

pub struct Analyzer {
    pub errors: Vec<String>,
    pub error_lines: HashSet<usize>,
    pub scopes: Vec<HashMap<String, bool>>,
    pub loop_depth: usize,
    pub in_function_depth: usize,
    pub in_match_expr_depth: usize,
}

impl Analyzer {
    pub fn new() -> Self {
        let mut root_scope = HashMap::new();
        root_scope.insert("Key".to_string(), true);
        root_scope.insert("Color".to_string(), true);
        root_scope.insert("Background".to_string(), true);
        root_scope.insert("Modifier".to_string(), true);
        root_scope.insert("Image".to_string(), true);
        Self {
            errors: Vec::new(),
            error_lines: HashSet::new(),
            scopes: vec![root_scope],
            loop_depth: 0,
            in_function_depth: 0,
            in_match_expr_depth: 0,
        }
    }

    fn error(&mut self, line: usize, msg: String) {
        self.error_lines.insert(line);
        self.errors.push(format!("Line {}: {}", line, msg));
    }

    fn push_scope(&mut self) {
        self.scopes.push(HashMap::new());
    }

    fn pop_scope(&mut self) {
        self.scopes.pop();
    }

    fn define(&mut self, name: &str, is_const: bool, line: usize) {
        if self.scopes.last().unwrap().contains_key(name) {
            self.error(
                line,
                format!("Variable '{}' is already defined in this scope", name),
            );
        }
        self.scopes
            .last_mut()
            .unwrap()
            .insert(name.to_string(), is_const);
    }

    fn is_defined(&self, name: &str) -> bool {
        for scope in self.scopes.iter().rev() {
            if scope.contains_key(name) {
                return true;
            }
        }
        false
    }

    fn is_const(&self, name: &str) -> bool {
        for scope in self.scopes.iter().rev() {
            if let Some(&is_const) = scope.get(name) {
                return is_const;
            }
        }
        false
    }

    pub fn analyze(&mut self, stmts: &[Stmt]) {
        for stmt in stmts {
            if let Stmt::Fn(name, _, _, line) = stmt {
                self.define(name, true, *line);
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
            Stmt::Let(name, expr, line) => {
                self.analyze_expr(expr);
                self.define(name, false, *line);
            }
            Stmt::Const(name, expr, line) => {
                self.analyze_expr(expr);
                self.define(name, true, *line);
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
            Stmt::Loop(count_expr, body, _) => {
                if let Some(expr) = count_expr {
                    self.analyze_expr(expr);
                }
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
                self.define(name, false, *line);
                self.analyze(body);
                self.pop_scope();
                self.loop_depth -= 1;
            }
            Stmt::Async(body, _) => {
                self.push_scope();
                self.analyze(body);
                self.pop_scope();
            }
            Stmt::Try(try_b, catch_b) => {
                self.push_scope();
                self.analyze(try_b);
                self.pop_scope();

                self.push_scope();
                self.analyze(catch_b);
                self.pop_scope();
            }
            Stmt::Break(line) => {
                if self.in_match_expr_depth > 0 {
                    self.error(
                        *line,
                        "Control flow 'break' is not allowed inside a match expression".into(),
                    );
                } else if self.loop_depth == 0 {
                    self.error(*line, "Break statement outside of a loop".into());
                }
            }
            Stmt::Continue(line) => {
                if self.in_match_expr_depth > 0 {
                    self.error(
                        *line,
                        "Control flow 'continue' is not allowed inside a match expression".into(),
                    );
                } else if self.loop_depth == 0 {
                    self.error(*line, "Continue statement outside of a loop".into());
                }
            }
            Stmt::Pass(_) => {}
            Stmt::Fn(_, params, body, line) => {
                for param in params {
                    if let Some(default) = &param.default {
                        self.analyze_expr(default);
                    }
                }
                self.push_scope();
                self.in_function_depth += 1;
                for param in params {
                    self.define(&param.name, false, *line);
                }
                self.analyze(body);
                self.in_function_depth -= 1;
                self.pop_scope();
            }
            Stmt::Return(expr, line) => {
                if self.in_match_expr_depth > 0 {
                    self.error(
                        *line,
                        "Control flow 'return' is not allowed inside a match expression".into(),
                    );
                } else if self.in_function_depth == 0 {
                    self.error(*line, "Return statement outside of a function".into());
                }
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
                        if !crate::engine::core::is_valid_key_variant(&prop) {
                            self.error(*line, format!("Invalid variant '{}' for enum 'Key'", prop));
                        }
                    } else if name == "Color" || name == "Background" {
                        if crate::theme::themecore::parse_color(prop).is_err() {
                            self.error(*line, format!("Invalid color variant '{}'", prop));
                        }
                    } else if name == "Modifier" {
                        if !crate::engine::core::is_valid_modifier_variant(&prop) {
                            self.error(
                                *line,
                                format!("Invalid variant '{}' for enum 'Modifier'", prop),
                            );
                        }
                    }
                }
            }
            Expr::MethodCall(left, method, args, line) => {
                self.analyze_expr(left);
                for (kw_opt, arg) in args {
                    if kw_opt.is_some() {
                        self.error(
                            *line,
                            "Keyword arguments not supported in method calls".to_string(),
                        );
                    }
                    self.analyze_expr(arg);
                }

                let is_valid_method = matches!(
                    method.as_str(),
                    "clone"
                        | "len"
                        | "append"
                        | "clear"
                        | "count"
                        | "extend"
                        | "index"
                        | "insert"
                        | "pop"
                        | "remove"
                        | "get"
                        | "keys"
                        | "values"
                        | "update"
                        | "set"
                        | "capitalize"
                        | "lower"
                        | "upper"
                        | "swapcase"
                        | "trim"
                        | "join"
                        | "split"
                        | "replace"
                        | "startswith"
                        | "endswith"
                        | "tonum"
                        | "abs"
                        | "neg"
                        | "floor"
                        | "trunc"
                        | "ceil"
                        | "fract"
                        | "clamp"
                        | "round"
                        | "sqrt"
                        | "tostr"
                );

                if !is_valid_method {
                    let mut is_enum = false;
                    if let Expr::Ident(name, _) = &**left {
                        if name == "Key"
                            || name == "Color"
                            || name == "Background"
                            || name == "Modifier"
                            || name == "Image"
                        {
                            is_enum = true;
                        }
                    }

                    if !is_enum {
                        self.error(*line, format!("Unknown method '{}'", method));
                    }
                }

                if let Expr::Ident(name, _) = &**left {
                    if name == "Key" {
                        if !crate::engine::core::is_valid_key_variant(&method) {
                            self.error(
                                *line,
                                format!("Invalid variant '{}' for enum 'Key'", method),
                            );
                        }
                    } else if name == "Color" || name == "Background" {
                        if crate::theme::themecore::parse_color(method).is_err() {
                            self.error(*line, format!("Invalid color variant '{}'", method));
                        }
                    } else if name == "Image" {
                        if !args.is_empty() {
                            self.error(*line, "Image variant does not take arguments".to_string());
                        }
                    } else if name == "Modifier" {
                        if !crate::engine::core::is_valid_modifier_variant(&method) {
                            self.error(
                                *line,
                                format!("Invalid variant '{}' for enum 'Modifier'", method),
                            );
                        }
                        if !args.is_empty() {
                            self.error(
                                *line,
                                "Modifier variant does not take arguments".to_string(),
                            );
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
            Expr::Match(expr, branches, default_branch, _) => {
                self.analyze_expr(expr);
                self.in_match_expr_depth += 1;
                for (pat, body) in branches {
                    self.analyze_expr(pat);
                    self.push_scope();
                    self.analyze(body);
                    self.pop_scope();
                }
                if let Some(body) = default_branch {
                    self.push_scope();
                    self.analyze(body);
                    self.pop_scope();
                }
                self.in_match_expr_depth -= 1;
            }
            Expr::Call(name, args, line) => {
                if !self.is_defined(name) && !super::core::BUILTIN_FUNCS.contains(&name.as_str()) {
                    self.error(*line, format!("Undefined function '{}'", name));
                } else if !self.is_defined(name) {
                    match name.as_str() {
                        "isdown" | "isup" | "isdownfocus" | "isupfocus" | "keydown" | "keyup"
                        | "interrupt" => {
                            if args.len() != 1 {
                                self.error(*line, format!("'{}' expects exactly 1 argument", name));
                            } else if matches!(
                                args[0].1,
                                Expr::String(_)
                                    | Expr::Number(_)
                                    | Expr::Bool(_)
                                    | Expr::List(_)
                                    | Expr::Dict(_)
                                    | Expr::FormatString(_)
                            ) {
                                self.error(*line, format!("'{}' expects a Key", name));
                            }
                        }
                        "sleep" | "sleepaccurate" | "scroll" => {
                            if args.len() != 1 {
                                self.error(*line, format!("'{}' expects exactly 1 argument", name));
                            } else if matches!(
                                args[0].1,
                                Expr::String(_)
                                    | Expr::Bool(_)
                                    | Expr::List(_)
                                    | Expr::Dict(_)
                                    | Expr::FormatString(_)
                            ) {
                                self.error(*line, format!("'{}' expects a Number", name));
                            }
                        }
                        "range" | "random" => {
                            if args.len() < 1 || args.len() > 3 {
                                self.error(*line, format!("'{}' expects 1 to 3 arguments", name));
                            } else {
                                for i in 0..args.len() {
                                    if matches!(
                                        args[i].1,
                                        Expr::String(_)
                                            | Expr::Bool(_)
                                            | Expr::List(_)
                                            | Expr::Dict(_)
                                            | Expr::FormatString(_)
                                    ) {
                                        self.error(*line, format!("'{}' expects Numbers", name));
                                    }
                                }
                            }
                        }
                        "exec" | "write" | "setclipboard" => {
                            if args.len() != 1 {
                                self.error(*line, format!("'{}' expects exactly 1 argument", name));
                            } else if matches!(
                                args[0].1,
                                Expr::Number(_) | Expr::Bool(_) | Expr::List(_) | Expr::Dict(_)
                            ) {
                                self.error(*line, format!("'{}' expects a String", name));
                            }
                        }
                        "input" => {
                            if args.len() > 1 {
                                self.error(*line, format!("'{}' expects 0 or 1 argument", name));
                            } else if args.len() == 1 {
                                if matches!(
                                    args[0].1,
                                    Expr::Number(_) | Expr::Bool(_) | Expr::List(_) | Expr::Dict(_)
                                ) {
                                    self.error(*line, format!("'{}' expects a String", name));
                                }
                            }
                        }
                        "setmouse" => {
                            if args.len() < 2 || args.len() > 3 {
                                self.error(*line, format!("'{}' expects 2 or 3 arguments", name));
                            } else {
                                if matches!(
                                    args[0].1,
                                    Expr::String(_)
                                        | Expr::Bool(_)
                                        | Expr::List(_)
                                        | Expr::Dict(_)
                                        | Expr::FormatString(_)
                                ) {
                                    self.error(*line, format!("'{}' x must be a number", name));
                                }
                                if matches!(
                                    args[1].1,
                                    Expr::String(_)
                                        | Expr::Bool(_)
                                        | Expr::List(_)
                                        | Expr::Dict(_)
                                        | Expr::FormatString(_)
                                ) {
                                    self.error(*line, format!("'{}' y must be a number", name));
                                }
                                if args.len() == 3
                                    && matches!(
                                        args[2].1,
                                        Expr::String(_)
                                            | Expr::Number(_)
                                            | Expr::List(_)
                                            | Expr::Dict(_)
                                            | Expr::FormatString(_)
                                    )
                                {
                                    self.error(
                                        *line,
                                        format!("'{}' relative flag must be a boolean", name),
                                    );
                                }
                            }
                        }
                        "getpixel" | "setcaret" => {
                            if args.len() != 2 {
                                self.error(
                                    *line,
                                    format!("'{}' expects exactly 2 arguments", name),
                                );
                            } else {
                                if matches!(
                                    args[0].1,
                                    Expr::String(_)
                                        | Expr::Bool(_)
                                        | Expr::List(_)
                                        | Expr::Dict(_)
                                        | Expr::FormatString(_)
                                ) {
                                    self.error(*line, format!("'{}' x must be a number", name));
                                }
                                if matches!(
                                    args[1].1,
                                    Expr::String(_)
                                        | Expr::Bool(_)
                                        | Expr::List(_)
                                        | Expr::Dict(_)
                                        | Expr::FormatString(_)
                                ) {
                                    self.error(*line, format!("'{}' y must be a number", name));
                                }
                            }
                        }
                        "imgbase" => {
                            if args.len() != 1 {
                                self.error(*line, format!("'{}' expects exactly 1 argument", name));
                            } else if matches!(
                                args[0].1,
                                Expr::Number(_) | Expr::Bool(_) | Expr::List(_) | Expr::Dict(_)
                            ) {
                                self.error(*line, format!("'{}' expects a String", name));
                            }
                        }
                        "snipbase" => {
                            if args.len() != 4 {
                                self.error(
                                    *line,
                                    format!("'{}' expects exactly 4 arguments", name),
                                );
                            } else {
                                for i in 0..4 {
                                    if matches!(
                                        args[i].1,
                                        Expr::String(_)
                                            | Expr::Bool(_)
                                            | Expr::List(_)
                                            | Expr::Dict(_)
                                            | Expr::FormatString(_)
                                    ) {
                                        self.error(
                                            *line,
                                            format!(
                                                "'{}' expects coordinate arguments to be Numbers",
                                                name
                                            ),
                                        );
                                    }
                                }
                            }
                        }
                        "imgsearch" => {
                            if args.len() < 5 || args.len() > 6 {
                                self.error(*line, format!("'{}' expects 5 or 6 arguments", name));
                            } else {
                                for i in 0..4 {
                                    if matches!(
                                        args[i].1,
                                        Expr::String(_)
                                            | Expr::Bool(_)
                                            | Expr::List(_)
                                            | Expr::Dict(_)
                                            | Expr::FormatString(_)
                                    ) {
                                        self.error(
                                            *line,
                                            format!(
                                                "'{}' expects coordinate arguments to be Numbers",
                                                name
                                            ),
                                        );
                                    }
                                }
                                if matches!(
                                    args[4].1,
                                    Expr::String(_)
                                        | Expr::Number(_)
                                        | Expr::Bool(_)
                                        | Expr::List(_)
                                        | Expr::Dict(_)
                                        | Expr::FormatString(_)
                                ) {
                                    self.error(
                                        *line,
                                        format!("'{}' expects an Image enum for argument 5", name),
                                    );
                                }
                                if args.len() == 6 {
                                    if matches!(
                                        args[5].1,
                                        Expr::String(_)
                                            | Expr::Bool(_)
                                            | Expr::List(_)
                                            | Expr::Dict(_)
                                            | Expr::FormatString(_)
                                    ) {
                                        self.error(
                                            *line,
                                            format!("'{}' expects tolerance to be a Number", name),
                                        );
                                    }
                                }
                            }
                        }
                        "pixelsearch" => {
                            if args.len() < 5 || args.len() > 6 {
                                self.error(*line, format!("'{}' expects 5 or 6 arguments", name));
                            } else {
                                for i in 0..4 {
                                    if matches!(
                                        args[i].1,
                                        Expr::String(_)
                                            | Expr::Bool(_)
                                            | Expr::List(_)
                                            | Expr::Dict(_)
                                            | Expr::FormatString(_)
                                    ) {
                                        self.error(
                                            *line,
                                            format!(
                                                "'{}' expects coordinate arguments to be Numbers",
                                                name
                                            ),
                                        );
                                    }
                                }
                                if matches!(
                                    args[4].1,
                                    Expr::String(_)
                                        | Expr::Number(_)
                                        | Expr::Bool(_)
                                        | Expr::List(_)
                                        | Expr::Dict(_)
                                        | Expr::FormatString(_)
                                ) {
                                    self.error(
                                        *line,
                                        format!("'{}' expects a Color enum for argument 5", name),
                                    );
                                }
                                if args.len() == 6 {
                                    if matches!(
                                        args[5].1,
                                        Expr::String(_)
                                            | Expr::Bool(_)
                                            | Expr::List(_)
                                            | Expr::Dict(_)
                                            | Expr::FormatString(_)
                                    ) {
                                        self.error(
                                            *line,
                                            format!("'{}' expects tolerance to be a Number", name),
                                        );
                                    }
                                }
                            }
                        }
                        "comcolor" => {
                            if args.len() < 2 || args.len() > 3 {
                                self.error(*line, format!("'{}' expects 2 or 3 arguments", name));
                            } else {
                                for i in 0..2 {
                                    if matches!(
                                        args[i].1,
                                        Expr::String(_)
                                            | Expr::Number(_)
                                            | Expr::Bool(_)
                                            | Expr::List(_)
                                            | Expr::Dict(_)
                                            | Expr::FormatString(_)
                                    ) {
                                        self.error(
                                            *line,
                                            format!(
                                                "'{}' expects a Color enum for argument {}",
                                                name,
                                                i + 1
                                            ),
                                        );
                                    }
                                }
                                if args.len() == 3 {
                                    if matches!(
                                        args[2].1,
                                        Expr::String(_)
                                            | Expr::Bool(_)
                                            | Expr::List(_)
                                            | Expr::Dict(_)
                                            | Expr::FormatString(_)
                                    ) {
                                        self.error(
                                            *line,
                                            format!(
                                                "'{}' expects a Number for the tolerance argument",
                                                name
                                            ),
                                        );
                                    }
                                }
                            }
                        }
                        "activekeys" => {
                            if args.len() != 1 {
                                self.error(*line, format!("'{}' expects exactly 1 argument", name));
                            } else if matches!(
                                args[0].1,
                                Expr::String(_)
                                    | Expr::Number(_)
                                    | Expr::Bool(_)
                                    | Expr::Dict(_)
                                    | Expr::FormatString(_)
                            ) {
                                self.error(*line, format!("'{}' expects a List", name));
                            }
                        }
                        "clear" => {
                            if args.len() > 1 {
                                self.error(*line, format!("'{}' expects 0 or 1 argument", name));
                            } else if args.len() == 1 {
                                if matches!(
                                    args[0].1,
                                    Expr::String(_)
                                        | Expr::Number(_)
                                        | Expr::List(_)
                                        | Expr::Dict(_)
                                        | Expr::FormatString(_)
                                ) {
                                    self.error(*line, format!("'{}' expects a Boolean", name));
                                }
                            }
                        }
                        "macrodata" => {
                            if args.len() > 1 {
                                self.error(*line, format!("'{}' expects 0 or 1 argument", name));
                            } else if args.len() == 1 {
                                if matches!(
                                    args[0].1,
                                    Expr::String(_)
                                        | Expr::Number(_)
                                        | Expr::Bool(_)
                                        | Expr::List(_)
                                        | Expr::FormatString(_)
                                ) {
                                    self.error(*line, format!("'{}' expects a Dictionary", name));
                                }
                            }
                        }
                        "keypress" => {
                            if args.len() < 1 || args.len() > 2 {
                                self.error(*line, format!("'{}' expects 1 or 2 arguments", name));
                            } else {
                                if matches!(
                                    args[0].1,
                                    Expr::String(_)
                                        | Expr::Number(_)
                                        | Expr::Bool(_)
                                        | Expr::List(_)
                                        | Expr::Dict(_)
                                        | Expr::FormatString(_)
                                ) {
                                    self.error(
                                        *line,
                                        format!("'{}' expects a Key for argument 1", name),
                                    );
                                }
                                if args.len() == 2
                                    && matches!(
                                        args[1].1,
                                        Expr::String(_)
                                            | Expr::Bool(_)
                                            | Expr::List(_)
                                            | Expr::Dict(_)
                                            | Expr::FormatString(_)
                                    )
                                {
                                    self.error(
                                        *line,
                                        format!("'{}' expects a Number for argument 2", name),
                                    );
                                }
                            }
                        }
                        "beep" => {
                            if args.len() > 2 {
                                self.error(
                                    *line,
                                    format!("'{}' expects 0, 1, or 2 arguments", name),
                                );
                            } else {
                                for i in 0..args.len() {
                                    if matches!(
                                        args[i].1,
                                        Expr::String(_)
                                            | Expr::Bool(_)
                                            | Expr::List(_)
                                            | Expr::Dict(_)
                                            | Expr::FormatString(_)
                                    ) {
                                        self.error(*line, format!("'{}' expects Numbers", name));
                                    }
                                }
                            }
                        }
                        "mousex" | "mousey" | "mousedelta" | "time" | "caretx" | "carety"
                        | "screenx" | "screeny" | "focused" | "displayx" | "displayy"
                        | "getclipboard" | "onlinux" | "onwindows" | "exit" => {
                            if args.len() != 0 {
                                self.error(
                                    *line,
                                    format!("'{}' expects exactly 0 arguments", name),
                                );
                            }
                        }
                        "len" => {
                            if args.len() != 1 {
                                self.error(*line, format!("'{}' expects exactly 1 argument", name));
                            } else if matches!(args[0].1, Expr::Number(_) | Expr::Bool(_)) {
                                self.error(
                                    *line,
                                    format!("'{}' expects a String, List, or Dictionary", name),
                                );
                            }
                        }
                        "max" | "min" => {
                            if args.is_empty() {
                                self.error(
                                    *line,
                                    format!("'{}' expects at least 1 argument", name),
                                );
                            } else if args.len() == 1 {
                                if matches!(
                                    args[0].1,
                                    Expr::Bool(_) | Expr::Dict(_) | Expr::Number(_)
                                ) {
                                    self.error(*line, format!("'{}' single argument must be an iterable (String, List)", name));
                                }
                            }
                        }
                        _ => {}
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
                } else if self.is_const(name) {
                    self.error(line, format!("Cannot assign to constant '{}'", name));
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
