use crate::{interpreter::obj::Object, RuntimeError};
use num_traits::Zero;

use super::type_converters::{normalize_int, obj_to_float, to_bigint};

fn type_mismatch_error(expected: &str, obj1: Object, obj2: Object) -> Object {
    Object::Error(RuntimeError::TypeMismatch {
        expected: expected.to_string(),
        got: format!("{} and {}", obj1.type_name(), obj2.type_name()),
    })
}

macro_rules! gen_numeric_op {
    ($name:ident, $operator:tt, $is_div:expr) => {
        pub fn $name(obj1: Object, obj2: Object) -> Object {
            if let Object::Error(_) = obj1 { return obj1; }
            if let Object::Error(_) = obj2 { return obj2; }

            if matches!(obj1, Object::Float(_)) || matches!(obj2, Object::Float(_)) {
                let f1 = match obj_to_float(obj1) { Ok(f) => f, Err(e) => return e };
                let f2 = match obj_to_float(obj2) { Ok(f) => f, Err(e) => return e };

                if $is_div && f2 == 0.0 {
                    return Object::Error(RuntimeError::DivisionByZero);
                }
                return Object::Float(f1 $operator f2);
            }

            if let (Some(b1), Some(b2)) = (to_bigint(&obj1), to_bigint(&obj2)) {
                if $is_div && b2.is_zero() {
                    return Object::Error(RuntimeError::DivisionByZero);
                }
                return normalize_int(b1 $operator b2);
            }

            type_mismatch_error("number", obj1, obj2)
        }
    };
}

macro_rules! gen_compare_op {
    ($name:ident, $operator:tt) => {
        pub fn $name(obj1: Object, obj2: Object) -> Object {
            if let Object::Error(_) = obj1 { return obj1; }
            if let Object::Error(_) = obj2 { return obj2; }

            if matches!(obj1, Object::Float(_)) || matches!(obj2, Object::Float(_)) {
                let f1 = match obj_to_float(obj1) { Ok(f) => f, Err(e) => return e };
                let f2 = match obj_to_float(obj2) { Ok(f) => f, Err(e) => return e };
                return Object::Boolean(f1 $operator f2);
            }

            if let (Some(b_int1), Some(b_int2)) = (to_bigint(&obj1), to_bigint(&obj2)) {
                return Object::Boolean(b_int1 $operator b_int2);
            }

            type_mismatch_error("number", obj1, obj2)
        }
    };
}

gen_numeric_op! {object_subtract, -, false}
gen_numeric_op! {object_multiply, *, false}
gen_numeric_op! {object_divide,   /, true}
gen_numeric_op! {object_modulo,   %, true}

gen_compare_op! {object_compare_gt,  >}
gen_compare_op! {object_compare_gte, >=}
gen_compare_op! {object_compare_lt,  <}
gen_compare_op! {object_compare_lte, <=}

pub fn object_add(object1: Object, object2: Object) -> Object {
    if let Object::Error(_) = object1 {
        return object1;
    }
    if let Object::Error(_) = object2 {
        return object2;
    }

    if matches!(object1, Object::Float(_)) || matches!(object2, Object::Float(_)) {
        let f1 = match obj_to_float(object1) {
            Ok(f) => f,
            Err(e) => return e,
        };
        let f2 = match obj_to_float(object2) {
            Ok(f) => f,
            Err(e) => return e,
        };
        return Object::Float(f1 + f2);
    }

    if let (Object::String(s1), Object::String(s2)) = (&object1, &object2) {
        return Object::String(format!("{}{}", s1, s2));
    }

    if let (Some(b_int1), Some(b_int2)) = (to_bigint(&object1), to_bigint(&object2)) {
        return normalize_int(b_int1 + b_int2);
    }

    Object::Error(RuntimeError::InvalidOperation(format!(
        "cannot add {} and {}",
        object1.type_name(),
        object2.type_name()
    )))
}
