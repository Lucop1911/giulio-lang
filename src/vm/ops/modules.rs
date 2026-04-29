//! Module import and export operations.

use crate::vm::runtime::runtime_errors::RuntimeError;
use crate::vm::runtime::module_registry::ModuleRegistry;
use crate::vm::obj::Object;
use crate::vm::chunk::Chunk;
use std::sync::{Arc, Mutex};

pub fn execute_import_module(
    stack: &mut Vec<Object>,
    chunk: &Chunk,
    module_registry: &Arc<Mutex<ModuleRegistry>>,
    idx: u16,
) -> Result<(), RuntimeError> {
    let path_obj = &chunk.constants[idx as usize];
    let path = match path_obj {
        Object::String(s) => s.clone(),
        _ => {
            stack.push(Object::Error(RuntimeError::InvalidOperation(
                "Import path must be a string".to_string(),
            )));
            return Ok(());
        }
    };

    let parts: Vec<String> = path.split("::").map(|s| s.to_string()).collect();
    let module = futures::executor::block_on(ModuleRegistry::load_module(
        Arc::clone(module_registry),
        &parts,
    ));

    match module {
        Ok(m) => {
            stack.push(Object::Module {
                name: m.name,
                exports: m.exports,
            });
        }
        Err(e) => {
            stack.push(Object::Error(e));
        }
    }

    Ok(())
}

pub fn execute_get_export(stack: &mut Vec<Object>) {
    let export_name_obj = match stack.pop() {
        Some(v) => v,
        None => {
            return stack.push(Object::Error(RuntimeError::InvalidOperation(
                "Stack underflow on GetExport".to_string(),
            )))
        }
    };
    let export_name = match export_name_obj {
        Object::String(s) => s,
        _ => {
            return stack.push(Object::Error(RuntimeError::InvalidOperation(
                "Export name must be a string".to_string(),
            )))
        }
    };
    let module_obj = match stack.pop() {
        Some(v) => v,
        None => {
            return stack.push(Object::Error(RuntimeError::InvalidOperation(
                "Stack underflow on GetExport".to_string(),
            )))
        }
    };

    let result = match module_obj {
        Object::Module { exports, .. } => {
            exports.get(&export_name).cloned().unwrap_or(Object::Null)
        }
        other => Object::Error(RuntimeError::InvalidOperation(format!(
            "Cannot get export from {}",
            other.type_name(),
        ))),
    };

    stack.push(result);
}