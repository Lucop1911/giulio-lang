//! Runtime support for the stack-based VM.
//!
//! # Modules
//!
//! - `env` — scoped variable environments with O(1) slot-based lookups
//! - `obj` — the [`Object`] enum representing all runtime values
//! - `builtins` — standard library functions (string, math, io, http, etc.)
//! - `module_registry` — module loading, caching, and WASM integration
//! - `helpers` — shared evaluation utilities

pub(crate) mod env;
pub(crate) mod builtins;
pub(crate) mod module_registry;
pub(crate) mod wasm_loader;
pub(crate) mod runtime_errors;
pub(crate) mod type_converters;