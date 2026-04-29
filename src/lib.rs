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


pub mod ast;
pub mod lexer;
pub mod parser;
pub mod std;
pub mod runners;
pub mod wasm;
pub mod vm;

#[cfg(test)]
mod tests;