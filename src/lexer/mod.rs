//! Lexical analysis for G-lang source code.
//!
//! The lexer transforms raw UTF-8 bytes into a stream of [`Token`]s,
//! skipping whitespace and `//` line comments. It is built entirely
//! on `nom` parser combinators for zero-copy, streaming-friendly parsing.
//!
//! # Modules
//!
//! - `lexer` — the main [`Lexer`] type with `lex_tokens` entry point
//! - `token` — the [`Token`] enum and the [`Tokens`](token::Tokens) wrapper

pub mod lexer;
pub mod token;
