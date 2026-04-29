use num_bigint::BigInt;
use num_traits::ToPrimitive;

use crate::vm::obj::Object;
use crate::vm::runtime::runtime_errors::RuntimeError;

pub(crate) fn obj_to_float(object: Object) -> Result<f64, Object> {
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

pub(crate) fn to_bigint(obj: &Object) -> Option<BigInt> {
    match obj {
        Object::Integer(i) => Some(BigInt::from(*i)),
        Object::BigInteger(big) => Some(big.clone()),
        _ => None,
    }
}

pub(crate) fn normalize_int(big: BigInt) -> Object {
    match big.to_i64() {
        Some(i) => Object::Integer(i),
        None => Object::BigInteger(big),
    }
}