use super::super::super::eval::Evaluator;
use crate::{
    ast::ast::{Ident, Infix, Literal, Prefix},
    errors::RuntimeError,
    interpreter::{
        env::Environment,
        helpers::{obj_operations, type_converters::obj_to_bool},
        obj::Object,
    },
};
use num_bigint::BigInt;

pub fn eval_ident_sync(env: &mut Environment, ident: &Ident) -> Object {
    if !ident.slot.is_unset() {
        if let Some(obj) = env.get_slot(ident.slot) {
            return obj;
        }
    }
    match env.get_by_name(&ident.name) {
        Some(o) => o,
        None => Object::Error(RuntimeError::UndefinedVariable(ident.name.clone())),
    }
}

pub fn eval_prefix_sync(_: &mut Environment, prefix: &Prefix, object: Object) -> Object {
    match prefix {
        Prefix::Not => match obj_to_bool(object) {
            Ok(b) => Object::Boolean(!b),
            Err(err) => err,
        },
        Prefix::PrefixPlus => match object {
            Object::Integer(_) | Object::BigInteger(_) | Object::Float(_) => object,
            Object::Error(e) => Object::Error(e),
            o => Object::Error(RuntimeError::TypeMismatch {
                expected: "integer".to_string(),
                got: o.type_name(),
            }),
        },
        Prefix::PrefixMinus => match object {
            Object::Integer(i) => match i.checked_neg() {
                Some(result) => Object::Integer(result),
                None => Object::BigInteger(-BigInt::from(i)),
            },
            Object::BigInteger(big) => {
                obj_operations::object_subtract(Object::Integer(0), Object::BigInteger(big))
            }
            Object::Float(f) => Object::Float(-f),
            Object::Error(e) => Object::Error(e),
            o => Object::Error(RuntimeError::TypeMismatch {
                expected: "integer".to_string(),
                got: o.type_name(),
            }),
        },
    }
}

pub fn eval_infix_sync(
    _: &mut Environment,
    infix: &Infix,
    object1: Object,
    object2: Object,
) -> Object {
    match infix {
        Infix::Plus => obj_operations::object_add(object1, object2),
        Infix::Minus => obj_operations::object_subtract(object1, object2),
        Infix::Divide => obj_operations::object_divide(object1, object2),
        Infix::Multiply => obj_operations::object_multiply(object1, object2),
        Infix::Modulo => obj_operations::object_modulo(object1, object2),
        Infix::Equal => Object::Boolean(object1 == object2),
        Infix::NotEqual => Object::Boolean(object1 != object2),
        Infix::GreaterThanEqual => obj_operations::object_compare_gte(object1, object2),
        Infix::GreaterThan => obj_operations::object_compare_gt(object1, object2),
        Infix::LessThanEqual => obj_operations::object_compare_lte(object1, object2),
        Infix::LessThan => obj_operations::object_compare_lt(object1, object2),
        Infix::And => {
            let b1 = obj_to_bool(object1);
            let b2 = obj_to_bool(object2);
            match (b1, b2) {
                (Ok(b1), Ok(b2)) => Object::Boolean(b1 && b2),
                (Err(err), _) | (_, Err(err)) => err,
            }
        }
        Infix::Or => {
            let b1 = obj_to_bool(object1);
            let b2 = obj_to_bool(object2);
            match (b1, b2) {
                (Ok(b1), Ok(b2)) => Object::Boolean(b1 || b2),
                (Err(err), _) | (_, Err(err)) => err,
            }
        }
    }
}

pub fn register_ident_sync(env: &mut Environment, ident: Ident, object: Object) -> Object {
    env.set(&ident, object.clone());
    object
}

impl Evaluator {
    pub fn eval_this(&self) -> Object {
        match self.context.env.lock().unwrap().get_by_name("this") {
            Some(obj) => obj,
            None => Object::Error(RuntimeError::InvalidOperation(
                "'this' can only be used inside a method".to_string(),
            )),
        }
    }

    pub fn eval_ident(&self, ident: Ident) -> Object {
        let borrow_env = self.context.env.lock().unwrap();
        match borrow_env.get(&ident) {
            Some(o) => o,
            None => Object::Error(RuntimeError::UndefinedVariable(ident.name)),
        }
    }

    pub fn eval_literal(&self, literal: &Literal) -> Object {
        match literal {
            Literal::IntLiteral(i) => Object::Integer(*i),
            Literal::BigIntLiteral(big) => Object::BigInteger(big.clone()),
            Literal::FloatLiteral(f) => Object::Float(*f),
            Literal::BoolLiteral(b) => Object::Boolean(*b),
            Literal::StringLiteral(s) => Object::String(s.clone()),
            Literal::NullLiteral => Object::Null,
        }
    }

    pub fn eval_prefix(&self, prefix: Prefix, object: Object) -> Object {
        match prefix {
            Prefix::Not => match obj_to_bool(object) {
                Ok(b) => Object::Boolean(!b),
                Err(err) => err,
            },
            Prefix::PrefixPlus => match object {
                Object::Integer(_) | Object::BigInteger(_) | Object::Float(_) => object,
                Object::Error(e) => Object::Error(e),
                o => Object::Error(RuntimeError::TypeMismatch {
                    expected: "integer".to_string(),
                    got: o.type_name(),
                }),
            },
            Prefix::PrefixMinus => match object {
                Object::Integer(i) => match i.checked_neg() {
                    Some(result) => Object::Integer(result),
                    None => Object::BigInteger(-BigInt::from(i)),
                },
                Object::BigInteger(big) => {
                    obj_operations::object_subtract(Object::Integer(0), Object::BigInteger(big))
                }
                Object::Float(f) => Object::Float(-f),
                Object::Error(e) => Object::Error(e),
                o => Object::Error(RuntimeError::TypeMismatch {
                    expected: "integer".to_string(),
                    got: o.type_name(),
                }),
            },
        }
    }

    pub fn eval_infix(&self, infix: Infix, object1: Object, object2: Object) -> Object {
        match infix {
            Infix::Plus => obj_operations::object_add(object1, object2),
            Infix::Minus => obj_operations::object_subtract(object1, object2),
            Infix::Divide => obj_operations::object_divide(object1, object2),
            Infix::Multiply => obj_operations::object_multiply(object1, object2),
            Infix::Modulo => obj_operations::object_modulo(object1, object2),
            Infix::Equal => Object::Boolean(object1 == object2),
            Infix::NotEqual => Object::Boolean(object1 != object2),
            Infix::GreaterThanEqual => obj_operations::object_compare_gte(object1, object2),
            Infix::GreaterThan => obj_operations::object_compare_gt(object1, object2),
            Infix::LessThanEqual => obj_operations::object_compare_lte(object1, object2),
            Infix::LessThan => obj_operations::object_compare_lt(object1, object2),
            Infix::And => {
                let b1 = obj_to_bool(object1);
                let b2 = obj_to_bool(object2);
                match (b1, b2) {
                    (Ok(b1), Ok(b2)) => Object::Boolean(b1 && b2),
                    (Err(err), _) | (_, Err(err)) => err,
                }
            }
            Infix::Or => {
                let b1 = obj_to_bool(object1);
                let b2 = obj_to_bool(object2);
                match (b1, b2) {
                    (Ok(b1), Ok(b2)) => Object::Boolean(b1 || b2),
                    (Err(err), _) | (_, Err(err)) => err,
                }
            }
        }
    }
}
