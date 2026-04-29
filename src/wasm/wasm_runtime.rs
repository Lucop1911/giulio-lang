//! WASM runtime — loading, instantiation, and execution of WASM modules.
//!
//! Supports both WASI Preview 1 (classic modules) and Preview 2 (components)
//! via `wasmtime`. The [`WasmRuntime`] holds the engine, [`WasmModule`]
//! represents a compiled module, and [`WasmInstance`] is a runnable
//! instantiation with unified function-call and memory APIs.

use crate::vm::runtime::runtime_errors::RuntimeError;
use std::path::Path;
use wasmtime::component::types::ComponentItem;
use wasmtime::component::{
    Component, Func, Instance, Linker as ComponentLinker, ResourceTable, Val,
};
use wasmtime::{Engine, Linker as ClassicLinker, Memory, Module, Store, Val as ClassicVal};
use wasmtime_wasi::filesystem::{DirPerms, FilePerms};
use wasmtime_wasi::p1::{self as wasi_p1, WasiP1Ctx};
use wasmtime_wasi::WasiCtx;
use wasmtime_wasi::WasiCtxBuilder;
use wasmtime_wasi::WasiView;

use super::type_conversions;

pub use type_conversions::*;

/// Wrapper around the `wasmtime::Engine`.
///
/// Manages compilation and store creation. The engine is cheaply cloneable
/// and internally reference-counted.
#[derive(Default, Clone)]
pub struct WasmRuntime {
    engine: Engine,
}

impl WasmRuntime {
    pub fn new() -> Result<Self, RuntimeError> {
        Ok(Self::default())
    }

    /// Returns a reference to the underlying Wasmtime engine.
    pub fn engine(&self) -> &Engine {
        &self.engine
    }

    /// Constructs a WASI Preview 2 context with inherited stdio, env, args,
    /// network access, and the current directory preopened.
    pub fn create_wasi_ctx() -> WasiCtx {
        WasiCtxBuilder::new()
            .inherit_stdio()
            .inherit_env()
            .inherit_args()
            .inherit_network()
            .preopened_dir(".", ".", DirPerms::all(), FilePerms::all())
            .unwrap()
            .allow_blocking_current_thread(true)
            .build()
    }

    /// Constructs a WASI Preview 1 context (classic WASI) with the same
    /// permissions as `create_wasi_ctx`.
    pub fn create_wasi_p1_ctx() -> WasiP1Ctx {
        WasiCtxBuilder::new()
            .inherit_stdio()
            .inherit_env()
            .inherit_args()
            .inherit_network()
            .preopened_dir(".", ".", DirPerms::all(), FilePerms::all())
            .unwrap()
            .allow_blocking_current_thread(true)
            .build_p1()
    }

    /// Creates a new `Store<WasmContext>` with both WASI P1 and P2 contexts
    /// and a fresh resource table.
    pub fn create_store(&self) -> Store<WasmContext> {
        let context = WasmContext {
            wasi: Self::create_wasi_ctx(),
            wasi_p1: Self::create_wasi_p1_ctx(),
            table: ResourceTable::new(),
        };
        Store::new(&self.engine, context)
    }
}

/// A compiled WASM module — either a WASI Preview 2 component or a
/// WASI Preview 1 classic module.
///
/// Loading is format-agnostic: `load_from_bytes` auto-detects the binary
/// magic header (classic vs component) and also supports WAT text format.
pub enum WasmModule {
    /// WASI Preview 2 component
    Component { component: Component },
    /// WASI Preview 1 classic module
    Classic { module: Module },
}

impl WasmModule {
    /// Loads a module from a file path. Auto-detects format by magic header.
    pub fn load(engine: &Engine, path: &Path) -> Result<Self, RuntimeError> {
        let wasm_bytes = std::fs::read(path).map_err(|e| {
            RuntimeError::InvalidOperation(format!(
                "Failed to read wasm file '{}': {}",
                path.display(),
                e
            ))
        })?;

        // Extract file name (without extension) for error messages
        let name = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown")
            .to_string();

        Self::load_from_binary(engine, &name, &wasm_bytes)
    }

    /// Loads from raw bytes — auto-detects WASM binary magic header
    /// (classic vs component) or parses WAT text format.
    fn load_from_binary(
        engine: &Engine,
        name: &str,
        wasm_bytes: &[u8],
    ) -> Result<Self, RuntimeError> {
        // Check for WASM binary magic header
        if wasm_bytes.len() >= 8 && wasm_bytes.starts_with(&[0x00, 0x61, 0x73, 0x6d]) {
            let version = &wasm_bytes[4..8];
            // Version 1 = classic module (P1), version 13 = component (P2)
            return if version == [0x01, 0x00, 0x00, 0x00] {
                Self::load_classic_module(engine, name, wasm_bytes)
            } else {
                Self::load_component(engine, name, wasm_bytes)
            };
        }

        // Handle WAT (WebAssembly Text) format - parse to binary first
        if let Ok(wat_str) = std::str::from_utf8(wasm_bytes) {
            let trimmed = wat_str.trim_start();
            if trimmed.starts_with("(module") || trimmed.starts_with("(;") {
                // WAT for classic module - parse with wat crate
                let wasm_binary = wat::parse_str(wat_str).map_err(|e| {
                    RuntimeError::InvalidOperation(format!(
                        "Failed to parse WAT for module '{}': {}",
                        name, e
                    ))
                })?;
                return Self::load_classic_module(engine, name, &wasm_binary);
            }
            if trimmed.starts_with("(component") {
                // WAT for component - parse with wat crate
                let wasm_binary = wat::parse_str(wat_str).map_err(|e| {
                    RuntimeError::InvalidOperation(format!(
                        "Failed to parse WAT for component '{}': {}",
                        name, e
                    ))
                })?;
                return Self::load_component(engine, name, &wasm_binary);
            }
        }

        Err(RuntimeError::InvalidOperation(format!(
            "File '{}' does not appear to be a valid WASM module or component",
            name
        )))
    }

    /// Compiles a WASI Preview 2 component from bytes.
    fn load_component(
        engine: &Engine,
        name: &str,
        wasm_bytes: &[u8],
    ) -> Result<Self, RuntimeError> {
        let component = Component::new(engine, wasm_bytes).map_err(|e| {
            RuntimeError::InvalidOperation(format!(
                "Failed to compile wasm component '{}': {}",
                name, e
            ))
        })?;

        Ok(WasmModule::Component { component })
    }

    /// Compiles a WASI Preview 1 classic module from bytes.
    fn load_classic_module(
        engine: &Engine,
        name: &str,
        wasm_bytes: &[u8],
    ) -> Result<Self, RuntimeError> {
        let module = Module::new(engine, wasm_bytes).map_err(|e| {
            RuntimeError::InvalidOperation(format!(
                "Failed to compile wasm module '{}': {}",
                name, e
            ))
        })?;

        Ok(WasmModule::Classic { module })
    }

    /// Public wrapper around `load_from_binary`.
    pub fn load_from_bytes(
        engine: &Engine,
        name: &str,
        bytes: &[u8],
    ) -> Result<Self, RuntimeError> {
        Self::load_from_binary(engine, name, bytes)
    }

    /// Instantiates the compiled module into a runnable [`WasmInstance`].
    ///
    /// Links WASI imports (P1 or P2) automatically and captures exported
    /// memory for classic modules.
    pub fn instantiate(
        &self,
        store: &mut Store<WasmContext>,
    ) -> Result<WasmInstance, RuntimeError> {
        match self {
            // Handle WASI Preview 2 components
            WasmModule::Component { component } => {
                let engine = store.engine();
                let mut linker: ComponentLinker<WasmContext> = ComponentLinker::new(engine);
                // Add WASI P2 support to the linker
                wasmtime_wasi::p2::add_to_linker_sync(&mut linker).map_err(|e| {
                    RuntimeError::InvalidOperation(format!("Failed to add WASI to linker: {}", e))
                })?;

                let instance = linker.instantiate(&mut *store, component).map_err(|e| {
                    RuntimeError::InvalidOperation(format!(
                        "Failed to instantiate wasm component '{}': {}",
                        self.name(),
                        e
                    ))
                })?;

                Ok(WasmInstance::Component(WasmComponentInstance { instance }))
            }
            // Handle WASI Preview 1 classic modules
            WasmModule::Classic { module } => {
                let engine = store.engine();
                let mut linker: ClassicLinker<WasmContext> = ClassicLinker::new(engine);
                // Add WASI P1 support to the linker
                wasi_p1::add_to_linker_sync(&mut linker, |ctx| &mut ctx.wasi_p1).map_err(|e| {
                    RuntimeError::InvalidOperation(format!("Failed to add WASI to linker: {}", e))
                })?;

                let instance = linker.instantiate(&mut *store, module).map_err(|e| {
                    RuntimeError::InvalidOperation(format!(
                        "Failed to instantiate wasm module '{}': {}",
                        self.name(),
                        e
                    ))
                })?;

                // Try to get exported memory (optional for classic modules)
                let memory = instance
                    .get_export(&mut *store, "memory")
                    .and_then(|e| e.into_memory());

                Ok(WasmInstance::Classic(WasmClassicInstance {
                    instance,
                    memory,
                }))
            }
        }
    }

    /// Returns a human-readable type name for debugging (`"component"` or `"classic"`).
    pub fn name(&self) -> &'static str {
        match self {
            WasmModule::Component { .. } => "component",
            WasmModule::Classic { .. } => "classic",
        }
    }

    /// Returns the inner `Component` if this is a component module, `None` otherwise.
    pub fn component(&self) -> Option<&Component> {
        match self {
            WasmModule::Component { component } => Some(component),
            WasmModule::Classic { .. } => None,
        }
    }
}

/// An instantiated WASM module — ready for function calls and memory access.
///
/// Provides a unified API over both WASI Preview 1 (classic) and
/// Preview 2 (component) instances, handling value conversions internally.
pub enum WasmInstance {
    Component(WasmComponentInstance),
    Classic(WasmClassicInstance),
}

/// Wrapper around an instantiated WASI Preview 2 component.
pub struct WasmComponentInstance {
    instance: Instance,
}

impl WasmComponentInstance {
    /// Looks up an exported function by name.
    fn get_export(&self, store: &mut Store<WasmContext>, name: &str) -> Option<Func> {
        self.instance.get_func(store, name)
    }

    /// Calls an exported function using component-model `Val` types.
    ///
    /// Pre-allocates the result vector based on the function's declared
    /// return arity.
    pub fn call_func_with_args(
        &self,
        store: &mut Store<WasmContext>,
        name: &str,
        args: &[Val],
    ) -> Result<Vec<Val>, RuntimeError> {
        let func = self.get_export(store, name).ok_or_else(|| {
            RuntimeError::InvalidOperation(format!(
                "Function '{}' not found in wasm component",
                name
            ))
        })?;

        // Get result types to allocate space for return values
        let func_ty = func.ty(&mut *store);
        let result_count = func_ty.results().len();
        let mut results = vec![Val::S32(0); result_count];

        func.call(&mut *store, args, &mut results).map_err(|e| {
            RuntimeError::InvalidOperation(format!(
                "Failed to call wasm function '{}': {}",
                name, e
            ))
        })?;

        Ok(results)
    }
}

/// Wrapper around an instantiated WASI Preview 1 classic module.
pub struct WasmClassicInstance {
    instance: wasmtime::Instance,
    /// Exported linear memory, if the module exports one named "memory".
    memory: Option<Memory>,
}

impl WasmClassicInstance {
    /// Returns the exported linear memory, if available.
    pub fn get_memory(&self) -> Option<&Memory> {
        self.memory.as_ref()
    }

    /// Looks up an exported function by name.
    fn get_export(&self, store: &mut Store<WasmContext>, name: &str) -> Option<wasmtime::Func> {
        self.instance
            .get_export(&mut *store, name)
            .and_then(|e| e.into_func())
    }

    /// Calls an exported function using classic (non-component) `Val` types.
    pub fn call_func_with_args(
        &self,
        store: &mut Store<WasmContext>,
        name: &str,
        args: &[ClassicVal],
    ) -> Result<Vec<ClassicVal>, RuntimeError> {
        let func = self.get_export(store, name).ok_or_else(|| {
            RuntimeError::InvalidOperation(format!("Function '{}' not found in wasm module", name))
        })?;

        // Get result types to allocate space for return values
        let func_ty = func.ty(&mut *store);
        let result_count = func_ty.results().len();
        let mut results = vec![ClassicVal::I32(0); result_count];

        func.call(&mut *store, args, &mut results).map_err(|e| {
            RuntimeError::InvalidOperation(format!(
                "Failed to call wasm function '{}': {}",
                name, e
            ))
        })?;

        Ok(results)
    }
}

impl WasmInstance {
    /// Returns the names of all exported functions.
    ///
    /// For classic modules this is a flat list. For components, nested
    /// instance exports are flattened with `instance#function` notation.
    pub fn get_export_names(
        &self,
        store: &mut Store<WasmContext>,
        component: Option<&Component>,
    ) -> Vec<String> {
        match self {
            WasmInstance::Classic(i) => i
                .instance
                .exports(store)
                .filter_map(|e| {
                    let name = e.name().to_string();
                    if e.into_func().is_some() {
                        Some(name)
                    } else {
                        None
                    }
                })
                .collect(),
            WasmInstance::Component(_) => {
                let engine = store.engine().clone();
                let mut names = Vec::new();
                if let Some(c) = component {
                    for (name, item) in c.component_type().exports(&engine) {
                        match item {
                            ComponentItem::ComponentFunc(_) => {
                                names.push(String::from(name));
                            }
                            ComponentItem::ComponentInstance(inst_ty) => {
                                for (fn_name, inner) in inst_ty.exports(&engine) {
                                    if matches!(inner, ComponentItem::ComponentFunc(_)) {
                                        names.push(format!("{}#{}", name, fn_name));
                                    }
                                }
                            }
                            _ => {}
                        }
                    }
                    names
                } else {
                    Vec::new()
                }
            }
        }
    }

    /// Returns exported linear memory (only available for classic modules).
    /// Components handle memory through the resource table instead.
    pub fn get_memory(&self) -> Option<&Memory> {
        match self {
            // Components handle memory differently through the resource table
            WasmInstance::Component(_) => None,
            WasmInstance::Classic(i) => i.get_memory(),
        }
    }

    // Unified call interface - converts component values to classic values for classic modules
    pub fn call_func_with_args(
        &self,
        store: &mut Store<WasmContext>,
        name: &str,
        args: &[Val],
    ) -> Result<Vec<Val>, RuntimeError> {
        match self {
            // Components use component values directly
            WasmInstance::Component(i) => i.call_func_with_args(store, name, args),
            // Classic modules need conversion from component values to classic values
            WasmInstance::Classic(i) => {
                // Convert component values to classic values
                let classic_args: Vec<ClassicVal> = args
                    .iter()
                    .map(|v| match v {
                        Val::S32(n) => ClassicVal::I32(*n),
                        Val::U32(n) => ClassicVal::I32(*n as i32),
                        Val::S64(n) => ClassicVal::I64(*n),
                        Val::U64(n) => ClassicVal::I64(*n as i64),
                        Val::Float32(n) => ClassicVal::F32(n.to_bits()),
                        Val::Float64(n) => ClassicVal::F64(n.to_bits()),
                        // Other types default to 0
                        _ => ClassicVal::I32(0),
                    })
                    .collect();

                // Call the function
                let results: Vec<ClassicVal> = i.call_func_with_args(store, name, &classic_args)?;

                // Convert results back to component values
                Ok(results
                    .iter()
                    .map(|v| match v {
                        ClassicVal::I32(n) => Val::S32(*n),
                        ClassicVal::I64(n) => Val::S64(*n),
                        ClassicVal::F32(n) => Val::Float32(f32::from_bits(*n)),
                        ClassicVal::F64(n) => Val::Float64(f64::from_bits(*n)),
                        _ => Val::S32(0),
                    })
                    .collect())
            }
        }
    }
}

// Context stored in each WASM store - holds WASI state and resource tables
pub struct WasmContext {
    pub wasi: WasiCtx,        // WASI Preview 2 context
    pub wasi_p1: WasiP1Ctx,   // WASI Preview 1 context
    pub table: ResourceTable, // Resource table for component model handles
}

// Required trait for WASI Preview 2 integration
impl WasiView for WasmContext {
    fn ctx(&mut self) -> wasmtime_wasi::WasiCtxView<'_> {
        wasmtime_wasi::WasiCtxView {
            ctx: &mut self.wasi,
            table: &mut self.table,
        }
    }
}

// Default implementation creates fresh WASI contexts
impl Default for WasmContext {
    fn default() -> Self {
        Self {
            wasi: WasmRuntime::create_wasi_ctx(),
            wasi_p1: WasmRuntime::create_wasi_p1_ctx(),
            table: ResourceTable::new(),
        }
    }
}

// Type alias for store with WASM context
pub type WasmStore = Store<WasmContext>;
