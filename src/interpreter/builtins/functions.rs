use crate::{
    ast::ast::Ident,
    interpreter::{obj::{BuiltinFunction, Object}},
};
use crate::interpreter::builtins::impls::{array::*, input::*, output::*, r#type::*, int::*, shared::*, hash::*};


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
            // I/O
            add_builtin("print", 1, 1, bprint_fn),
            add_builtin("println", 1, 1, bprintln_fn),
            add_builtin("input", 0, 1, binput_fn),
            // Core
            add_builtin("type", 1, 1, btype_fn),
            // Array 
            add_builtin("len", 1, 1, blen_fn),
            add_builtin("head", 1, 1, bhead_fn),
            add_builtin("tail", 1, 1, btail_fn),
            add_builtin("cons", 2, 2, bcons_fn),
            add_builtin("push", 2, 2, bpush_fn),
            // Int
            add_builtin("pow", 2, 2, bpow_fn),
            add_builtin("abs", 1, 1, babs_fn),
            add_builtin("min", 2, 2, bmin_fn),
            add_builtin("max", 2, 2, bmax_fn),
            // Hash
            add_builtin("keys",1 , 1, bkeys_fn),
            add_builtin("values", 1, 1, bvalues_fn),
            add_builtin("has", 2, 2, bhas_fn),
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
