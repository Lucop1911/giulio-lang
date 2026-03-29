use crate::ast::ast::{Ident, Program, SlotIndex, Stmt};
use crate::interpreter::builtins::functions::BuiltinsFunctions;
use crate::interpreter::obj::Object;
use ahash::{AHasher, HashMapExt};
use std::hash::BuildHasherDefault;
use std::sync::{Arc, Mutex};

type HashMap<K, V> = std::collections::HashMap<K, V, BuildHasherDefault<AHasher>>;

// The environment stores variables in two ways:
// - store: HashMap for name-based lookups (builtins, top-level lets, globals)
// - slots: Vec for O(1) slot-based lookups (function params and locals)
#[derive(Debug, Clone)]
pub struct Environment {
    store: HashMap<String, Object>,
    slots: Vec<Object>,
    parent: Option<Arc<Mutex<Environment>>>,
}

impl Default for Environment {
    fn default() -> Self {
        Self::new()
    }
}

impl Environment {
    pub fn new() -> Self {
        let mut hashmap = HashMap::new();
        Self::fill_env_with_builtins(&mut hashmap);
        Environment {
            store: hashmap,
            slots: Vec::new(),
            parent: None,
        }
    }

    pub fn new_with_outer(outer: Arc<Mutex<Environment>>) -> Self {
        let mut hashmap = HashMap::new();
        Self::fill_env_with_builtins(&mut hashmap);
        Environment {
            store: hashmap,
            slots: Vec::new(),
            parent: Some(outer),
        }
    }

    pub fn new_with_slots(num_slots: usize) -> Self {
        let mut hashmap = HashMap::new();
        Self::fill_env_with_builtins(&mut hashmap);
        Environment {
            store: hashmap,
            slots: vec![Object::Null; num_slots],
            parent: None,
        }
    }

    // Creates a new function environment with pre-allocated slots.
    // num_slots = params.len() + number of let-bindings in the body.
    // Use count_slots(params, body) to compute the correct value.
    pub fn new_function_env(outer: Arc<Mutex<Environment>>, num_slots: usize) -> Self {
        let mut hashmap = HashMap::new();
        Self::fill_env_with_builtins(&mut hashmap);
        Environment {
            store: hashmap,
            slots: vec![Object::Null; num_slots],
            parent: Some(outer),
        }
    }

    // Total slots needed for a function frame:
    // params.len() + number of direct let-bindings in body.
    pub fn count_slots(params: &[Ident], body: &Program) -> usize {
        let let_count = body
            .iter()
            .filter(|s| matches!(s, Stmt::LetStmt(..)))
            .count();
        params.len() + let_count
    }

    fn fill_env_with_builtins(hashmap: &mut HashMap<String, Object>) {
        let builtins_functions = BuiltinsFunctions::new();
        let builtins = builtins_functions.get_builtins();
        for (Ident { name, .. }, object) in builtins {
            hashmap.insert(name, object);
        }
    }

    // Sets a variable's value. Uses slot if available (O(1)),
    // otherwise falls back to name-based lookup in store/hashmap.
    pub fn set(&mut self, ident: &Ident, val: Object) {
        let name = &ident.name;
        let slot = ident.slot;

        if !slot.is_unset() {
            // Write to both slot (for O(1) access) and store
            //(so that closures capturing this variable can find it by name when
            // their slot indices are UNSET to avoid cross-frame collisions).
            self.set_slot(slot, val.clone());
            self.store.insert(name.clone(), val);
            return;
        }

        match self.store.get_key_value(name) {
            Some(_) => {
                self.store.insert(name.clone(), val);
            }
            None => {
                if let Some(ref parent_env) = self.parent {
                    if parent_env.lock().unwrap().has_var(name) {
                        parent_env.lock().unwrap().set(ident, val);
                        return;
                    }
                }
                self.store.insert(name.clone(), val);
            }
        }
    }

    // Unlike set(), set_by_name() ignores the slot and uses the HashMap directly
    pub fn set_by_name(&mut self, name: &str, val: Object) {
        match self.store.get_key_value(name) {
            Some(_) => {
                self.store.insert(name.to_string(), val);
            }
            None => {
                if let Some(ref parent_env) = self.parent {
                    if parent_env.lock().unwrap().has_var(name) {
                        parent_env.lock().unwrap().set_by_name(name, val);
                        return;
                    }
                }
                self.store.insert(name.to_string(), val);
            }
        }
    }

    pub fn set_slot(&mut self, slot: SlotIndex, val: Object) {
        let idx = slot.0 as usize;
        if idx >= self.slots.len() {
            self.slots.resize(idx + 1, Object::Null);
        }
        self.slots[idx] = val;
    }

    pub fn ensure_slots(&mut self, min_slots: usize) {
        if self.slots.len() < min_slots {
            self.slots.resize(min_slots, Object::Null);
        }
    }

    pub fn get_slot(&self, slot: SlotIndex) -> Option<Object> {
        let idx = slot.0 as usize;
        if idx < self.slots.len() {
            Some(self.slots[idx].clone())
        } else {
            None
        }
    }

    fn has_var(&self, name: &str) -> bool {
        if self.store.contains_key(name) {
            return true;
        }
        if let Some(ref parent) = self.parent {
            return parent.lock().unwrap().has_var(name);
        }
        false
    }

    // Gets a variable's value. Uses slot if available (O(1)),
    // otherwise searches store by name. Recurses to parent if not found.
    // Falls back to name lookup if slot is out of bounds (safety net).
    pub fn get(&self, ident: &Ident) -> Option<Object> {
        let name = &ident.name;
        let slot = ident.slot;

        if !slot.is_unset() {
            if let Some(obj) = self.get_slot(slot) {
                return Some(obj);
            }
            // Slot out of bounds — fall back to name lookup.
        }

        match self.store.get(name) {
            Some(o) => Some(o.clone()),
            None => match &self.parent {
                Some(parent_env) => parent_env.lock().unwrap().get(ident),
                None => None,
            },
        }
    }

    // Unlike get(), get_by_name() goes straight to name based lookup
    pub fn get_by_name(&self, name: &str) -> Option<Object> {
        match self.store.get(name) {
            Some(o) => Some(o.clone()),
            None => match &self.parent {
                Some(parent_env) => parent_env.lock().unwrap().get_by_name(name),
                None => None,
            },
        }
    }
}
