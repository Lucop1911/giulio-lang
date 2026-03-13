use super::super::super::eval::Evaluator;
use crate::{
    ast::ast::{Ident, Program},
    interpreter::obj::Object,
};
use std::sync::Arc;

impl Evaluator {
    pub fn eval_fn(&mut self, params: Vec<Ident>, body: Program) -> Object {
        Object::Function(params, body, Arc::clone(&self.env))
    }

    pub fn eval_method(&mut self, params: Vec<Ident>, body: Program) -> Object {
        Object::Method(params, body, Arc::clone(&self.env))
    }
}
