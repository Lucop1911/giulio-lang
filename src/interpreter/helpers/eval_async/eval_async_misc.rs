use std::sync::Arc;
use ahash::HashMapExt;

use crate::{
    ast::ast::{Expr, Ident, ImportItems},
    errors::RuntimeError,
    interpreter::{
        module_registry::ModuleRegistry, obj::{HashMap, Object}
    },
};
use super::super::super::eval::Evaluator;

impl Evaluator {
    pub fn async_eval_import(&mut self, path: Vec<String>, items: ImportItems) -> impl Future<Output = Object> + Send + '_  {
        let self_clone = self.clone();
        async move {
            let path_clone = path.clone();
            let module_registry_arc = Arc::clone(&self_clone.module_registry);
            
            let module_result = ModuleRegistry::load_module(module_registry_arc, &path_clone).await;
            
            let module = match module_result {
                Ok(m) => m,
                Err(e) => return Object::Error(e),
            };
            
            let simple_name = path.last().unwrap().clone();
            
            match items {
                ImportItems::All => {
                    let module_obj = Object::Module {
                        name: path.join("::"),
                        exports: module.exports.clone(),
                    };
                    self_clone.env.lock().unwrap().set_by_name(&simple_name, module_obj);
                }
                ImportItems::Specific(names) => {
                    let mut exports = HashMap::new();
                    for name in names {
                        if let Some(obj) = module.exports.get(&name) {
                            exports.insert(name, obj.clone());
                        } else {
                            return Object::Error(RuntimeError::InvalidOperation(
                                format!("Module {} has no export '{}'", module.name, name)
                            ));
                        }
                    }
                    let module_obj = Object::Module {
                        name: path.join("::"),
                        exports,
                    };
                    self_clone.env.lock().unwrap().set_by_name(&simple_name, module_obj);
                }
                ImportItems::Single(name) => {
                    if let Some(obj) = module.exports.get(&name) {
                        self_clone.env.lock().unwrap().set_by_name(&name, obj.clone());
                    } else {
                        return Object::Error(RuntimeError::InvalidOperation(
                            format!("Module {} has no export '{}'", module.name, name)
                        ));
                    }
                }
            }
            
            Object::Null
        }
    }
    
    pub fn async_eval_tuple_assign(&mut self, targets: Vec<Ident>, values: Vec<Expr>) -> impl Future<Output = Object> + Send + '_ {
        let mut self_clone = self.clone();
        async move {
            for ident in &targets {
                if self_clone.env.lock().unwrap().get_by_name(&ident.name).is_none() {
                    return Object::Error(RuntimeError::UndefinedVariable(ident.name.clone()));
                }
            }
            if targets.len() != values.len() {
                return Object::Error(RuntimeError::InvalidOperation(
                    format!("assignment mismatch: {} targets, {} values", targets.len(), values.len())
                ));
            }

            let mut evaluated_values = Vec::new();
            for expr in values {
                let obj = self_clone.eval_expr(expr).await;
                if let Object::Error(_) = obj {
                    return obj;
                }
                evaluated_values.push(obj);
            }

            for (ident, value) in targets.into_iter().zip(evaluated_values) {
                self_clone.register_ident(ident, value);
            }
            Object::Null
        }
    }
}