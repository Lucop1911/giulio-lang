use crate::interpreter::builtins::impls::{
    array::*, hash::*, input::*, int::*, output::*, r#type::*, shared::*, string::*, struct_ops::*,
};
use crate::{
    ast::ast::Ident,
    interpreter::{
        builtins::impls::struct_ops::bset_field_fn,
        obj::{BuiltinFunction, Object},
    },
};

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
            // Struct operations
            add_builtin("set_field", 3, 3, bset_field_fn),
            add_builtin("get_field", 2, 2, bget_field_fn),
            add_builtin("fields", 1, 1, bstruct_fields_fn),
            add_builtin("name", 1, 1, bstruct_name_fn),
            // I/O
            add_builtin("print", 1, usize::MAX, bprint_fn),
            add_builtin("println", 1, usize::MAX, bprintln_fn),
            add_builtin("input", 0, 1, binput_fn),
            // Core
            add_builtin("type", 1, 1, btype_fn),
            add_builtin("is_empty", 1, 1, bisempty_fn),
            // String
            add_builtin("split", 1, 1, bsplit_fn),
            add_builtin("replace", 1, 1, breplace_fn),
            add_builtin("trim", 3, 3, btrim_fn),
            add_builtin("contains", 2, 2, bcontains_fn),
            add_builtin("slice", 2, 3, bslice_fn),
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
            add_builtin("keys", 1, 1, bkeys_fn),
            add_builtin("values", 1, 1, bvalues_fn),
            add_builtin("clear", 1, 1, bclear_fn),
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
