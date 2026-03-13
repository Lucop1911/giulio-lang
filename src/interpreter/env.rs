use std::sync::{Arc, Mutex};
use crate::{ast::ast::Ident, interpreter::obj::Object};
use crate::interpreter::builtins::functions::BuiltinsFunctions;
use std::hash::BuildHasherDefault;
use ahash::{AHasher, HashMapExt};

type HashMap<K, V> = std::collections::HashMap<K, V, BuildHasherDefault<AHasher>>;

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
    pub fn new() -> Self {
        let mut hashmap = HashMap::new();
        Self::fill_env_with_builtins(&mut hashmap);
        Environment {
            store: hashmap,
            parent: None,
        }
    }

    pub fn new_with_outer(outer: Arc<Mutex<Environment>>) -> Self {
        let mut hashmap = HashMap::new();
        Self::fill_env_with_builtins(&mut hashmap);
        Environment {
            store: hashmap,
            parent: Some(outer),
        }
    }

    fn fill_env_with_builtins(hashmap: &mut HashMap<String, Object>) {
        let builtins_functions = BuiltinsFunctions::new();
        let builtins = builtins_functions.get_builtins();
        for (Ident(name), object) in builtins {
            hashmap.insert(name, object);
        }
    }

    pub fn set(&mut self, name: &str, val: Object) {
        // Check if key exists in current scope
        match self.store.get_key_value(name) {
            Some(_) => {
                // Key exists in current scope, replace it
                self.store.insert(name.to_string(), val);
            }
            None => {
                // Check parent scope
                if let Some(ref parent_env) = self.parent {
                    if parent_env.lock().unwrap().get(name).is_some() {
                        // Exists in parent, set it there
                        parent_env.lock().unwrap().set(name, val);
                        return;
                    }
                }
                // New key, insert in current scope
                self.store.insert(name.to_string(), val);
            }
        }
    }

    pub fn get(&self, name: &str) -> Option<Object> {
        match self.store.get(name) {
            Some(o) => {
                // Avoid clone for primitive types
                match o {
                    Object::Integer(i) => Some(Object::Integer(*i)),
                    Object::Float(f) => Some(Object::Float(*f)),
                    Object::Boolean(b) => Some(Object::Boolean(*b)),
                    Object::Null => Some(Object::Null),
                    // Keep clone for complex types
                    _ => Some(o.clone()),
                }
            }
            None => {
                match &self.parent {
                    Some(parent_env) => parent_env.lock().unwrap().get(name),
                    None => None,
                }
            }
        }
    }
}