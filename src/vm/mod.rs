//! Stack-based bytecode virtual machine
//!
//! # Architecture
//!
//! The VM executes bytecode produced by the compiler (`compiler.rs`).
//! It uses an operand stack for intermediate values and a call frame
//! stack for function invocation.
//!
//! # Modules
//!
//! - `instruction` — opcode definitions, encoding, and decoding
//! - `chunk` — bytecode units with constant pools and source maps
//! - `frame` — call frame management
//! - `compiler` — AST → bytecode compiler
//! - `vm` — execution engine
//! - `ops` — modular operation implementations

pub mod chunk;
pub mod compiler;
pub mod frame;
pub mod instruction;
pub mod ops;
pub mod vm;
pub mod runtime;
pub mod obj;