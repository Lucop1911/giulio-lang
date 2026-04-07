//! CLI execution modes for the G-lang interpreter.
//!
//! Each submodule implements one way to run G-lang code:
//!
//! - `run_source` — lex, parse, and execute a `.g` file
//! - `run_check` — lex and parse only (syntax validation)
//! - `run_repl_mode` — interactive read-eval-print loop
//! - `print_help` — CLI usage information

pub mod print_help;
pub mod run_repl_mode;
pub mod run_source;
pub mod run_check;