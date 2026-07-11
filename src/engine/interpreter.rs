use super::ast::{BinaryOp, Expr, Stmt, StringPart};
use super::value::Value;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::SyncSender;

pub struct Environment {
    pub scopes: Vec<HashMap<String, Value>>,
    pub constants: Vec<HashSet<String>>,
}

impl Environment {
    pub fn new() -> Self {
        Self {
            scopes: vec![HashMap::new()],
            constants: vec![HashSet::new()],
        }
    }

    pub fn push(&mut self) {
        self.scopes.push(HashMap::new());
        self.constants.push(HashSet::new());
    }

    pub fn pop(&mut self) {
        self.scopes.pop();
        self.constants.pop();
    }

    pub fn define(&mut self, name: String, val: Value, is_const: bool) -> Result<(), String> {
        let last_scope = self.scopes.last_mut().unwrap();
        if last_scope.contains_key(&name) {
            return Err(format!("Variable '{}' already defined in this scope", name));
        }
        last_scope.insert(name.clone(), val);
        if is_const {
            self.constants.last_mut().unwrap().insert(name);
        }
        Ok(())
    }

    pub fn assign(&mut self, name: &str, val: Value) -> Result<(), String> {
        for (scope, consts) in self.scopes.iter_mut().zip(self.constants.iter()).rev() {
            if scope.contains_key(name) {
                if consts.contains(name) {
                    return Err(format!("Cannot modify constant '{}'", name));
                }
                scope.insert(name.to_string(), val);
                return Ok(());
            }
        }
        Err(format!("Undefined variable '{}'", name))
    }

    pub fn get(&self, name: &str) -> Option<Value> {
        for scope in self.scopes.iter().rev() {
            if let Some(val) = scope.get(name) {
                return Some(val.clone());
            }
        }
        None
    }
}

pub struct Interpreter {
    pub output: Vec<String>,
    pub errors: Vec<String>,
    current_line: String,
    tx: SyncSender<Vec<String>>,
    pub should_exit: bool,
    pub env: Environment,
    cancel_token: Arc<AtomicBool>,
}

#[derive(PartialEq)]
pub enum Signal {
    None,
    Break,
}

impl Interpreter {
    pub fn new(tx: SyncSender<Vec<String>>, cancel_token: Arc<AtomicBool>) -> Self {
        Self {
            output: Vec::new(),
            errors: Vec::new(),
            current_line: String::new(),
            tx,
            should_exit: false,
            env: Environment::new(),
            cancel_token,
        }
    }

    fn send_output(&mut self) {
        if self.output.len() > 1000 {
            let excess = self.output.len() - 1000;
            self.output.drain(0..excess);
        }

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
        let _ = self.exec_block(stmts);
        self.send_output();
    }

    fn exec_block(&mut self, stmts: &[Stmt]) -> Result<Signal, String> {
        self.env.push();
        let mut res = Ok(Signal::None);

        for stmt in stmts {
            if self.should_exit || self.cancel_token.load(Ordering::SeqCst) {
                self.should_exit = true;
                break;
            }
            match self.execute_stmt(stmt) {
                Ok(Signal::Break) => {
                    res = Ok(Signal::Break);
                    break;
                }
                Ok(Signal::None) => continue,
                Err(err) => {
                    self.errors.push(err);
                    self.should_exit = true;
                    res = Ok(Signal::None);
                    break;
                }
            }
        }

        self.env.pop();
        res
    }

    fn execute_stmt(&mut self, stmt: &Stmt) -> Result<Signal, String> {
        match stmt {
            Stmt::Expr(expr) => {
                self.eval_expr(expr)?;
                Ok(Signal::None)
            }
            Stmt::Let(name, expr, line) => {
                let val = self.eval_expr(expr)?;
                if let Err(e) = self.env.define(name.clone(), val, false) {
                    return Err(format!("Line {}: {}", line, e));
                }
                Ok(Signal::None)
            }
            Stmt::Const(name, expr, line) => {
                let val = self.eval_expr(expr)?;
                if let Err(e) = self.env.define(name.clone(), val, true) {
                    return Err(format!("Line {}: {}", line, e));
                }
                Ok(Signal::None)
            }
            Stmt::Assign(name, expr, line) => {
                let val = self.eval_expr(expr)?;
                if let Err(e) = self.env.assign(name, val) {
                    return Err(format!("Line {}: {}", line, e));
                }
                Ok(Signal::None)
            }
            Stmt::AssignOp(name, op, expr, line) => {
                let right_val = self.eval_expr(expr)?;
                let left_val = self
                    .env
                    .get(name)
                    .ok_or_else(|| format!("Line {}: Undefined variable '{}'", line, name))?;
                let new_val = self.eval_binary_op(&left_val, op, &right_val, *line)?;
                if let Err(e) = self.env.assign(name, new_val) {
                    return Err(format!("Line {}: {}", line, e));
                }
                Ok(Signal::None)
            }
            Stmt::If(cond, then_b, elifs, else_b) => {
                let cond_val = self.eval_expr(cond)?;
                if cond_val.is_truthy() {
                    return self.exec_block(then_b);
                }
                for (elif_cond, elif_b) in elifs {
                    let elif_val = self.eval_expr(elif_cond)?;
                    if elif_val.is_truthy() {
                        return self.exec_block(elif_b);
                    }
                }
                if let Some(e_b) = else_b {
                    return self.exec_block(e_b);
                }
                Ok(Signal::None)
            }
            Stmt::Loop(body) => {
                loop {
                    if self.should_exit || self.cancel_token.load(Ordering::SeqCst) {
                        self.should_exit = true;
                        break;
                    }
                    match self.exec_block(body)? {
                        Signal::Break => break,
                        Signal::None => continue,
                    }
                }
                Ok(Signal::None)
            }
            Stmt::Break(_) => Ok(Signal::Break),
        }
    }

    fn eval_binary_op(
        &mut self,
        left: &Value,
        op: &BinaryOp,
        right: &Value,
        line: usize,
    ) -> Result<Value, String> {
        match (left, right) {
            (Value::Number(ln), Value::Number(rn)) => match op {
                BinaryOp::Add => Ok(Value::Number(ln + rn)),
                BinaryOp::Sub => Ok(Value::Number(ln - rn)),
                BinaryOp::Mul => Ok(Value::Number(ln * rn)),
                BinaryOp::Div => {
                    if *rn == 0.0 {
                        Err(format!("Line {}: Division by zero", line))
                    } else {
                        Ok(Value::Number(ln / rn))
                    }
                }
                BinaryOp::Mod => {
                    if *rn == 0.0 {
                        Err(format!("Line {}: Modulo by zero", line))
                    } else {
                        Ok(Value::Number(ln % rn))
                    }
                }
                BinaryOp::EqEq => Ok(Value::Bool(ln == rn)),
                BinaryOp::NotEq => Ok(Value::Bool(ln != rn)),
                BinaryOp::Less => Ok(Value::Bool(ln < rn)),
                BinaryOp::Greater => Ok(Value::Bool(ln > rn)),
                BinaryOp::LessEq => Ok(Value::Bool(ln <= rn)),
                BinaryOp::GreaterEq => Ok(Value::Bool(ln >= rn)),
            },
            (Value::Bool(lb), Value::Bool(rb)) => match op {
                BinaryOp::EqEq => Ok(Value::Bool(lb == rb)),
                BinaryOp::NotEq => Ok(Value::Bool(lb != rb)),
                _ => Err(format!("Line {}: Unsupported boolean operation", line)),
            },
            (Value::String(ls), right) if *op == BinaryOp::Add => {
                Ok(Value::String(format!("{}{}", ls, right)))
            }
            (left, Value::String(rs)) if *op == BinaryOp::Add => {
                Ok(Value::String(format!("{}{}", left, rs)))
            }
            (Value::String(ls), Value::String(rs)) => match op {
                BinaryOp::EqEq => Ok(Value::Bool(ls == rs)),
                BinaryOp::NotEq => Ok(Value::Bool(ls != rs)),
                _ => Err(format!("Line {}: Unsupported string operation", line)),
            },
            (Value::String(ls), Value::Number(rn)) => match op {
                BinaryOp::Mul => {
                    let count = *rn as i64;
                    if count < 0 {
                        return Err(format!(
                            "Line {}: Cannot multiply string by a negative number",
                            line
                        ));
                    }
                    Ok(Value::String(ls.repeat(count as usize)))
                }
                _ => Err(format!("Line {}: Unsupported string operation", line)),
            },
            (Value::Number(ln), Value::String(rs)) => match op {
                BinaryOp::Mul => {
                    let count = *ln as i64;
                    if count < 0 {
                        return Err(format!(
                            "Line {}: Cannot multiply string by a negative number",
                            line
                        ));
                    }
                    Ok(Value::String(rs.repeat(count as usize)))
                }
                _ => Err(format!("Line {}: Unsupported string operation", line)),
            },
            _ => {
                if *op == BinaryOp::EqEq {
                    Ok(Value::Bool(left == right))
                } else if *op == BinaryOp::NotEq {
                    Ok(Value::Bool(left != right))
                } else {
                    Err(format!("Line {}: Unsupported operands for {:?}", line, op))
                }
            }
        }
    }

    fn eval_expr(&mut self, expr: &Expr) -> Result<Value, String> {
        match expr {
            Expr::Number(n, _) => Ok(Value::Number(*n)),
            Expr::String(s, _) => Ok(Value::String(s.clone())),
            Expr::Bool(b, _) => Ok(Value::Bool(*b)),
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
                if let Some(val) = self.env.get(name) {
                    Ok(val.clone())
                } else {
                    Err(format!("Line {}: Undefined variable '{}'", line, name))
                }
            }
            Expr::Binary(left, op, right, line) => {
                let l = self.eval_expr(left)?;
                let r = self.eval_expr(right)?;
                self.eval_binary_op(&l, op, &r, *line)
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
                        if self.cancel_token.load(Ordering::SeqCst) {
                            self.should_exit = true;
                            return Ok(Value::Nil);
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
