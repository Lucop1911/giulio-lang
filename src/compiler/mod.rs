//! Compiler pass — slot allocation for O(1) variable access.
//!
//! After parsing, the AST contains identifiers with `SlotIndex::UNSET`.
//! This pass walks the AST and assigns concrete slot indices based on
//! lexical scope, so that the interpreter can use `Vec` indexing instead
//! of `HashMap` lookups for local variables.

pub mod compute_slots;
