//! Type conversions between G-lang [`Object`]s and WASM values.
//!
//! This module handles the FFI boundary between the interpreter and
//! the WASM runtime. It provides:
//!
//! - [`WasmType`] — enumeration of core WASM primitive types
//! - [`TypeMapping`] — bidirectional mapping between G-lang type names
//!   (`"Int"`, `"Float"`, `"Bool"`) and WASM types
//! - [`g_to_component_val`] / [`component_val_to_g`] — conversion between
//!   G-lang `Object` and WASM component-model `Val`

use crate::vm::runtime::runtime_errors::RuntimeError;
use crate::vm::obj::Object;
use std::collections::HashMap;
use wasmtime::{ValType, component::Val as ComponentVal};

/// Core WASM primitive types supported by the FFI layer.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum WasmType {
    I32,
    I64,
    F32,
    F64,
}

impl From<ValType> for WasmType {
    fn from(vt: ValType) -> Self {
        match vt {
            ValType::I32 => WasmType::I32,
            ValType::I64 => WasmType::I64,
            ValType::F32 => WasmType::F32,
            ValType::F64 => WasmType::F64,
            _ => WasmType::I32,
        }
    }
}

impl WasmType {
    pub fn from_string(s: &str) -> Option<Self> {
        match s {
            "i32" => Some(WasmType::I32),
            "i64" => Some(WasmType::I64),
            "f32" => Some(WasmType::F32),
            "f64" => Some(WasmType::F64),
            _ => None,
        }
    }

    pub fn to_valtype(&self) -> ValType {
        match self {
            WasmType::I32 => ValType::I32,
            WasmType::I64 => ValType::I64,
            WasmType::F32 => ValType::F32,
            WasmType::F64 => ValType::F64,
        }
    }
}

/// Bidirectional type name mapping between G-lang and WASM.
///
/// Maps G-lang type names (`"Int"`, `"Float"`, `"Bool"`, `"String"`)
/// to their closest WASM equivalents and vice versa. Used when
/// introspecting WASM module signatures to determine how to marshal
/// arguments and return values.
pub struct TypeMapping {
    g_to_wasm: HashMap<String, WasmType>,
    wasm_to_g: HashMap<WasmType, String>,
}

impl TypeMapping {
    pub fn new() -> Self {
        let mut g_to_wasm = HashMap::new();
        let mut wasm_to_g = HashMap::new();

        g_to_wasm.insert("Int".to_string(), WasmType::I32);
        g_to_wasm.insert("Float".to_string(), WasmType::F64);
        g_to_wasm.insert("Bool".to_string(), WasmType::I32);
        g_to_wasm.insert("String".to_string(), WasmType::I32);

        wasm_to_g.insert(WasmType::I32, "Int".to_string());
        wasm_to_g.insert(WasmType::I64, "Int".to_string());
        wasm_to_g.insert(WasmType::F32, "Float".to_string());
        wasm_to_g.insert(WasmType::F64, "Float".to_string());

        TypeMapping {
            g_to_wasm,
            wasm_to_g,
        }
    }

    pub fn get_wasm_type(&self, g_type: &str) -> Option<WasmType> {
        self.g_to_wasm.get(g_type).copied()
    }

    pub fn get_g_type(&self, wasm_type: WasmType) -> Option<String> {
        self.wasm_to_g.get(&wasm_type).cloned()
    }
}

impl Default for TypeMapping {
    fn default() -> Self {
        Self::new()
    }
}

/// Converts a G-lang [`Object`] to a WASM component-model `Val`.
///
/// Supported conversions:
/// - `Object::Integer` → `Val::S32` (truncated to 32-bit)
/// - `Object::Float` → `Val::Float64`
/// - `Object::Boolean` → `Val::S32` (1 or 0)
///
/// All other object types return an error, as WASM has no native
/// representation for arrays, strings, or hash maps.
pub fn g_to_component_val(obj: &Object) -> Result<ComponentVal, RuntimeError> {
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
pub fn component_val_to_g(val: &ComponentVal) -> Result<Object, RuntimeError> {
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
