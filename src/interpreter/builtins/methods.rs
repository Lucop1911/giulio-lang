use crate::interpreter::builtins::impls::struct_ops::bset_field_fn;
use crate::{RuntimeError, interpreter::obj::Object};
use crate::interpreter::builtins::impls::{string::*, array::*, int::*, hash::*, shared::*};

pub struct BuiltinMethods;

impl BuiltinMethods {
    pub fn call_method(object: Object, method_name: &str, args: Vec<Object>) -> Result<Object, RuntimeError> {
        match (&object, method_name) {
            // Conversion methods
            (Object::Integer(_), "to_string") => {
                let mut all_args = vec![object];
                all_args.extend(args);
                btostring_fn(all_args).map_err(|e| RuntimeError::InvalidArguments(e))
            }

            (Object::String(_), "to_int") => {
                let mut all_args = vec![object];
                all_args.extend(args);
                btoint_fn(all_args).map_err(|e| RuntimeError::InvalidArguments(e))
            }

            // Shared methods
            (Object::Array(_) | Object::String(_) | Object::Hash(_), "len") => {
                let mut all_args = vec![object];
                all_args.extend(args);
                blen_fn(all_args).map_err(|e| RuntimeError::InvalidArguments(e))
            }
            (Object::String(_) | Object::Array(_) | Object::Hash(_), "is_empty") => {
                let mut all_args = vec![object];
                all_args.extend(args);
                bisempty_fn(all_args).map_err(|e| RuntimeError::InvalidArguments(e))
            }

            // String methods
            (Object::String(_), "starts_with") => {
                let mut all_args = vec![object];
                all_args.extend(args);
                bstartswith_fn(all_args).map_err(|e| RuntimeError::InvalidArguments(e))
            }
            (Object::String(_), "ends_with") => {
                let mut all_args = vec![object];
                all_args.extend(args);
                bendswith_fn(all_args).map_err(|e| RuntimeError::InvalidArguments(e))
            }
            (Object::String(_), "replace") => {
                let mut all_args = vec![object];
                all_args.extend(args);
                breplace_fn(all_args).map_err(|e| RuntimeError::InvalidArguments(e))
            }
            (Object::String(_), "split") => {
                let mut all_args = vec![object];
                all_args.extend(args);
                bsplit_fn(all_args).map_err(|e| RuntimeError::InvalidArguments(e))
            }
            (Object::String(_), "trim") => {
                let mut all_args = vec![object];
                all_args.extend(args);
                btrim_fn(all_args).map_err(|e| RuntimeError::InvalidArguments(e))
            }
            
            // Array methods
            (Object::Array(_), "head") => {
                let mut all_args = vec![object];
                all_args.extend(args);
                bhead_fn(all_args).map_err(|e| RuntimeError::InvalidArguments(e))
            }
            (Object::Array(_), "tail") => {
                let mut all_args = vec![object];
                all_args.extend(args);
                btail_fn(all_args).map_err(|e| RuntimeError::InvalidArguments(e))
            }
            (Object::Array(_), "push") => {
                let mut all_args = vec![object];
                all_args.extend(args);
                bpush_fn(all_args).map_err(|e| RuntimeError::InvalidArguments(e))
            }

            // Int methods
            (Object::Integer(_), "pow") => {
                let mut all_args = vec![object];
                all_args.extend(args);
                bpow_fn(all_args).map_err(|e| RuntimeError::InvalidArguments(e))
            }
            (Object::Integer(_), "min") => {
                let mut all_args = vec![object];
                all_args.extend(args);
                bmin_fn(all_args).map_err(|e| RuntimeError::InvalidArguments(e))
            }
            (Object::Integer(_), "max") => {
                let mut all_args = vec![object];
                all_args.extend(args);
                bmax_fn(all_args).map_err(|e| RuntimeError::InvalidArguments(e))
            }
            (Object::Integer(_), "abs") => {
                let mut all_args = vec![object];
                all_args.extend(args);
                babs_fn(all_args).map_err(|e| RuntimeError::InvalidArguments(e))
            }

            // Hash methods
            (Object::Hash(_), "get") => {
                let mut all_args = vec![object];
                all_args.extend(args);
                bget_fn(all_args).map_err(|e| RuntimeError::InvalidArguments(e))
            }
            (Object::Hash(_), "set") => {
                let mut all_args = vec![object];
                all_args.extend(args);
                bset_fn(all_args).map_err(|e| RuntimeError::InvalidArguments(e))
            }
            (Object::Hash(_), "remove") => {
                let mut all_args = vec![object];
                all_args.extend(args);
                bremove_fn(all_args).map_err(|e| RuntimeError::InvalidArguments(e))
            }
            (Object::Hash(_), "has") => {
                let mut all_args = vec![object];
                all_args.extend(args);
                bhas_fn(all_args).map_err(|e| RuntimeError::InvalidArguments(e))
            }
            (Object::Hash(_), "keys") => {
                let mut all_args = vec![object];
                all_args.extend(args);
                bkeys_fn(all_args).map_err(|e| RuntimeError::InvalidArguments(e))
            }
            (Object::Hash(_), "values") => {
                let mut all_args = vec![object];
                all_args.extend(args);
                bvalues_fn(all_args).map_err(|e| RuntimeError::InvalidArguments(e))
            }
            (Object::Hash(_), "clear") => {
                let mut all_args = vec![object];
                all_args.extend(args);
                bclear_fn(all_args).map_err(|e| RuntimeError::InvalidArguments(e))
            }

            // Struct methods
            (Object::Struct { name: _, fields: _, methods: _ }, "set") => {
                let mut all_args = vec![object];
                all_args.extend(args);
                bset_field_fn(all_args).map_err(|e| RuntimeError::InvalidArguments(e))
            }

            // Method not found for this type
            _ => Err(RuntimeError::InvalidOperation(
                format!("{} has no method '{}'", object.type_name(), method_name)
            ))
        }
    }
}