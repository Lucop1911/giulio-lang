use num_bigint::BigInt;
use num_traits::ToPrimitive;

use crate::{runtime::obj::Object, RuntimeError};

pub fn obj_to_bool(object: Object) -> Result<bool, Object> {
    match object {
        Object::Boolean(b) => Ok(b),
        Object::Error(e) => Err(Object::Error(e)),
        o => Err(Object::Error(RuntimeError::TypeMismatch {
            expected: "boolean".to_string(),
            got: o.type_name(),
        })),
    }
}

pub fn obj_to_int(object: Object) -> Result<i64, Object> {
    match object {
        Object::Integer(i) => Ok(i),
        Object::BigInteger(big) => match big.to_i64() {
            Some(i) => Ok(i),
            None => Err(Object::Error(RuntimeError::InvalidOperation(
                "Integer too large to convert to i64".to_string(),
            ))),
        },
        Object::Error(e) => Err(Object::Error(e)),
        o => Err(Object::Error(RuntimeError::TypeMismatch {
            expected: "integer".to_string(),
            got: o.type_name(),
        })),
    }
}

pub fn obj_to_float(object: Object) -> Result<f64, Object> {
    match object {
        Object::Float(f) => Ok(f),
        Object::Integer(i) => Ok(i as f64),
        Object::BigInteger(big) => big.to_f64().ok_or_else(|| {
            Object::Error(RuntimeError::InvalidOperation(
                "BigInt too large for float".into(),
            ))
        }),
        Object::Error(e) => Err(Object::Error(e)),
        o => Err(Object::Error(RuntimeError::TypeMismatch {
            expected: "numeric".into(),
            got: o.type_name(),
        })),
    }
}

pub fn obj_to_func(object: Object) -> Object {
    match object {
        Object::Function(..)
        | Object::AsyncFunction(..)
        | Object::Builtin(..)
        | Object::BuiltinStd(..)
        | Object::WasmImportedFunction { .. } => object,
        Object::Error(e) => Object::Error(e),
        o => Object::Error(RuntimeError::NotCallable(o.type_name())),
    }
}

pub fn obj_to_hash(object: Object) -> Object {
    match object {
        Object::Integer(i) => Object::Integer(i),
        Object::BigInteger(big) => Object::BigInteger(big),
        Object::Boolean(b) => Object::Boolean(b),
        Object::String(s) => Object::String(s),
        Object::Error(e) => Object::Error(e),
        x => Object::Error(RuntimeError::NotHashable(x.type_name())),
    }
}

pub fn to_bigint(obj: &Object) -> Option<BigInt> {
    match obj {
        Object::Integer(i) => Some(BigInt::from(*i)),
        Object::BigInteger(big) => Some(big.clone()),
        _ => None,
    }
}

pub fn normalize_int(big: BigInt) -> Object {
    match big.to_i64() {
        Some(i) => Object::Integer(i),
        None => Object::BigInteger(big),
    }
}
