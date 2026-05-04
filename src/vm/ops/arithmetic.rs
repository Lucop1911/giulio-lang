//! Arithmetic and comparison operations.
//!
//! All arithmetic logic is centralized here for consistency and maintainability.

use crate::vm::runtime::runtime_errors::RuntimeError;
use crate::vm::runtime::type_converters::{normalize_int, obj_to_float, to_bigint};
use crate::vm::obj::Object;
use num_traits::Zero;

pub fn add(obj1: Object, obj2: Object) -> Object {
    if let Object::Error(_) = obj1 {
        return obj1;
    }
    if let Object::Error(_) = obj2 {
        return obj2;
    }

    match (&obj1, &obj2) {
        (Object::Integer(ia), Object::Integer(ib)) => Object::Integer(ia.wrapping_add(*ib)),
        (Object::Float(fa), Object::Float(fb)) => Object::Float(fa + fb),
        (Object::String(s), Object::String(t)) => Object::String(format!("{}{}", s, t)),
        (Object::String(s), other) => Object::String(format!("{}{}", s, other)),
        (other, Object::String(s)) => Object::String(format!("{}{}", other, s)),
        _ => {
            if matches!(obj1, Object::Float(_)) || matches!(obj2, Object::Float(_)) {
                let f1 = match obj_to_float(obj1) {
                    Ok(f) => f,
                    Err(e) => return e,
                };
                let f2 = match obj_to_float(obj2) {
                    Ok(f) => f,
                    Err(e) => return e,
                };
                return Object::Float(f1 + f2);
            }
            if let (Some(b1), Some(b2)) = (to_bigint(&obj1), to_bigint(&obj2)) {
                return normalize_int(b1 + b2);
            }
            type_mismatch_error("number", obj1, obj2)
        }
    }
}

pub fn subtract(obj1: Object, obj2: Object) -> Object {
    if let Object::Error(_) = obj1 {
        return obj1;
    }
    if let Object::Error(_) = obj2 {
        return obj2;
    }

    match (&obj1, &obj2) {
        (Object::Integer(ia), Object::Integer(ib)) => Object::Integer(ia.wrapping_sub(*ib)),
        (Object::Float(fa), Object::Float(fb)) => Object::Float(fa - fb),
        _ => {
            if matches!(obj1, Object::Float(_)) || matches!(obj2, Object::Float(_)) {
                let f1 = match obj_to_float(obj1) {
                    Ok(f) => f,
                    Err(e) => return e,
                };
                let f2 = match obj_to_float(obj2) {
                    Ok(f) => f,
                    Err(e) => return e,
                };
                return Object::Float(f1 - f2);
            }
            if let (Some(b1), Some(b2)) = (to_bigint(&obj1), to_bigint(&obj2)) {
                return normalize_int(b1 - b2);
            }
            type_mismatch_error("number", obj1, obj2)
        }
    }
}

pub fn multiply(obj1: Object, obj2: Object) -> Object {
    if let Object::Error(_) = obj1 {
        return obj1;
    }
    if let Object::Error(_) = obj2 {
        return obj2;
    }

    match (&obj1, &obj2) {
        (Object::Integer(ia), Object::Integer(ib)) => Object::Integer(ia.wrapping_mul(*ib)),
        (Object::Float(fa), Object::Float(fb)) => Object::Float(fa * fb),
        _ => {
            if matches!(obj1, Object::Float(_)) || matches!(obj2, Object::Float(_)) {
                let f1 = match obj_to_float(obj1) {
                    Ok(f) => f,
                    Err(e) => return e,
                };
                let f2 = match obj_to_float(obj2) {
                    Ok(f) => f,
                    Err(e) => return e,
                };
                return Object::Float(f1 * f2);
            }
            if let (Some(b1), Some(b2)) = (to_bigint(&obj1), to_bigint(&obj2)) {
                return normalize_int(b1 * b2);
            }
            type_mismatch_error("number", obj1, obj2)
        }
    }
}

pub fn divide(obj1: Object, obj2: Object) -> Object {
    if let Object::Error(_) = obj1 {
        return obj1;
    }
    if let Object::Error(_) = obj2 {
        return obj2;
    }

    match (&obj1, &obj2) {
        (Object::Integer(ia), Object::Integer(ib)) => {
            if *ib == 0 {
                Object::Error(RuntimeError::DivisionByZero)
            } else {
                Object::Integer(ia / ib)
            }
        }
        (Object::Float(fa), Object::Float(fb)) => {
            if *fb == 0.0 {
                Object::Error(RuntimeError::DivisionByZero)
            } else {
                Object::Float(fa / fb)
            }
        }
        _ => {
            if matches!(obj1, Object::Float(_)) || matches!(obj2, Object::Float(_)) {
                let f1 = match obj_to_float(obj1) {
                    Ok(f) => f,
                    Err(e) => return e,
                };
                let f2 = match obj_to_float(obj2) {
                    Ok(f) => f,
                    Err(e) => return e,
                };
                if f2 == 0.0 {
                    return Object::Error(RuntimeError::DivisionByZero);
                }
                return Object::Float(f1 / f2);
            }
            if let (Some(b1), Some(b2)) = (to_bigint(&obj1), to_bigint(&obj2)) {
                if b2.is_zero() {
                    return Object::Error(RuntimeError::DivisionByZero);
                }
                return normalize_int(b1 / b2);
            }
            type_mismatch_error("number", obj1, obj2)
        }
    }
}

pub fn modulo(obj1: Object, obj2: Object) -> Object {
    if let Object::Error(_) = obj1 {
        return obj1;
    }
    if let Object::Error(_) = obj2 {
        return obj2;
    }

    match (&obj1, &obj2) {
        (Object::Integer(ia), Object::Integer(ib)) => {
            if *ib == 0 {
                Object::Error(RuntimeError::DivisionByZero)
            } else {
                Object::Integer(ia % ib)
            }
        }
        _ => {
            if matches!(obj1, Object::Float(_)) || matches!(obj2, Object::Float(_)) {
                let f1 = match obj_to_float(obj1) {
                    Ok(f) => f,
                    Err(e) => return e,
                };
                let f2 = match obj_to_float(obj2) {
                    Ok(f) => f,
                    Err(e) => return e,
                };
                if f2 == 0.0 {
                    return Object::Error(RuntimeError::DivisionByZero);
                }
                return Object::Float(f1 % f2);
            }
            if let (Some(b1), Some(b2)) = (to_bigint(&obj1), to_bigint(&obj2)) {
                if b2.is_zero() {
                    return Object::Error(RuntimeError::DivisionByZero);
                }
                return normalize_int(b1 % b2);
            }
            type_mismatch_error("number", obj1, obj2)
        }
    }
}

pub fn less_than(obj1: Object, obj2: Object) -> Object {
    if let Object::Error(_) = obj1 {
        return obj1;
    }
    if let Object::Error(_) = obj2 {
        return obj2;
    }

    match (&obj1, &obj2) {
        (Object::Integer(ia), Object::Integer(ib)) => Object::Boolean(ia < ib),
        (Object::Float(fa), Object::Float(fb)) => Object::Boolean(fa < fb),
        _ => {
            if matches!(obj1, Object::Float(_)) || matches!(obj2, Object::Float(_)) {
                let f1 = match obj_to_float(obj1) {
                    Ok(f) => f,
                    Err(e) => return e,
                };
                let f2 = match obj_to_float(obj2) {
                    Ok(f) => f,
                    Err(e) => return e,
                };
                return Object::Boolean(f1 < f2);
            }
            if let (Some(b1), Some(b2)) = (to_bigint(&obj1), to_bigint(&obj2)) {
                return Object::Boolean(b1 < b2);
            }
            type_mismatch_error("number", obj1, obj2)
        }
    }
}

pub fn greater_than(obj1: Object, obj2: Object) -> Object {
    if let Object::Error(_) = obj1 {
        return obj1;
    }
    if let Object::Error(_) = obj2 {
        return obj2;
    }

    match (&obj1, &obj2) {
        (Object::Integer(ia), Object::Integer(ib)) => Object::Boolean(ia > ib),
        (Object::Float(fa), Object::Float(fb)) => Object::Boolean(fa > fb),
        _ => {
            if matches!(obj1, Object::Float(_)) || matches!(obj2, Object::Float(_)) {
                let f1 = match obj_to_float(obj1) {
                    Ok(f) => f,
                    Err(e) => return e,
                };
                let f2 = match obj_to_float(obj2) {
                    Ok(f) => f,
                    Err(e) => return e,
                };
                return Object::Boolean(f1 > f2);
            }
            if let (Some(b1), Some(b2)) = (to_bigint(&obj1), to_bigint(&obj2)) {
                return Object::Boolean(b1 > b2);
            }
            type_mismatch_error("number", obj1, obj2)
        }
    }
}

pub fn less_equal(obj1: Object, obj2: Object) -> Object {
    if let Object::Error(_) = obj1 {
        return obj1;
    }
    if let Object::Error(_) = obj2 {
        return obj2;
    }

    match (&obj1, &obj2) {
        (Object::Integer(ia), Object::Integer(ib)) => Object::Boolean(ia <= ib),
        (Object::Float(fa), Object::Float(fb)) => Object::Boolean(fa <= fb),
        _ => {
            if matches!(obj1, Object::Float(_)) || matches!(obj2, Object::Float(_)) {
                let f1 = match obj_to_float(obj1) {
                    Ok(f) => f,
                    Err(e) => return e,
                };
                let f2 = match obj_to_float(obj2) {
                    Ok(f) => f,
                    Err(e) => return e,
                };
                return Object::Boolean(f1 <= f2);
            }
            if let (Some(b1), Some(b2)) = (to_bigint(&obj1), to_bigint(&obj2)) {
                return Object::Boolean(b1 <= b2);
            }
            type_mismatch_error("number", obj1, obj2)
        }
    }
}

pub fn greater_equal(obj1: Object, obj2: Object) -> Object {
    if let Object::Error(_) = obj1 {
        return obj1;
    }
    if let Object::Error(_) = obj2 {
        return obj2;
    }

    match (&obj1, &obj2) {
        (Object::Integer(ia), Object::Integer(ib)) => Object::Boolean(ia >= ib),
        (Object::Float(fa), Object::Float(fb)) => Object::Boolean(fa >= fb),
        _ => {
            if matches!(obj1, Object::Float(_)) || matches!(obj2, Object::Float(_)) {
                let f1 = match obj_to_float(obj1) {
                    Ok(f) => f,
                    Err(e) => return e,
                };
                let f2 = match obj_to_float(obj2) {
                    Ok(f) => f,
                    Err(e) => return e,
                };
                return Object::Boolean(f1 >= f2);
            }
            if let (Some(b1), Some(b2)) = (to_bigint(&obj1), to_bigint(&obj2)) {
                return Object::Boolean(b1 >= b2);
            }
            type_mismatch_error("number", obj1, obj2)
        }
    }
}

pub fn execute_equal(obj1: Object, obj2: Object) -> Object {
    Object::Boolean(obj1 == obj2)
}

pub fn execute_not_equal(obj1: Object, obj2: Object) -> Object {
    Object::Boolean(obj1 != obj2)
}

pub fn execute_not(obj: Object) -> Object {
    Object::Boolean(!is_truthy(&obj))
}

pub fn execute_negate(obj: Object) -> Object {
    match obj {
        Object::Integer(i) => Object::Integer(i.wrapping_neg()),
        Object::Float(f) => Object::Float(-f),
        Object::BigInteger(b) => Object::BigInteger(Box::new(-*b)),
        other => Object::Error(RuntimeError::InvalidOperation(format!(
            "Negate not supported for {}",
            other.type_name()
        ))),
    }
}

pub fn is_truthy(obj: &Object) -> bool {
    match obj {
        Object::Boolean(b) => *b,
        Object::Null => false,
        Object::Integer(i) => *i != 0,
        Object::Float(f) => *f != 0.0,
        Object::String(s) => !s.is_empty(),
        Object::Array(a) => !a.is_empty(),
        Object::Hash(h) => !h.is_empty(),
        _ => true,
    }
}

fn type_mismatch_error(expected: &str, obj1: Object, obj2: Object) -> Object {
    Object::Error(RuntimeError::TypeMismatch {
        expected: expected.to_string(),
        got: format!("{} and {}", obj1.type_name(), obj2.type_name()),
    })
}
