use crate::vm::runtime::builtins::impls::{
    array::*, hash::*, input::*, int::*, output::*, r#type::*, shared::*, string::*, struct_ops::*,
};
use crate::{
    ast::ast::Ident,
    vm::{
        runtime::builtins::impls::struct_ops::bset_field_fn,
        obj::{BuiltinFunction, Object}
    },
};

pub struct BuiltinsFunctions;

impl Default for BuiltinsFunctions {
    fn default() -> Self {
        Self::new()
    }
}

impl BuiltinsFunctions {
    pub const BUILTIN_NAMES: &'static [&'static str] = &[
        "set_field",
        "get_field",
        "fields",
        "name",
        "print",
        "println",
        "input",
        "type",
        "is_empty",
        "split",
        "replace",
        "trim",
        "contains",
        "slice",
        "len",
        "head",
        "tail",
        "cons",
        "push",
        "pow",
        "abs",
        "min",
        "max",
        "keys",
        "values",
        "clear",
    ];

    pub fn new() -> Self {
        BuiltinsFunctions {}
    }

    pub fn get_builtins(&self) -> Vec<(Ident, Object)> {
        vec![
            // Struct operations
            add_builtin(Self::BUILTIN_NAMES[0], 3, 3, bset_field_fn),
            add_builtin(Self::BUILTIN_NAMES[1], 2, 2, bget_field_fn),
            add_builtin(Self::BUILTIN_NAMES[2], 1, 1, bstruct_fields_fn),
            add_builtin(Self::BUILTIN_NAMES[3], 1, 1, bstruct_name_fn),
            // I/O
            add_builtin(Self::BUILTIN_NAMES[4], 1, usize::MAX, bprint_fn),
            add_builtin(Self::BUILTIN_NAMES[5], 1, usize::MAX, bprintln_fn),
            add_builtin(Self::BUILTIN_NAMES[6], 0, 1, binput_fn),
            // Core
            add_builtin(Self::BUILTIN_NAMES[7], 1, 1, btype_fn),
            add_builtin(Self::BUILTIN_NAMES[8], 1, 1, bisempty_fn),
            // String
            add_builtin(Self::BUILTIN_NAMES[9], 1, 1, bsplit_fn),
            add_builtin(Self::BUILTIN_NAMES[10], 1, 1, breplace_fn),
            add_builtin(Self::BUILTIN_NAMES[11], 3, 3, btrim_fn),
            add_builtin(Self::BUILTIN_NAMES[12], 2, 2, bcontains_fn),
            add_builtin(Self::BUILTIN_NAMES[13], 2, 3, bslice_fn),
            // Array
            add_builtin(Self::BUILTIN_NAMES[14], 1, 1, blen_fn),
            add_builtin(Self::BUILTIN_NAMES[15], 1, 1, bhead_fn),
            add_builtin(Self::BUILTIN_NAMES[16], 1, 1, btail_fn),
            add_builtin(Self::BUILTIN_NAMES[17], 2, 2, bcons_fn),
            add_builtin(Self::BUILTIN_NAMES[18], 2, 2, bpush_fn),
            // Int
            add_builtin(Self::BUILTIN_NAMES[19], 2, 2, bpow_fn),
            add_builtin(Self::BUILTIN_NAMES[20], 1, 1, babs_fn),
            add_builtin(Self::BUILTIN_NAMES[21], 2, 2, bmin_fn),
            add_builtin(Self::BUILTIN_NAMES[22], 2, 2, bmax_fn),
            // Hash
            add_builtin(Self::BUILTIN_NAMES[23], 1, 1, bkeys_fn),
            add_builtin(Self::BUILTIN_NAMES[24], 1, 1, bvalues_fn),
            add_builtin(Self::BUILTIN_NAMES[25], 1, 1, bclear_fn),
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
        Ident::new(name.clone()),
        Object::Builtin(name, min_param, max_param, func),
    )
}
