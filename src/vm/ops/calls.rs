//! Function call, closure, and await operations.

use std::sync::{Arc, Mutex};

use crate::ast::ast::Ident;
use crate::vm::runtime::runtime_errors::RuntimeError;
use crate::vm::runtime::env::Environment;
use crate::vm::runtime::module_registry::ModuleRegistry;
use crate::vm::obj::Object;
use crate::vm::frame::CallFrame;
use crate::vm::vm::{ExecResult, VirtualMachine};

pub fn execute_call(
    stack: &mut Vec<Object>,
    frames: &mut Vec<CallFrame>,
    module_registry: &Arc<Mutex<ModuleRegistry>>,
    globals: &Arc<Mutex<crate::vm::runtime::env::Environment>>,
    argc: usize,
) -> Result<ExecResult, RuntimeError> {
    if stack.len() < argc + 1 {
        stack.push(Object::Error(RuntimeError::InvalidOperation(
            "Stack underflow on Call".to_string(),
        )));
        return Ok(ExecResult::Continue);
    }

    let fn_idx = stack.len() - argc - 1;
    let fn_obj = stack[fn_idx].clone();
    if let Object::Error(_) = fn_obj {
        println!("execute_call: fn_obj is an Error: {:?}", fn_obj);
    }

    match fn_obj {
        Object::Function(params, chunk, closure_env, local_names) => {
            let caller_stack_len = stack.len() - argc - 1;
            let args: Vec<Object> = stack.drain(stack.len() - argc..).collect();
            stack.pop();

            let slots_base = stack.len();
            let slot_count = params.len().max(argc) + 64;
            stack.resize(slots_base + slot_count, Object::Null);

            for (i, arg) in args.iter().enumerate() {
                if i < slot_count {
                    stack[slots_base + i] = arg.clone();
                }
            }

            let mut new_env = Environment::new_with_outer(Arc::clone(&closure_env));

            for (i, param) in params.iter().enumerate() {
                if i < args.len() {
                    new_env.set_by_name(&param.name, args[i].clone());
                }
            }

            if let Some(caller) = frames.last_mut() {
                caller.ip += 2;
            }

            let frame = CallFrame::new_function(
                Arc::clone(&chunk),
                slots_base,
                slot_count,
                caller_stack_len,
                Arc::new(Mutex::new(new_env)),
                local_names,
            );
            frames.push(frame);
            Ok(ExecResult::Continue)
        }
        Object::AsyncFunction(params, chunk, closure_env, local_names) => {
            let args: Vec<Object> = stack.drain(stack.len() - argc..).collect();
            stack.pop();

            // Advance caller's IP past OpCallAsync (1 byte opcode + 1 byte argc operand)
            if let Some(caller) = frames.last_mut() {
                caller.ip += 2;
            }

            let future = call_async_function_vm(params.to_vec(), chunk, local_names, args, closure_env.clone(), Arc::clone(module_registry), Arc::clone(globals));
            stack.push(Object::Future(Arc::new(Mutex::new(Some(future)))));
            Ok(ExecResult::Continue)
        }
        Object::BuiltinStd(_name, min_param, max_param, func) => {
            let args: Vec<Object> = stack.drain(stack.len() - argc..).collect();
            stack.pop();

            if argc < min_param || argc > max_param {
                return Ok(ExecResult::ContinueWith(Object::Error(
                    RuntimeError::WrongNumberOfArguments {
                        min: min_param,
                        max: max_param,
                        got: argc,
                    },
                )));
            }

            match func(args) {
                Ok(result) => {
                    stack.push(result);
                    Ok(ExecResult::Continue)
                }
                Err(e) => Ok(ExecResult::ContinueWith(Object::Error(e))),
            }
        }
        Object::BuiltinStdAsync(_name, min_param, max_param, func) => {
            let args: Vec<Object> = stack.drain(stack.len() - argc..).collect();
            stack.pop();

            if argc < min_param || argc > max_param {
                return Ok(ExecResult::ContinueWith(Object::Error(
                    RuntimeError::WrongNumberOfArguments {
                        min: min_param,
                        max: max_param,
                        got: argc,
                    },
                )));
            }

            match func(args) {
                Ok(obj) => {
                    stack.push(obj);
                    Ok(ExecResult::Continue)
                }
                Err(e) => {
                    Ok(ExecResult::ContinueWith(Object::Error(e)))
                }
            }
        }
        Object::Builtin(_name, min_param, max_param, func) => {
            let args: Vec<Object> = stack.drain(stack.len() - argc..).collect();
            stack.pop();

            if argc < min_param || argc > max_param {
                return Ok(ExecResult::ContinueWith(Object::Error(
                    RuntimeError::WrongNumberOfArguments {
                        min: min_param,
                        max: max_param,
                        got: argc,
                    },
                )));
            }

            match func(args) {
                Ok(result) => {
                    stack.push(result);
                    Ok(ExecResult::Continue)
                }
                Err(e) => Ok(ExecResult::ContinueWith(Object::Error(
                    RuntimeError::InvalidOperation(e),
                ))),
            }
        }
        #[cfg(feature = "wasm")]
        Object::WasmImportedFunction {
            module_name: _,
            func_name,
            instance,
        } => {
            let args: Vec<Object> = stack.drain(stack.len() - argc..).collect();
            stack.pop();

            // Convert G-lang Objects to WASM values
            use crate::wasm::type_conversions::g_to_component_val;
            let wasm_args: Result<Vec<_>, _> = args
                .iter()
                .map(g_to_component_val)
                .collect();

            match wasm_args {
                Ok(wasm_args) => {
                    let mut instance_guard = instance.lock().unwrap();
                    match instance_guard.as_mut() {
                        Some(wasm_instance) => {
                            // Get the runtime and store from the registry
                            let mut store_opt = {
                                let mut registry = module_registry.lock().unwrap();
                                registry.wasm_store.take()
                            };

                            match store_opt.as_mut() {
                                Some(store) => {
                                    match wasm_instance.call_func_with_args(store, &func_name, &wasm_args)
                                    {
                                        Ok(results) => {
                                            use crate::wasm::type_conversions::component_val_to_g;
                                            // Put the store back
                                            let mut registry = module_registry.lock().unwrap();
                                            registry.wasm_store = store_opt.take();
                                            
                                            // Convert results back to G-lang Objects
                                            match results.first() {
                                                Some(val) => {
                                                    match component_val_to_g(val) {
                                                        Ok(obj) => {
                                                            stack.push(obj);
                                                            Ok(ExecResult::Continue)
                                                        }
                                                        Err(e) => {
                                                            Ok(ExecResult::ContinueWith(Object::Error(e)))
                                                        }
                                                    }
                                                }
                                                None => {
                                                    // Function returned nothing
                                                    stack.push(Object::Null);
                                                    Ok(ExecResult::Continue)
                                                }
                                            }
                                        }
                                        Err(e) => {
                                            // Put the store back even on error
                                            let mut registry = module_registry.lock().unwrap();
                                            registry.wasm_store = store_opt.take();
                                            Ok(ExecResult::ContinueWith(Object::Error(e)))
                                        }
                                    }
                                }
                                None => Ok(ExecResult::ContinueWith(Object::Error(
                                    RuntimeError::InvalidOperation(
                                        "WASM store not available".to_string(),
                                    ),
                                ))),
                            }
                        }
                        None => Ok(ExecResult::ContinueWith(Object::Error(
                            RuntimeError::InvalidOperation(
                                "WASM instance has been consumed".to_string(),
                            ),
                        ))),
                    }
                }
                Err(e) => Ok(ExecResult::ContinueWith(Object::Error(e))),
            }
        }
        _ => Ok(ExecResult::ContinueWith(Object::Error(
            RuntimeError::NotCallable(fn_obj.type_name()),
        ))),
    }
}

pub fn execute_closure(
    stack: &mut Vec<Object>,
    frames: &mut [CallFrame],
) {
    if let Some(top) = stack.pop() {
        match top {
            Object::Function(params, chunk, _old_env, local_names) => {
                // Only capture names from the *outer* scope, not current local_names.
                // They are identified by checking what's currently in the caller's frame local_names.
                let new_env = if let Some(caller) = frames.last() {
                    // Use closure_env if present, otherwise just use a new root environment.
                    // If at root - the global scope must be linked
                    let outer_env = caller.closure_env.clone().unwrap_or_else(|| {
                         Arc::new(Mutex::new(Environment::new_root()))
                    });

                    let mut env = Environment::new_with_outer(outer_env);

                    // Capture variables that are present in the caller's frame.
                    for name in &caller.local_names {
                        if !name.is_empty()
                            && let Some(slot) = caller.local_names.iter().position(|n| n == name) {
                                let value = caller.get_local(stack, slot).clone();
                                env.set_by_name(name, value);
                        }
                    }
                    env
                } else {
                    Environment::new_root()
                };

                stack.push(Object::Function(
                    params,
                    chunk,
                    Arc::new(Mutex::new(new_env)),
                    local_names,
                ));
            }
            Object::AsyncFunction(params, chunk, _old_env, local_names) => {
                let new_env = if let Some(caller) = frames.last() {
                    let outer_env = caller.closure_env.clone().unwrap_or_else(|| {
                        Arc::new(Mutex::new(Environment::new_root()))
                    });
                    
                    let mut env = Environment::new_with_outer(outer_env);
                    for name in &caller.local_names {
                        if !name.is_empty()
                            && let Some(slot) = caller.local_names.iter().position(|n| n == name) {
                                let value = caller.get_local(stack, slot).clone();
                                env.set_by_name(name, value);
                        }
                    }
                    env
                } else {
                    Environment::new()
                };
                stack.push(Object::AsyncFunction(
                    params,
                    chunk,
                    Arc::new(Mutex::new(new_env)),
                    local_names,
                ));
            }
            other => {
                stack.push(other);
            }
        }
    }
}

pub fn execute_return_value() -> ExecResult {
    ExecResult::Return
}

pub fn call_async_function_vm(
    params: Vec<Ident>,
    chunk: Arc<crate::vm::chunk::Chunk>,
    local_names: Vec<String>,
    args: Vec<Object>,
    closure_env: Arc<Mutex<Environment>>,
    module_registry: Arc<Mutex<ModuleRegistry>>,
    caller_globals: Arc<Mutex<Environment>>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<Object, RuntimeError>> + Send + 'static>> {
    Box::pin(async move {
        // Use the captured closure_env as the parent for the new environment.
        // This ensures the async function's scope inherits both captured variables 
        // and the global scope (via the closure_env's parent chain).
        let mut closure_env_inner = Environment::new_with_outer(Arc::clone(&closure_env));
        
        for (i, param) in params.iter().enumerate() {
            if i < args.len() {
                closure_env_inner.set_by_name(&param.name, args[i].clone());
            }
        }
        
        let slot_count = std::cmp::max(64, params.len() + 10);
        
        let globals_with_locals = Arc::new(Mutex::new(closure_env_inner));
        let mut vm = VirtualMachine::new_with_slots(
            Arc::clone(&caller_globals), // Still use original globals for the VM's global context
            module_registry,
            slot_count,
            args,
        );
        
        vm.set_root_local_names(local_names);
        vm.set_root_closure_env(Arc::clone(&globals_with_locals));

        return vm.run(Arc::clone(&chunk)).await;
    })
}
