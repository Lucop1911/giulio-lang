//! Scoped variable environment for the interpreter.
//!
//! The [`Environment`] stores bindings using a dual strategy:
//!
//! 1. **Slot-based** (`slots: Vec<Object>`) — O(1) indexed access for
//!    function parameters and local `let` bindings. Slot indices are
//!    assigned by the compiler pass in `compute_slots`.
//! 2. **Name-based** (`store: HashMap<String, Object>`) — fallback for
//!    builtins, top-level globals, and variables captured from enclosing
//!    scopes whose slot indices would collide with the current frame.
//!
//! Environments form a parent chain (`Option<Arc<Mutex<Environment>>>`)
//! that implements lexical scoping. Closures capture their defining
//! environment via `Arc<Mutex<Environment>>`.

use crate::ast::ast::Ident;
use crate::vm::runtime::builtins::functions::BuiltinsFunctions;
use crate::vm::obj::Object;
use ahash::{AHasher, HashMapExt};
use std::hash::BuildHasherDefault;
use std::sync::{Arc, Mutex};

type HashMap<K, V> = std::collections::HashMap<K, V, BuildHasherDefault<AHasher>>;

/// The environment stores variables in two ways:
/// - `store`: HashMap for name-based lookups (builtins, top-level lets, globals)
/// - `slots`: Vec for O(1) slot-based lookups (function params and locals)
///
/// Environments form a parent chain that implements lexical scoping.
/// Closures capture their defining environment via `Arc<Mutex<Environment>>`.
#[derive(Debug, Clone)]
pub struct Environment {
    store: HashMap<String, Object>,
    parent: Option<Arc<Mutex<Environment>>>,
}

impl Default for Environment {
    fn default() -> Self {
        Self::new()
    }
}

impl Environment {
    pub(crate) fn new() -> Self {
        Environment {
            store: HashMap::new(),
            parent: None,
        }
    }

    pub(crate) fn new_root() -> Self {
        let mut hashmap = HashMap::new();
        Self::fill_env_with_builtins(&mut hashmap);
        Environment {
            store: hashmap,
            parent: None,
        }
    }

    pub(crate) fn new_with_outer(outer: Arc<Mutex<Environment>>) -> Self {
        Environment {
            store: HashMap::new(),
            parent: Some(outer),
        }
    }

    fn fill_env_with_builtins(hashmap: &mut HashMap<String, Object>) {
        let builtins_functions = BuiltinsFunctions::new();
        let builtins = builtins_functions.get_builtins();
        for (Ident { name, .. }, object) in builtins {
            hashmap.insert(name, object);
        }
    }

    pub(crate) fn set_by_name(&mut self, name: &str, val: Object) {
        match self.store.get_key_value(name) {
            Some(_) => {
                self.store.insert(name.to_string(), val);
            }
            None => {
                if let Some(ref parent_env) = self.parent 
                    && parent_env.lock().unwrap().has_var(name) {
                        parent_env.lock().unwrap().set_by_name(name, val);
                        return;
                }
                self.store.insert(name.to_string(), val);
            }
        }
    }

    pub(crate) fn get_by_name(&self, name: &str) -> Option<Object> {
        match self.store.get(name) {
            Some(o) => Some(o.clone()),
            None => match &self.parent {
                Some(parent_env) => parent_env.lock().unwrap().get_by_name(name),
                None => None,
            },
        }
    }

    pub(crate) fn has_var(&self, name: &str) -> bool {
        if self.store.contains_key(name) {
            return true;
        }
        if let Some(ref parent) = self.parent {
            return parent.lock().unwrap().has_var(name);
        }
        false
    }

    
}
