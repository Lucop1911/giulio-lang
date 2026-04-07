//! G-lang — a dynamically-typed, interpreted programming language written in Rust.
//!
//! # Architecture
//!
//! The language pipeline follows a classic interpreter design:
//!
//! 1. **Lexer** — tokenises raw source bytes into a stream of [`Token`]s
//! 2. **Parser** — consumes tokens via a Pratt-parser combinator (built on `nom`)
//!    and produces an [`ast::Program`] (a vector of [`ast::Stmt`])
//! 3. **Compiler** — performs a lightweight slot-allocation pass so that variable
//!    lookups inside function frames resolve in O(1) instead of O(n)
//! 4. **Interpreter** — walks the AST and evaluates it, maintaining a scoped
//!    [`interpreter::env::Environment`] and a rich [`interpreter::obj::Object`] type
//!
//! # Public API
//!
//! The most commonly used types are re-exported at crate root:
//!
//! - [`Lexer`] — entry point for lexical analysis
//! - [`Parser`] — entry point for syntactic analysis
//! - [`Evaluator`] — the async tree-walk interpreter
//! - [`Token`], [`Tokens`] — the token stream types
//! - [`LangError`], [`RuntimeError`] — error enumerations

pub mod ast;
pub mod lexer;
pub mod parser;
pub mod interpreter;
pub mod compiler;
pub mod errors;
pub mod std;
pub mod runners;
pub mod parser_errors;
pub mod wasm;

pub use crate::lexer::lexer::Lexer;
pub use crate::parser::parser::Parser;
pub use crate::interpreter::eval::Evaluator;
pub use crate::lexer::token::{Token, Tokens};
pub use crate::errors::{LangError, RuntimeError};