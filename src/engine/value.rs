use std::collections::HashMap;
use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Number(f64),
    String(String),
    Bool(bool),
    List(Vec<Value>),
    Dict(HashMap<Value, Value>),
    Nil,
}

impl Eq for Value {}

impl std::hash::Hash for Value {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        match self {
            Value::Number(n) => {
                state.write_u8(1);
                let bits = if *n == 0.0 {
                    0.0f64.to_bits()
                } else {
                    n.to_bits()
                };
                state.write_u64(bits);
            }
            Value::String(s) => {
                state.write_u8(2);
                s.hash(state);
            }
            Value::Bool(b) => {
                state.write_u8(3);
                b.hash(state);
            }
            Value::List(l) => {
                state.write_u8(4);
                for item in l {
                    item.hash(state);
                }
            }
            Value::Dict(d) => {
                state.write_u8(5);
                state.write_usize(d.len());
            }
            Value::Nil => {
                state.write_u8(6);
            }
        }
    }
}

impl Value {
    pub fn is_truthy(&self) -> bool {
        match self {
            Value::Number(n) => *n != 0.0,
            Value::String(s) => !s.is_empty(),
            Value::Bool(b) => *b,
            Value::List(l) => !l.is_empty(),
            Value::Dict(d) => !d.is_empty(),
            Value::Nil => false,
        }
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::Number(n) => write!(f, "{}", n),
            Value::String(s) => write!(f, "{}", s),
            Value::Bool(b) => write!(f, "{}", if *b { "True" } else { "False" }),
            Value::List(l) => {
                write!(f, "[")?;
                for (i, val) in l.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    if let Value::String(s) = val {
                        write!(f, "\"{}\"", s)?;
                    } else {
                        write!(f, "{}", val)?;
                    }
                }
                write!(f, "]")
            }
            Value::Dict(d) => {
                write!(f, "{{")?;
                for (i, (k, v)) in d.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    if let Value::String(s) = k {
                        write!(f, "\"{}\": ", s)?;
                    } else {
                        write!(f, "{}: ", k)?;
                    }
                    if let Value::String(s) = v {
                        write!(f, "\"{}\"", s)?;
                    } else {
                        write!(f, "{}", v)?;
                    }
                }
                write!(f, "}}")
            }
            Value::Nil => write!(f, "None"),
        }
    }
}
