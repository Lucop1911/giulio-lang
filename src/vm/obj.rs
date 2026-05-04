//! Runtime value representation for the G-lang interpreter.
//!
//! [`Object`] is the central type that every expression evaluates to.
//! It covers primitives (integers, floats, strings, booleans), collections
//! (arrays, hash maps), first-class functions (sync and async), builtin
//! functions, structs, modules, and control-flow sentinels (Break, Continue,
//! ReturnValue, ThrownValue).

use ahash::AHasher;
use num_bigint::BigInt;
use std::fmt;
use std::hash::{BuildHasherDefault, Hash, Hasher};
use std::sync::{Arc, Mutex};

use crate::ast::ast::Ident;
use crate::vm::runtime::env::Environment;
use crate::vm::runtime::runtime_errors::RuntimeError;

#[cfg(feature = "wasm")]
use crate::wasm::WasmInstance;

pub type HashMap<K, V> = std::collections::HashMap<K, V, BuildHasherDefault<AHasher>>;

/// A struct value with named fields and methods.
/// Boxed to reduce the size of the Object enum.
#[derive(Clone)]
pub struct StructObject {
    pub name: String,
    pub fields: HashMap<String, Object>,
    pub methods: HashMap<String, Object>,
}

/// A loaded module with its exported bindings.
/// Boxed to reduce the size of the Object enum.
#[derive(Clone)]
pub struct ModuleObject {
    pub name: String,
    pub exports: HashMap<String, Object>,
}

/// The universal value type of the G-lang runtime.
///
/// Every expression in a G-lang program evaluates to one of these variants.
/// Large variants are boxed to keep the enum size small (typically 24 bytes
/// on 64-bit systems instead of 48-64 bytes).
#[derive(Clone)]
pub enum Object {
    Integer(i64),
    /// Boxed to reduce enum size since BigInt can be large.
    BigInteger(Box<BigInt>),
    Float(f64),
    Boolean(bool),
    String(String),
    /// Boxed to reduce enum size (Vec is 24 bytes).
    Array(Box<Vec<Object>>),
    /// Boxed to reduce enum size (HashMap is ~48+ bytes).
    Hash(Box<HashMap<Object, Object>>),
    /// User-defined function: (params, chunk, closure_env, local_names)
    Function(
        Vec<Ident>,
        Arc<crate::vm::chunk::Chunk>,
        Arc<Mutex<Environment>>,
        Vec<String>,
    ),
    /// Async user-defined function: (params, chunk, closure_env, local_names)
    AsyncFunction(
        Vec<Ident>,
        Arc<crate::vm::chunk::Chunk>,
        Arc<Mutex<Environment>>,
        Vec<String>,
    ),
    /// Builtin function implemented in Rust (simple variant).
    Builtin(String, usize, usize, BuiltinFunction),
    /// Builtin function with RuntimeError-based error handling.
    BuiltinStd(String, usize, usize, StdFunction),
    /// Async builtin function.
    BuiltinStdAsync(String, usize, usize, AsyncStdFunction),
    /// A function imported from a WASM module.
    WasmImportedFunction {
        module_name: String,
        func_name: String,
        instance: Arc<Mutex<Option<WasmInstance>>>,
    },
    /// A struct value with named fields and methods.
    /// Boxed to reduce enum size (contains two HashMaps).
    Struct(Box<StructObject>),
    /// A loaded module with its exported bindings.
    /// Boxed to reduce enum size (contains a HashMap).
    Module(Box<ModuleObject>),
    Null,
    /// Wraps a value that was returned from a function.
    /// Used as a control-flow sentinel to short-circuit evaluation.
    ReturnValue(Box<Object>),
    Error(RuntimeError),
    /// A method bound to a struct instance.
    Method(
        Vec<Ident>,
        Arc<crate::vm::chunk::Chunk>,
        Arc<Mutex<Environment>>,
        Vec<String>,
    ),
    /// Control-flow sentinel: exits the innermost loop.
    Break,
    /// Control-flow sentinel: skips to the next loop iteration.
    Continue,
    /// Wraps a value passed to `throw`. Propagates up the call stack
    /// until caught by a `try/catch` or reaches the top level.
    ThrownValue(Box<Object>),
    /// An async computation that has not yet been awaited.
    /// The inner future is `Option`-wrapped so that `await` can
    /// take ownership exactly once (double-await is an error).
    Future(
        Arc<
            Mutex<
                Option<
                    std::pin::Pin<
                        Box<
                            dyn std::future::Future<Output = Result<Object, RuntimeError>>
                                + Send
                                + 'static,
                        >,
                    >,
                >,
            >,
        >,
    ),
    #[cfg(feature = "wasm")]
    WasmModule {
        name: String,
        exports: HashMap<String, Object>,
        instance: Arc<Mutex<Option<WasmInstance>>>,
    },
}

pub type BuiltinFunction = fn(Vec<Object>) -> Result<Object, String>;
pub type StdFunction = fn(Vec<Object>) -> Result<Object, RuntimeError>;
pub type AsyncStdFunction = fn(Vec<Object>) -> Result<Object, RuntimeError>;

impl fmt::Debug for Object {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Object::Integer(i) => write!(f, "Integer({})", i),
            Object::BigInteger(b) => write!(f, "BigInteger({})", b),
            Object::Float(fl) => write!(f, "Float({})", fl),
            Object::Boolean(b) => write!(f, "Boolean({})", b),
            Object::String(s) => write!(f, "String(\"{}\")", s),
            Object::Array(a) => write!(f, "Array({:?})", a),
            Object::Hash(h) => write!(f, "Hash({:?})", h),
            Object::Function(p, _, _, _) => write!(f, "Function(params:{:?})", p),
            Object::AsyncFunction(p, _, _, _) => {
                write!(f, "AsyncFunction(params:{:?})", p)
            }
            Object::WasmImportedFunction {
                module_name,
                func_name,
                ..
            } => {
                write!(f, "WasmImportedFunction({}::{})", module_name, func_name)
            }
            Object::Builtin(n, _, _, _) => write!(f, "Builtin(\"{}\")", n),
            Object::BuiltinStd(n, _, _, _) => write!(f, "BuiltinStd(\"{}\")", n),
            Object::BuiltinStdAsync(n, _, _, _) => write!(f, "BuiltinStdAsync(\"{}\")", n),
            Object::Struct(s) => write!(
                f,
                "Struct(name:{}, fields:{:?}, methods:{:?})",
                s.name, s.fields, s.methods
            ),
            Object::Module(m) => write!(
                f,
                "Module(name:{}, exports:{:?})",
                m.name,
                m.exports.keys().collect::<Vec<_>>()
            ),
            Object::Null => write!(f, "Null"),
            Object::ReturnValue(o) => write!(f, "ReturnValue({:?})", o),
            Object::Error(e) => write!(f, "Error({:?})", e),
            Object::Method(p, _, _, _) => write!(f, "Method(params:{:?})", p),
            Object::Break => write!(f, "Break"),
            Object::Continue => write!(f, "Continue"),
            Object::ThrownValue(o) => write!(f, "ThrownValue({:?})", o),
            Object::Future(_) => write!(f, "Future(_)"),
            #[cfg(feature = "wasm")]
            Object::WasmModule { name, exports, .. } => write!(
                f,
                "WasmModule(name:{}, exports:{:?})",
                name,
                exports.keys().collect::<Vec<_>>()
            ),
        }
    }
}

impl PartialEq for Object {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Object::Integer(a), Object::Integer(b)) => a == b,
            (Object::BigInteger(a), Object::BigInteger(b)) => a == b,
            (Object::Float(a), Object::Float(b)) => a == b,
            (Object::Boolean(a), Object::Boolean(b)) => a == b,
            (Object::String(a), Object::String(b)) => a == b,
            (Object::Array(a), Object::Array(b)) => a == b,
            (Object::Hash(a), Object::Hash(b)) => a == b,
            (Object::Null, Object::Null) => true,
            (Object::ReturnValue(a), Object::ReturnValue(b)) => a == b,
            (Object::Error(a), Object::Error(b)) => a == b,
            (Object::ThrownValue(a), Object::ThrownValue(b)) => a == b,
            (
                Object::Builtin(name_a, params_a, params_a1, _),
                Object::Builtin(name_b, params_b, params_b1, _),
            ) => name_a == name_b && params_a == params_b && params_a1 == params_b1,
            (
                Object::BuiltinStd(name_a, params_a, params_a1, _),
                Object::BuiltinStd(name_b, params_b, params_b1, _),
            ) => name_a == name_b && params_a == params_b && params_a1 == params_b1,
            (
                Object::BuiltinStdAsync(name_a, params_a, params_a1, _),
                Object::BuiltinStdAsync(name_b, params_b, params_b1, _),
            ) => name_a == name_b && params_a == params_b && params_a1 == params_b1,
            (
                Object::Function(params_a, chunk_a, _, names_a),
                Object::Function(params_b, chunk_b, _, names_b),
            ) => params_a == params_b && Arc::ptr_eq(chunk_a, chunk_b) && names_a == names_b,
            (
                Object::AsyncFunction(params_a, chunk_a, _, names_a),
                Object::AsyncFunction(params_b, chunk_b, _, names_b),
            ) => params_a == params_b && Arc::ptr_eq(chunk_a, chunk_b) && names_a == names_b,
            (
                Object::WasmImportedFunction {
                    module_name: a,
                    func_name: b,
                    ..
                },
                Object::WasmImportedFunction {
                    module_name: c,
                    func_name: d,
                    ..
                },
            ) => a == c && b == d,
            (Object::Break, Object::Break) => true,
            (Object::Continue, Object::Continue) => true,
            (Object::Future(_), Object::Future(_)) => false,
            (
                Object::Module(a),
                Object::Module(b),
            ) => a.name == b.name && a.exports.keys().collect::<Vec<_>>() == b.exports.keys().collect::<Vec<_>>(),
            (
                Object::Struct(a),
                Object::Struct(b),
            ) => a.name == b.name && a.fields == b.fields && a.methods == b.methods,
            #[cfg(feature = "wasm")]
            (
                Object::WasmModule {
                    name: a,
                    exports: e_a,
                    ..
                },
                Object::WasmModule {
                    name: b,
                    exports: e_b,
                    ..
                },
            ) => a == b && e_a.keys().collect::<Vec<_>>() == e_b.keys().collect::<Vec<_>>(),
            _ => false,
        }
    }
}

impl Object {
    pub fn type_name(&self) -> String {
        match self {
            Object::Integer(_) => "integer".to_string(),
            Object::BigInteger(_) => "bigInteger".to_string(),
            Object::Float(_) => "float".to_string(),
            Object::Boolean(_) => "boolean".to_string(),
            Object::String(_) => "string".to_string(),
            Object::Array(_) => "array".to_string(),
            Object::Hash(_) => "hash".to_string(),
            Object::Function(_, _, _, _) => "function".to_string(),
            Object::AsyncFunction(_, _, _, _) => "async function".to_string(),
            Object::WasmImportedFunction { .. } => "wasm imported function".to_string(),
            Object::Builtin(_, _, _, _) => "builtin function".to_string(),
            Object::BuiltinStd(_, _, _, _) => "builtin function".to_string(),
            Object::BuiltinStdAsync(_, _, _, _) => "async builtin function".to_string(),
            Object::Null => "null".to_string(),
            Object::ReturnValue(_) => "return value".to_string(),
            Object::Error(_) => "error".to_string(),
            Object::Method(_, _, _, _) => "method".to_string(),
            Object::Struct(s) => format!("struct {}", s.name),
            Object::Module(m) => format!("module {}", m.name),
            Object::Break => "break".to_string(),
            Object::Continue => "continue".to_string(),
            Object::ThrownValue(_) => "thrown value".to_string(),
            Object::Future(_) => "future".to_string(),
            #[cfg(feature = "wasm")]
            Object::WasmModule { name, .. } => format!("wasm module {}", name),
        }
    }
}

impl fmt::Display for Object {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Object::Integer(ref i) => write!(f, "{}", i),
            Object::BigInteger(ref i) => write!(f, "{}", i),
            Object::Float(ref i) => write!(f, "{}", i),
            Object::Boolean(ref b) => {
                if *b {
                    write!(f, "true")
                } else {
                    write!(f, "false")
                }
            }
            Object::String(ref s) => write!(f, "{}", s),
            Object::Array(ref v) => {
                let mut fmt_string = String::new();
                fmt_string.push('[');
                for (i, o) in v.iter().enumerate() {
                    fmt_string.push_str(format!("{}", o).as_str());
                    if i < v.len() - 1 {
                        fmt_string.push_str(", ");
                    }
                }
                fmt_string.push(']');
                write!(f, "{}", fmt_string)
            }
            Object::Hash(ref hashmap) => {
                let mut fmt_string = String::new();
                fmt_string.push('{');
                for (i, (k, v)) in hashmap.iter().enumerate() {
                    fmt_string.push_str(format!("{} : {}", k, v).as_str());
                    if i < hashmap.len() - 1 {
                        fmt_string.push_str(", ");
                    }
                }
                fmt_string.push('}');
                write!(f, "{}", fmt_string)
            }
            Object::Function(_, _, _, _) => write!(f, "[function]"),
            Object::AsyncFunction(_, _, _, _) => write!(f, "[async function]"),
            Object::WasmImportedFunction {
                ref module_name,
                ref func_name,
                ..
            } => write!(f, "[wasm function: {}::{}]", module_name, func_name),
            Object::Builtin(ref name, _, _, _) => write!(f, "[built-in function: {}]", *name),
            Object::BuiltinStd(ref name, _, _, _) => write!(f, "[built-in function: {}]", *name),
            Object::BuiltinStdAsync(ref name, _, _, _) => {
                write!(f, "[async built-in function: {}]", *name)
            }
            Object::Null => write!(f, "null"),
            Object::ReturnValue(ref o) => write!(f, "{}", *o),
            Object::Error(ref e) => write!(f, "{}", e),
            Object::Method(_, _, _, _) => write!(f, "[method]"),
            Object::Struct(ref s) => {
                write!(f, "{}{{ ", s.name)?;
                for (i, (field_name, field_value)) in s.fields.iter().enumerate() {
                    write!(f, "{}: {}", field_name, field_value)?;
                    if i < s.fields.len() - 1 {
                        write!(f, ", ")?;
                    }
                }
                write!(f, " }}")
            }
            Object::Break => write!(f, "break"),
            Object::Continue => write!(f, "continue"),
            Object::ThrownValue(ref o) => write!(f, "Thrown: {}", *o),
            Object::Future(_) => write!(f, "[future]"),
            Object::Module(ref m) => write!(f, "[module: {}]", m.name),
            #[cfg(feature = "wasm")]
            Object::WasmModule { ref name, .. } => write!(f, "[wasm module: {}]", name),
        }
    }
}

impl Eq for Object {}

#[allow(clippy::all)]
impl Hash for Object {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match *self {
            Object::Integer(ref i) => i.hash(state),
            Object::BigInteger(ref i) => i.hash(state),
            Object::Boolean(ref b) => b.hash(state),
            Object::String(ref s) => s.hash(state),
            Object::Function(ref params, ref chunk, _, ref names) => {
                params.hash(state);
                chunk.code.hash(state);
                names.hash(state);
            }
            Object::AsyncFunction(ref params, ref chunk, _, ref names) => {
                params.hash(state);
                chunk.code.hash(state);
                names.hash(state);
            }
            Object::Method(ref params, ref chunk, _, ref names) => {
                params.hash(state);
                chunk.code.hash(state);
                names.hash(state);
            }
            _ => "".hash(state),
        }
    }
}
