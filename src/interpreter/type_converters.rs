use crate::{Evaluator, RuntimeError, ast::ast::Literal, interpreter::obj::Object};

impl Evaluator {
    pub fn obj_to_bool(&mut self, object: Object) -> Result<bool, Object> {
        match object {
            Object::Boolean(b) => Ok(b),
            Object::Error(e) => Err(Object::Error(e)),
            o => Err(Object::Error(RuntimeError::TypeMismatch {
                expected: "boolean".to_string(),
                got: o.type_name(),
            })),
        }
    }

    pub fn obj_to_int(&mut self, object: Object) -> Result<i64, Object> {
        match object {
            Object::Integer(i) => Ok(i),
            Object::Error(e) => Err(Object::Error(e)),
            o => Err(Object::Error(RuntimeError::TypeMismatch {
                expected: "integer".to_string(),
                got: o.type_name(),
            })),
        }
    }

    pub fn obj_to_func(&mut self, object: Object) -> Object {
        match object {
            Object::Function(_, _, _) | Object::Builtin(_, _, _, _) => object,
            Object::Error(e) => Object::Error(e),
            o => Object::Error(RuntimeError::NotCallable(o.type_name())),
        }
    }

    pub fn obj_to_hash(&mut self, object: Object) -> Object {
        match object {
            Object::Integer(i) => Object::Integer(i),
            Object::Boolean(b) => Object::Boolean(b),
            Object::String(s) => Object::String(s),
            Object::Error(e) => Object::Error(e),
            x => Object::Error(RuntimeError::NotHashable(x.type_name())),
        }
    }

    pub fn literal_to_hash(&mut self, literal: Literal) -> Object {
        let object = self.eval_literal(literal);
        self.obj_to_hash(object)
    }
}