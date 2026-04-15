//! Runtime support for the stack-based VM.
//!
//! # Modules
//!
//! - `env` — scoped variable environments with O(1) slot-based lookups
//! - `obj` — the [`Object`] enum representing all runtime values
//! - `builtins` — standard library functions (string, math, io, http, etc.)
//! - `module_registry` — module loading, caching, and WASM integration
//! - `constant_pool` — compile-time literal extraction for faster evaluation
//! - `helpers` — shared evaluation utilities

pub mod env;
pub mod obj;
pub mod builtins;
pub mod module_registry;
pub mod helpers;
pub mod constant_pool;
pub mod runtime_errors;