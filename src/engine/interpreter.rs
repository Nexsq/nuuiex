use super::ast::{BinaryOp, Expr, Stmt, StringPart};
use super::value::Value;
use std::collections::{HashMap, HashSet};

pub struct Environment {
    pub values: HashMap<String, Value>,
    pub constants: HashSet<String>,
}

pub struct Interpreter {
    pub output: Vec<String>,
    pub errors: Vec<String>,
    current_line: String,
    tx: std::sync::mpsc::Sender<Vec<String>>,
    pub should_exit: bool,
    pub env: Environment,
}

impl Interpreter {
    pub fn new(tx: std::sync::mpsc::Sender<Vec<String>>) -> Self {
        Self {
            output: Vec::new(),
            errors: Vec::new(),
            current_line: String::new(),
            tx,
            should_exit: false,
            env: Environment {
                values: HashMap::new(),
                constants: HashSet::new(),
            },
        }
    }

    fn send_output(&mut self) {
        let mut res = self.output.clone();
        if !self.current_line.is_empty() {
            if res.is_empty() {
                res.push(self.current_line.clone());
            } else {
                let last = res.len() - 1;
                res[last].push_str(&self.current_line);
            }
        }
        if !self.errors.is_empty() {
            if !res.is_empty() && !res.last().unwrap().is_empty() {
                res.push("".to_string());
            }
            res.push("--- Runtime Errors ---".to_string());
            res.extend(self.errors.clone());
        }
        if res.is_empty() || (res.len() == 1 && res[0].is_empty()) {
            res.push("Execution finished with no output.".to_string());
        }

        if self.tx.send(res).is_err() {
            self.should_exit = true;
        }
    }

    pub fn exec(&mut self, stmts: &[Stmt]) {
        for stmt in stmts {
            if self.should_exit {
                break;
            }
            if let Err(err) = self.execute_stmt(stmt) {
                self.errors.push(err);
                break;
            }
        }
        self.send_output();
    }

    fn execute_stmt(&mut self, stmt: &Stmt) -> Result<(), String> {
        match stmt {
            Stmt::Expr(expr) => {
                self.eval_expr(expr)?;
                Ok(())
            }
            Stmt::Let(name, expr, line) => {
                let val = self.eval_expr(expr)?;
                if self.env.values.contains_key(name) {
                    return Err(format!(
                        "Line {}: Variable '{}' already defined",
                        line, name
                    ));
                }
                self.env.values.insert(name.clone(), val);
                Ok(())
            }
            Stmt::Const(name, expr, line) => {
                let val = self.eval_expr(expr)?;
                if self.env.values.contains_key(name) {
                    return Err(format!(
                        "Line {}: Variable '{}' already defined",
                        line, name
                    ));
                }
                self.env.values.insert(name.clone(), val);
                self.env.constants.insert(name.clone());
                Ok(())
            }
            Stmt::Assign(name, expr, line) => {
                if self.env.constants.contains(name) {
                    return Err(format!(
                        "Line {}: Cannot reassign constant '{}'",
                        line, name
                    ));
                }
                if !self.env.values.contains_key(name) {
                    return Err(format!("Line {}: Undefined variable '{}'", line, name));
                }
                let val = self.eval_expr(expr)?;
                self.env.values.insert(name.clone(), val);
                Ok(())
            }
        }
    }

    fn eval_expr(&mut self, expr: &Expr) -> Result<Value, String> {
        match expr {
            Expr::Number(n, _) => Ok(Value::Number(*n)),
            Expr::String(s, _) => Ok(Value::String(s.clone())),
            Expr::FormatString(parts, _) => {
                let mut result = String::new();
                for part in parts {
                    match part {
                        StringPart::Text(t) => result.push_str(t),
                        StringPart::Expr(e) => {
                            let val = self.eval_expr(e)?;
                            result.push_str(&val.to_string());
                        }
                    }
                }
                Ok(Value::String(result))
            }
            Expr::Ident(name, line) => {
                if let Some(val) = self.env.values.get(name) {
                    Ok(val.clone())
                } else {
                    Err(format!("Line {}: Undefined variable '{}'", line, name))
                }
            }
            Expr::Binary(left, op, right, line) => {
                let l = self.eval_expr(left)?;
                let r = self.eval_expr(right)?;

                match (l, r) {
                    (Value::Number(ln), Value::Number(rn)) => match op {
                        BinaryOp::Add => Ok(Value::Number(ln + rn)),
                        BinaryOp::Sub => Ok(Value::Number(ln - rn)),
                        BinaryOp::Mul => Ok(Value::Number(ln * rn)),
                        BinaryOp::Div => {
                            if rn == 0.0 {
                                Err(format!("Line {}: Division by zero", line))
                            } else {
                                Ok(Value::Number(ln / rn))
                            }
                        }
                    },
                    (Value::String(ls), Value::String(rs)) if *op == BinaryOp::Add => {
                        Ok(Value::String(format!("{}{}", ls, rs)))
                    }
                    (Value::String(ls), Value::Number(rn)) if *op == BinaryOp::Add => {
                        Ok(Value::String(format!("{}{}", ls, rn)))
                    }
                    (Value::Number(ln), Value::String(rs)) if *op == BinaryOp::Add => {
                        Ok(Value::String(format!("{}{}", ln, rs)))
                    }
                    (Value::String(ls), Value::Number(rn)) if *op == BinaryOp::Mul => {
                        let count = rn as i64;
                        if count < 0 {
                            return Err(format!(
                                "Line {}: Cannot multiply string by a negative number",
                                line
                            ));
                        }
                        Ok(Value::String(ls.repeat(count as usize)))
                    }
                    (Value::Number(ln), Value::String(rs)) if *op == BinaryOp::Mul => {
                        let count = ln as i64;
                        if count < 0 {
                            return Err(format!(
                                "Line {}: Cannot multiply string by a negative number",
                                line
                            ));
                        }
                        Ok(Value::String(rs.repeat(count as usize)))
                    }
                    _ => Err(format!("Line {}: Unsupported operands for {:?}", line, op)),
                }
            }
            Expr::Call(name, args, line) => {
                let mut eval_args = Vec::with_capacity(args.len());
                for arg in args {
                    eval_args.push(self.eval_expr(arg)?);
                }

                if name == "sleep" {
                    if eval_args.len() != 1 {
                        return Err(format!("Line {}: 'sleep' expects 1 argument", line));
                    }
                    if let Value::Number(ms) = eval_args[0] {
                        if ms < 0.0 {
                            return Err(format!("Line {}: 'sleep' time cannot be negative", line));
                        }
                        std::thread::sleep(std::time::Duration::from_millis(ms as u64));
                        Ok(Value::Nil)
                    } else {
                        Err(format!("Line {}: 'sleep' expects a number", line))
                    }
                } else if name == "exit" {
                    if eval_args.len() != 0 {
                        return Err(format!("Line {}: 'exit' expects 0 arguments", line));
                    }
                    self.should_exit = true;
                    Ok(Value::Nil)
                } else if name == "print" || name == "println" {
                    let mut combined = String::new();
                    for (i, arg) in eval_args.iter().enumerate() {
                        if i > 0 {
                            combined.push(' ');
                        }
                        combined.push_str(&arg.to_string());
                    }

                    if name == "println" {
                        combined.push('\n');
                    }

                    let segments: Vec<&str> = combined.split('\n').collect();
                    for (i, segment) in segments.iter().enumerate() {
                        if i == 0 {
                            self.current_line.push_str(segment);
                        } else {
                            self.output.push(self.current_line.clone());
                            self.current_line = segment.to_string();
                        }
                    }

                    self.send_output();
                    Ok(Value::Nil)
                } else {
                    Err(format!("Line {}: Undefined function '{}'", line, name))
                }
            }
        }
    }
}
