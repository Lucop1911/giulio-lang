//! Type conversions between G-lang [`Object`]s and WASM values.
//!
//! This module handles the FFI boundary between the interpreter and
//! the WASM runtime. It provides:
//!
//! - [`g_to_component_val`] / [`component_val_to_g`] — conversion between
//!   G-lang `Object` and WASM component-model `Val`

use crate::vm::runtime::runtime_errors::RuntimeError;
use crate::vm::obj::Object;
use wasmtime::component::Val as ComponentVal;

/// Converts a G-lang [`Object`] to a WASM component-model `Val`.
///
/// Supported conversions:
/// - `Object::Integer` → `Val::S32` (truncated to 32-bit)
/// - `Object::Float` → `Val::Float64`
/// - `Object::Boolean` → `Val::S32` (1 or 0)
///
/// All other object types return an error, as WASM has no native
/// representation for arrays, strings, or hash maps.
pub(crate) fn g_to_component_val(obj: &Object) -> Result<ComponentVal, RuntimeError> {
    match obj {
        Object::Integer(n) => Ok(ComponentVal::S32(*n as i32)),
        Object::Float(n) => Ok(ComponentVal::Float64(*n)),
        Object::Boolean(b) => Ok(ComponentVal::S32(if *b { 1 } else { 0 })),
        _ => Err(RuntimeError::InvalidOperation(format!(
            "Cannot convert {:?} to wasm component value",
            obj
        ))),
    }
}

/// Converts a WASM component-model `Val` back to a G-lang [`Object`].
///
/// All integer variants (signed/unsigned, 32/64-bit) are unified into
/// `Object::Integer`. Float variants map to `Object::Float`.
pub(crate) fn component_val_to_g(val: &ComponentVal) -> Result<Object, RuntimeError> {
    match val {
        ComponentVal::S32(n) => Ok(Object::Integer(*n as i64)),
        ComponentVal::U32(n) => Ok(Object::Integer(*n as i64)),
        ComponentVal::S64(n) => Ok(Object::Integer(*n)),
        ComponentVal::U64(n) => Ok(Object::Integer(*n as i64)),
        ComponentVal::Float32(n) => Ok(Object::Float(*n as f64)),
        ComponentVal::Float64(n) => Ok(Object::Float(*n)),
        ComponentVal::Bool(b) => Ok(Object::Boolean(*b)),
        _ => Err(RuntimeError::InvalidOperation(
            "Unsupported wasm value type".to_string(),
        )),
    }
}
