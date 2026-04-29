//! Struct operations: build, get field, set field, method call.

use crate::runtime::runtime_errors::RuntimeError;
use crate::runtime::constant_pool::ConstantPool;
use crate::runtime::obj::{HashMap, Object};
use ahash::HashMapExt;

/// Result of method call execution
pub enum MethodCallResult {
    /// Method needs to be called: its function object is on the stack
    NeedsCall,
    /// Method result is already computed and on the stack
    Done,
    /// Error occurred
    Error(Object),
}

pub fn execute_build_struct(stack: &mut Vec<Object>, field_count: u8) {
    let field_count = field_count as usize;
    if stack.len() < field_count + 1 {
        stack.push(Object::Error(RuntimeError::InvalidOperation(
            "Stack underflow on BuildStruct".to_string(),
        )));
        return;
    }

    let name_obj = stack.pop().unwrap();
    let name = match name_obj {
        Object::String(s) => s,
        _ => {
            stack.push(Object::Error(RuntimeError::InvalidOperation(
                "Struct name must be a string constant".to_string(),
            )));
            return;
        }
    };

    let mut fields = HashMap::new();
    for _ in 0..field_count {
        let value = stack.pop().unwrap();
        let field_name_obj = stack.pop().unwrap();
        let field_name = match field_name_obj {
            Object::String(s) => s,
            _ => {
                stack.push(Object::Error(RuntimeError::InvalidOperation(
                    "Struct field name must be a string".to_string(),
                )));
                return;
            }
        };
        fields.insert(field_name, value);
    }

    stack.push(Object::Struct {
        name,
        fields,
        methods: HashMap::new(),
        constants: ConstantPool::new(),
    });
}

pub fn execute_get_field(stack: &mut Vec<Object>) {
    let field_name_obj = match stack.pop() {
        Some(v) => v,
        None => {
            return stack.push(Object::Error(RuntimeError::InvalidOperation(
                "Stack underflow on GetField".to_string(),
            )))
        }
    };
    let field_name = match field_name_obj {
        Object::String(s) => s,
        _ => {
            return stack.push(Object::Error(RuntimeError::InvalidOperation(
                "Field name must be a string".to_string(),
            )))
        }
    };
    let struct_obj = match stack.pop() {
        Some(v) => v,
        None => {
            return stack.push(Object::Error(RuntimeError::InvalidOperation(
                "Stack underflow on GetField".to_string(),
            )))
        }
    };

    let result = match struct_obj {
        Object::Struct { fields, .. } => fields.get(&field_name).cloned().unwrap_or(Object::Null),
        Object::Module { exports, .. } => exports.get(&field_name).cloned().unwrap_or(Object::Null),
        other => Object::Error(RuntimeError::InvalidOperation(format!(
            "Cannot get field from {}",
            other.type_name(),
        ))),
    };

    stack.push(result);
}

pub fn execute_set_field(stack: &mut Vec<Object>) {
    let value = match stack.pop() {
        Some(v) => v,
        None => {
            return stack.push(Object::Error(RuntimeError::InvalidOperation(
                "Stack underflow on SetField".to_string(),
            )))
        }
    };
    let field_name_obj = match stack.pop() {
        Some(v) => v,
        None => {
            return stack.push(Object::Error(RuntimeError::InvalidOperation(
                "Stack underflow on SetField".to_string(),
            )))
        }
    };
    let field_name = match field_name_obj {
        Object::String(s) => s,
        _ => {
            return stack.push(Object::Error(RuntimeError::InvalidOperation(
                "Field name must be a string".to_string(),
            )))
        }
    };
    let struct_obj = match stack.pop() {
        Some(v) => v,
        None => {
            return stack.push(Object::Error(RuntimeError::InvalidOperation(
                "Stack underflow on SetField".to_string(),
            )))
        }
    };

    let result = match struct_obj {
        Object::Struct {
            name,
            mut fields,
            methods,
            constants,
        } => {
            fields.insert(field_name, value);
            Object::Struct {
                name,
                fields,
                methods,
                constants,
            }
        }
        other => Object::Error(RuntimeError::InvalidOperation(format!(
            "Cannot set field on {}",
            other.type_name(),
        ))),
    };

    stack.push(result);
}

pub fn execute_call_method(
    stack: &mut Vec<Object>,
    argc: usize,
) -> Result<MethodCallResult, RuntimeError> {
    // Stack layout before this function:
    // [... object, method_name, arg1, arg2, ..., argN]

    // We need to pop arguments first (they're on top), then method_name, then object
    let mut args = Vec::new();
    for _ in 0..argc {
        match stack.pop() {
            Some(v) => args.push(v),
            None => {
                return Ok(MethodCallResult::Error(Object::Error(
                    RuntimeError::InvalidOperation("Stack underflow: missing argument".to_string()),
                )))
            }
        }
    }
    args.reverse(); // Restore original order

    // Now pop method_name and object
    let method_name_obj = match stack.pop() {
        Some(v) => v,
        None => {
            return Ok(MethodCallResult::Error(Object::Error(
                RuntimeError::InvalidOperation("Stack underflow: missing method name".to_string()),
            )))
        }
    };

    let method_name = match method_name_obj {
        Object::String(s) => s,
        _ => {
            return Ok(MethodCallResult::Error(Object::Error(
                RuntimeError::InvalidOperation("Method name must be a string".to_string()),
            )))
        }
    };

    let struct_obj = match stack.pop() {
        Some(v) => v,
        None => {
            return Ok(MethodCallResult::Error(Object::Error(
                RuntimeError::InvalidOperation("Stack underflow: missing object".to_string()),
            )))
        }
    };

    match &struct_obj {
        Object::Struct { methods, .. } => {
            if let Some(method) = methods.get(&method_name) {
                stack.push(method.clone());
                for arg in args {
                    stack.push(arg);
                }
                Ok(MethodCallResult::NeedsCall) // caller should dispatch to execute_call
            } else {
                Ok(MethodCallResult::Error(Object::Error(
                    RuntimeError::InvalidOperation(format!("Method '{}' not found", method_name)),
                )))
            }
        }
        Object::Module { exports, .. } => {
            if let Some(method) = exports.get(&method_name) {
                stack.push(method.clone());
                for arg in args {
                    stack.push(arg);
                }
                Ok(MethodCallResult::NeedsCall)
            } else {
                Ok(MethodCallResult::Error(Object::Error(
                    RuntimeError::InvalidOperation(format!(
                        "Method '{}' not found on module",
                        method_name
                    )),
                )))
            }
        }
        _ => {
            // Handle built-in methods for other types
            match crate::runtime::builtins::methods::BuiltinMethods::call_method(
                struct_obj,
                &method_name,
                args,
            ) {
                Ok(result) => {
                    stack.push(result);
                    Ok(MethodCallResult::Done)
                }
                Err(e) => Ok(MethodCallResult::Error(Object::Error(e))),
            }
        }
    }
}