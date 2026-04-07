//! Abstract Syntax Tree definitions for G-lang.
//!
//! # Core types
//!
//! - [`Program`](ast::Program) — a sequence of [`Stmt`](ast::Stmt)s
//! - [`Expr`](ast::Expr) — expressions that evaluate to an [`Object`](crate::interpreter::obj::Object)
//! - [`Stmt`](ast::Stmt) — statements that perform actions (declarations, control flow)
//! - [`SlotIndex`](ast::SlotIndex) — compile-time indices for O(1) variable access
//! - [`Precedence`](ast::Precedence) — operator precedence levels for the Pratt parser

pub mod ast;
