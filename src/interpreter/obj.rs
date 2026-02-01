use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt;
use std::hash::{Hash, Hasher};
use std::rc::Rc;

use num_bigint::BigInt;

use crate::ast::ast::{Ident, Program};
use crate::errors::RuntimeError;
use crate::interpreter::env::Environment;

#[derive(Debug, Clone)]
pub enum Object {
    Integer(i64),
    BigInteger(BigInt),
    Float(f64),
    Boolean(bool),
    String(String),
    Array(Vec<Object>),
    Hash(HashMap<Object, Object>),
    Function(Vec<Ident>, Program, Rc<RefCell<Environment>>),
    Builtin(String, usize, usize, BuiltinFunction),
    BuiltinStd(String, usize, usize, StdFunction),
    Struct {
        name: String,
        fields: HashMap<String, Object>,
        methods: HashMap<String, Object>,
    },
    Null,
    ReturnValue(Box<Object>),
    Error(RuntimeError),
    Method(Vec<Ident>, Program, Rc<RefCell<Environment>>),
    Break,
    Continue,
}

pub type BuiltinFunction = fn(Vec<Object>) -> Result<Object, String>;
pub type StdFunction = fn(Vec<Object>) -> Result<Object, RuntimeError>;

impl PartialEq for Object {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Object::Integer(a), Object::Integer(b)) => a == b,
            (Object::BigInteger(a), Object::BigInteger(b)) => a == b,
            (Object::Float(a), Object::Float(b)) => a == b,
            (Object::Boolean(a), Object::Boolean(b)) => a == b,
            (Object::String(a), Object::String(b)) => a == b,
            (Object::Array(a), Object::Array(b)) => a == b,
            (Object::Hash(a), Object::Hash(b)) => a == b,
            (Object::Null, Object::Null) => true,
            (Object::ReturnValue(a), Object::ReturnValue(b)) => a == b,
            (Object::Error(a), Object::Error(b)) => a == b,
            (Object::Builtin(name_a, params_a, params_a1, _), Object::Builtin(name_b, params_b, params_b1, _)) => {
                name_a == name_b && params_a == params_b && params_a1 == params_b1
            }
            (Object::BuiltinStd(name_a, params_a, params_a1, _), Object::BuiltinStd(name_b, params_b, params_b1, _)) => {
                name_a == name_b && params_a == params_b && params_a1 == params_b1
            }
            (Object::Function(params_a, body_a, _), Object::Function(params_b, body_b, _)) => {
                params_a == params_b && body_a == body_b
            }
            (Object::Break, Object::Break) => true,
            (Object::Continue, Object::Continue) => true,
            _ => false,
        }
    }
}

impl Object {
    pub fn is_returned(&self) -> bool {
        matches!(*self, Object::ReturnValue(_))
    }

    pub fn returned(self) -> Self {
        match self {
            Object::ReturnValue(o) => *o,
            o => o,
        }
    }

    pub fn type_name(&self) -> String {
        match self {
            Object::Integer(_) => "integer".to_string(),
            Object::BigInteger(_) => "bigInteger".to_string(),
            Object::Float(_) => "float".to_string(),
            Object::Boolean(_) => "boolean".to_string(),
            Object::String(_) => "string".to_string(),
            Object::Array(_) => "array".to_string(),
            Object::Hash(_) => "hash".to_string(),
            Object::Function(_, _, _) => "function".to_string(),
            Object::Builtin(_, _, _, _) => "builtin function".to_string(),
            Object::BuiltinStd(_, _, _, _) => "builtin function".to_string(),
            Object::Null => "null".to_string(),
            Object::ReturnValue(_) => "return value".to_string(),
            Object::Error(_) => "error".to_string(),
            Object::Method(_, _, _) => "method".to_string(),
            Object::Struct { name, .. } => format!("struct {}", name),
            Object::Break => "break".to_string(),
            Object::Continue => "continue".to_string(),
        }
    }
}

impl fmt::Display for Object {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Object::Integer(ref i) => write!(f, "{}", i),
            Object::BigInteger(ref i) => write!(f, "{}", i),
            Object::Float(ref i) => write!(f, "{}", i),
            Object::Boolean(ref b) => {
                if *b {
                    write!(f, "true")
                } else {
                    write!(f, "false")
                }
            }
            Object::String(ref s) => write!(f, "{}", s),
            Object::Array(ref v) => {
                let mut fmt_string = String::new();
                fmt_string.push('[');
                for (i, o) in v.iter().enumerate() {
                    fmt_string.push_str(format!("{}", o).as_str());
                    if i < v.len() - 1 {
                        fmt_string.push_str(", ");
                    }
                }
                fmt_string.push(']');
                write!(f, "{}", fmt_string)
            }
            Object::Hash(ref hashmap) => {
                let mut fmt_string = String::new();
                fmt_string.push('{');
                for (i, (k, v)) in hashmap.iter().enumerate() {
                    fmt_string.push_str(format!("{} : {}", k, v).as_str());
                    if i < hashmap.len() - 1 {
                        fmt_string.push_str(", ");
                    }
                }
                fmt_string.push('}');
                write!(f, "{}", fmt_string)
            }
            Object::Function(_, _, _) => write!(f, "[function]"),
            Object::Builtin(ref name,_, _, _) => write!(f, "[built-in function: {}]", *name),
            Object::BuiltinStd(ref name,_, _, _) => write!(f, "[built-in function: {}]", *name),
            Object::Null => write!(f, "null"),
            Object::ReturnValue(ref o) => write!(f, "{}", *o),
            Object::Error(ref e) => write!(f, "{}", e),
            Object::Method(_, _, _) => write!(f, "[method]"),
            Object::Struct { ref name, ref fields, .. } => {
                write!(f, "{}{{ ", name)?;
                for (i, (field_name, field_value)) in fields.iter().enumerate() {
                    write!(f, "{}: {}", field_name, field_value)?;
                    if i < fields.len() - 1 {
                        write!(f, ", ")?;
                    }
                }
                write!(f, " }}")
            },
            Object::Break => write!(f, "break"),
            Object::Continue => write!(f, "continue"),
        }
    }
}

impl Eq for Object {}

#[allow(clippy::all)]
impl Hash for Object {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match *self {
            Object::Integer(ref i) => i.hash(state),
            Object::BigInteger(ref i) => i.hash(state),
            Object::Boolean(ref b) => b.hash(state),
            Object::String(ref s) => s.hash(state),
            _ => "".hash(state),
        }
    }
}