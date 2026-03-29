use std::sync::{Arc, Mutex};
use crate::interpreter::{env::Environment, module_registry::ModuleRegistry};

#[derive(Clone)]
pub struct EvalContext {
    pub env: Arc<Mutex<Environment>>,
    pub module_registry: Arc<Mutex<ModuleRegistry>>,
}
impl EvalContext {
    pub fn new(env: Arc<Mutex<Environment>>, module_registry: Arc<Mutex<ModuleRegistry>>) -> Self {
        Self { env, module_registry }
    }
}