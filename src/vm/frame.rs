//! Call frame — the execution context for a single function invocation.
//!
//! Each frame tracks:
//! - Which chunk is being executed
//! - The current instruction pointer
//! - The slot vector for local variables
//! - The closure environment (for capturing outer scope)
//! - The stack base (where this frame's slots begin in the VM's stack)
//! - Local variable names (for closure capture resolution)

use std::sync::{Arc, Mutex};

use crate::vm::runtime::env::Environment;
use crate::vm::obj::Object;
use crate::vm::chunk::Chunk;

/// A single call frame on the VM's call stack.
///
/// Frames are created when functions are called and popped when they
/// return. The global scope also runs in a frame (the "root frame").
pub struct CallFrame {
    /// The bytecode chunk being executed (function body or program).
    pub chunk: Arc<Chunk>,
    /// Current instruction pointer within `chunk.code`.
    pub ip: usize,
    /// Base index in the VM's stack where this frame's slots begin.
    /// Slot 0 is at `stack[slots_base]`, slot 1 at `stack[slots_base + 1]`, etc.
    pub slots_base: usize,
    /// Number of slots allocated for this frame (params + locals).
    pub slot_count: usize,
    /// Closure environment — `Some` for closures that capture outer scope,
    /// `None` for top-level functions and the root frame.
    pub closure_env: Option<Arc<Mutex<Environment>>>,
    /// Local variable names indexed by slot. Used by `OpClosure` to resolve
    /// captured variable names to stack slot indices.
    pub local_names: Vec<String>,
}

impl CallFrame {
    /// Creates a new frame for the root (top-level program) execution.
    pub fn new_root(chunk: Arc<Chunk>, slot_count: usize) -> Self {
        CallFrame {
            chunk,
            ip: 0,
            slots_base: 0,
            slot_count,
            closure_env: None,
            local_names: Vec::new(),
        }
    }

    /// Creates a new frame for a function body execution (for async functions).
    pub fn new_function_body(
        chunk: Arc<Chunk>,
        slot_count: usize,
        local_names: Vec<String>,
    ) -> Self {
        CallFrame {
            chunk,
            ip: 0,
            slots_base: 0,
            slot_count,
            closure_env: None,
            local_names,
        }
    }

    /// Creates a new frame for a function call.
    ///
    /// - `chunk`: the function's compiled bytecode
    /// - `slots_base`: where this frame's slots start in the VM stack
    /// - `slot_count`: total slots needed (params + locals)
    /// - `closure_env`: the environment captured at function definition time
    /// - `local_names`: names of local variables indexed by slot (params first, then lets)
    pub fn new_function(
        chunk: Arc<Chunk>,
        slots_base: usize,
        slot_count: usize,
        closure_env: Arc<Mutex<Environment>>,
        local_names: Vec<String>,
    ) -> Self {
        CallFrame {
            chunk,
            ip: 0,
            slots_base,
            slot_count,
            closure_env: Some(closure_env),
            local_names,
        }
    }

    /// Reads a local variable by slot index.
    ///
    /// # Panics
    /// Panics if `slot` is out of bounds for this frame.
    pub fn get_local<'a>(&self, stack: &'a [Object], slot: usize) -> &'a Object {
        &stack[self.slots_base + slot]
    }

    /// Writes a local variable by slot index.
    ///
    /// # Panics
    /// Panics if `slot` is out of bounds for this frame.
    pub fn set_local(&self, stack: &mut [Object], slot: usize, value: Object) {
        stack[self.slots_base + slot] = value;
    }
}