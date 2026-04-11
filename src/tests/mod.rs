//! Consolidated integration tests for the G-lang VM.
//!
//! All language features are tested against the stack-based VM execution engine.

#[cfg(test)]
mod lexer_tests;

#[cfg(test)]
mod parser_tests;

#[cfg(test)]
mod wasm_tests;

#[cfg(test)]
mod vm_tests;
