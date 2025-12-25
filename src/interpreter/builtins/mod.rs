pub mod array;
pub mod input;
pub mod output;
pub mod string;
pub mod r#type;

use crate::{
    ast::ast::Ident,
    interpreter::obj::{BuiltinFunction, Object},
};
use crate::interpreter::builtins::{array::*, input::*, output::*, r#type::*};


pub struct BuiltinsFunctions;

impl Default for BuiltinsFunctions {
    fn default() -> Self {
        Self::new()
    }
}

impl BuiltinsFunctions {
    pub fn new() -> Self {
        BuiltinsFunctions {}
    }

    pub fn get_builtins(&self) -> Vec<(Ident, Object)> {
        vec![
            add_builtin("print", 1, 1, bprint_fn),
            add_builtin("println", 1, 1, bprintln_fn),
            add_builtin("len", 1, 1, blen_fn),
            add_builtin("head", 1, 1, bhead_fn),
            add_builtin("tail", 1, 1, btail_fn),
            add_builtin("cons", 2, 2, bcons_fn),
            add_builtin("push", 2, 2, bpush_fn),
            add_builtin("type", 1, 1, btype_fn),
            add_builtin("input", 0, 1, binput_fn),
        ]
    }
}

fn add_builtin(
    name: &str,
    min_param: usize,
    max_param: usize,
    func: BuiltinFunction,
) -> (Ident, Object) {
    let name = name.to_owned();
    (
        Ident(name.clone()),
        Object::Builtin(name, min_param, max_param, func),
    )
}
