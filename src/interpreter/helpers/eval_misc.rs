use std::sync::Arc;
use crate::{
    ast::ast::{Ident, ImportItems},
    errors::RuntimeError,
    interpreter::{
        module_registry::ModuleRegistry, obj::Object
    },
};
use super::super::eval::{Evaluator, EvalFuture};

impl Evaluator {
    pub fn register_ident(&mut self, ident: Ident, object: Object) -> Object {
        let Ident(name) = ident;
        self.env.lock().unwrap().set(&name, object.clone());
        object
    }

    pub fn eval_import(&mut self, path: Vec<String>, items: ImportItems) -> EvalFuture {
        let self_clone = self.clone();
        Box::pin(async move {
            let path_clone = path.clone();
            let module_registry_arc = Arc::clone(&self_clone.module_registry);
            
            let module_result = ModuleRegistry::load_module(module_registry_arc, &path_clone).await;
            
            let module = match module_result {
                Ok(m) => m,
                Err(e) => return Object::Error(e),
            };
            
            match items {
                ImportItems::All => {
                    for (name, obj) in &module.exports { // Iterate by reference
                        self_clone.env.lock().unwrap().set(name, obj.clone()); // name is &String, obj is &Object
                    }
                }
                ImportItems::Specific(names) => {
                    for name in names {
                        if let Some(obj) = module.exports.get(&name) {
                            self_clone.env.lock().unwrap().set(&name, obj.clone());
                        } else {
                            return Object::Error(RuntimeError::InvalidOperation(
                                format!("Module {} has no export '{}'", module.name, name)
                            ));
                        }
                    }
                }
                ImportItems::Single(name) => {
                    if let Some(obj) = module.exports.get(&name) {
                        self_clone.env.lock().unwrap().set(&name, obj.clone());
                    } else {
                        return Object::Error(RuntimeError::InvalidOperation(
                            format!("Module {} has no export '{}'", module.name, name)
                        ));
                    }
                }
            }
            
            Object::Null
        })
    }
}
