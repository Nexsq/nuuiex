use super::ast::{BinaryOp, Expr, Stmt};
use super::value::Value;

pub struct Interpreter {
    pub output: Vec<String>,
    pub errors: Vec<String>,
    current_line: String,
}

impl Interpreter {
    pub fn new() -> Self {
        Self {
            output: Vec::new(),
            errors: Vec::new(),
            current_line: String::new(),
        }
    }

    pub fn exec(&mut self, stmts: &[Stmt]) {
        for stmt in stmts {
            if let Err(err) = self.execute_stmt(stmt) {
                self.errors.push(err);
                break;
            }
        }

        if !self.current_line.is_empty() {
            self.output.push(self.current_line.clone());
            self.current_line.clear();
        }
    }

    fn execute_stmt(&mut self, stmt: &Stmt) -> Result<(), String> {
        match stmt {
            Stmt::Expr(expr) => {
                self.eval_expr(expr)?;
                Ok(())
            }
        }
    }

    fn eval_expr(&mut self, expr: &Expr) -> Result<Value, String> {
        match expr {
            Expr::Number(n) => Ok(Value::Number(*n)),
            Expr::String(s) => Ok(Value::String(s.clone())),
            Expr::Ident(name) => Err(format!("Undefined variable '{}'", name)),
            Expr::Binary(left, op, right) => {
                let l = self.eval_expr(left)?;
                let r = self.eval_expr(right)?;

                match (l, r) {
                    (Value::Number(ln), Value::Number(rn)) => match op {
                        BinaryOp::Add => Ok(Value::Number(ln + rn)),
                        BinaryOp::Sub => Ok(Value::Number(ln - rn)),
                        BinaryOp::Mul => Ok(Value::Number(ln * rn)),
                        BinaryOp::Div => {
                            if rn == 0.0 {
                                Err("Division by zero".to_string())
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
                        let count = rn.max(0.0) as usize;
                        Ok(Value::String(ls.repeat(count)))
                    }
                    (Value::Number(ln), Value::String(rs)) if *op == BinaryOp::Mul => {
                        let count = ln.max(0.0) as usize;
                        Ok(Value::String(rs.repeat(count)))
                    }
                    _ => Err(format!("Unsupported operands for {:?}", op)),
                }
            }
            Expr::Call(name, args) => {
                let mut eval_args = Vec::with_capacity(args.len());
                for arg in args {
                    eval_args.push(self.eval_expr(arg)?);
                }

                if *name == "print" || *name == "println" {
                    let mut combined = String::new();
                    for (i, arg) in eval_args.iter().enumerate() {
                        if i > 0 {
                            combined.push(' ');
                        }
                        combined.push_str(&arg.to_string());
                    }

                    if *name == "println" {
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

                    Ok(Value::Nil)
                } else {
                    Err(format!("Undefined function '{}'", name))
                }
            }
        }
    }
}
