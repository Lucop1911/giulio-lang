//! WebAssembly integration via `wasmtime`.
//!
//! G-lang can load and execute WASM modules (both WASI Preview 1 classic
//! modules and Preview 2 components), call exported functions with G-lang
//! `Object` values, and interact with WASM linear memory.
//!
//! # Modules
//!
//! - `wasm_runtime` — [`WasmRuntime`], [`WasmModule`], [`WasmInstance`], and
//!   the WASI-aware [`WasmContext`] store type
//! - `type_conversions` — bidirectional conversion between G-lang [`Object`]s
//!   and WASM component/classic values, plus memory management helpers

pub mod type_conversions;
pub mod wasm_runtime;

pub use type_conversions::*;
pub use wasm_runtime::*;
