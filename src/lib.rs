//! G-lang — a dynamically-typed, interpreted programming language written in Rust.
//!
//! # Architecture
//!
//! The language pipeline follows a stack-based VM design:
//!
//! 1. **Lexer** — tokenises raw source bytes into a stream of [`Token`]s
//! 2. **Parser** — consumes tokens via a Pratt-parser combinator (built on `nom`)
//!    and produces an [`ast::Program`] (a vector of [`ast::Stmt`])
//! 3. **Compiler** — compiles AST into bytecode chunks for the VM
//! 4. **VM** — executes bytecode with a stack-based virtual machine
//!
//! # Public API
//!
//! The most commonly used types are re-exported at crate root:
//!
//! - [`Lexer`] — entry point for lexical analysis
//! - [`Parser`] — entry point for syntactic analysis
//! - [`Compiler`] — compiles programs to bytecode
//! - [`VM`] — the stack-based virtual machine
//! - [`Token`], [`Tokens`] — the token stream types
//! - [`LangError`], [`RuntimeError`] — error enumerations

pub mod ast;
pub mod lexer;
pub mod parser;
pub mod runtime;
pub mod std;
pub mod runners;
pub mod wasm;
pub mod vm;

#[cfg(test)]
mod tests;

pub use crate::lexer::lexer::Lexer;
pub use crate::parser::parser::Parser;
pub use crate::vm::compiler::Compiler;
pub use crate::vm::vm::VirtualMachine;
pub use crate::lexer::token::{Token, Tokens};
pub use crate::runtime::runtime_errors::{LangError, RuntimeError};
