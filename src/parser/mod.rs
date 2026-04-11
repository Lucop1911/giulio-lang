//! Parser module — converts a token stream into an AST.
//!
//! The parser is built on `nom` combinators and uses a Pratt-parser
//! (precedence-climbing) strategy for expressions. This avoids the
//! left-recursion and ambiguity issues of a naive recursive descent
//! approach while keeping the grammar declarative.
//!
//! # Modules
//!
//! - `parser` — the main Pratt parser and statement parsers
//! - `parser_helpers` — shared combinators (`parens`, `braced`, `comma_separated`, etc.)
//! - `await_ctx_helpers` — validates that `await` only appears inside `async fn`

pub mod await_ctx_helpers;
pub mod parser;
pub mod parser_helpers;
