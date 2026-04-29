//! Runtime support for the stack-based VM.
//!
//! # Modules
//!
//! - `env` — scoped variable environments with O(1) slot-based lookups
//! - `obj` — the [`Object`] enum representing all runtime values
//! - `builtins` — standard library functions (string, math, io, http, etc.)
//! - `module_registry` — module loading, caching, and WASM integration
//! - `helpers` — shared evaluation utilities

pub mod env;
pub mod builtins;
pub mod module_registry;
pub mod wasm_loader;
pub mod runtime_errors;
pub mod type_converters;