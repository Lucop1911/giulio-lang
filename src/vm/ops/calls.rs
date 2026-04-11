//! Function call, closure, and await operations.

use std::sync::{Arc, Mutex};

use crate::ast::ast::{Ident, Program};
use crate::errors::RuntimeError;
use crate::runtime::env::Environment;
use crate::runtime::module_registry::ModuleRegistry;
use crate::runtime::obj::Object;
use crate::vm::compiler::Compiler;
use crate::vm::frame::CallFrame;
use crate::vm::vm::{ExecResult, VirtualMachine};

pub fn execute_call(
    stack: &mut Vec<Object>,
    frames: &mut Vec<CallFrame>,
    module_registry: &Arc<Mutex<ModuleRegistry>>,
    globals: &Arc<Mutex<crate::runtime::env::Environment>>,
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

    match fn_obj {
        Object::Function(params, body, closure_env, _constants) => {
            let args: Vec<Object> = stack.drain(stack.len() - argc..).collect();
            stack.pop();

            let (chunk, _param_count, local_names) = Compiler::compile_function_body(&params, &body, false);

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
                Arc::new(chunk),
                slots_base,
                slot_count,
                Arc::new(Mutex::new(new_env)),
                local_names,
            );
            frames.push(frame);
            Ok(ExecResult::Continue)
        }
        Object::AsyncFunction(params, body, closure_env) => {
            let args: Vec<Object> = stack.drain(stack.len() - argc..).collect();
            stack.pop();

            // Advance caller's IP past OpCallAsync (1 byte opcode + 1 byte argc operand)
            if let Some(caller) = frames.last_mut() {
                caller.ip += 2;
            }

            let future = call_async_function_vm(params.to_vec(), body.clone(), args, closure_env.clone(), Arc::clone(module_registry), Arc::clone(globals));
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
                .map(|obj| g_to_component_val(obj))
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
                                            {
                                                let mut registry = module_registry.lock().unwrap();
                                                registry.wasm_store = store_opt.take();
                                            }
                                            
                                            // Convert results back to G-lang Objects
                                            match results.get(0) {
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
                                            {
                                                let mut registry = module_registry.lock().unwrap();
                                                registry.wasm_store = store_opt.take();
                                            }
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
    frames: &mut Vec<CallFrame>,
) {
    if let Some(top) = stack.pop() {
        match top {
            Object::Function(params, body, _old_env, constants) => {
                let captured = collect_captured_vars(&body, &params);
                let caller_env = frames.last().and_then(|f| f.closure_env.clone());
                let mut new_env = match caller_env {
                    Some(env) => Environment::new_with_outer(env),
                    None => Environment::new(),
                };

                if let Some(frame) = frames.last() {
                    for name in &captured {
                        if let Some(slot) = frame.local_names.iter().position(|n| n == name) {
                            let value = frame.get_local(stack, slot).clone();
                            new_env.set_by_name(name, value);
                        }
                    }
                }

                stack.push(Object::Function(
                    params,
                    body,
                    Arc::new(Mutex::new(new_env)),
                    constants,
                ));
            }
            Object::AsyncFunction(params, body, _old_env) => {
                let captured = collect_captured_vars(&body, &params);
                let caller_env = frames.last().and_then(|f| f.closure_env.clone());
                let mut new_env = match caller_env {
                    Some(env) => Environment::new_with_outer(env),
                    None => Environment::new(),
                };
                if let Some(frame) = frames.last() {
                    for name in &captured {
                        if let Some(slot) = frame.local_names.iter().position(|n| n == name) {
                            let value = frame.get_local(stack, slot).clone();
                            new_env.set_by_name(name, value);
                        }
                    }
                }
                stack.push(Object::AsyncFunction(
                    params,
                    body,
                    Arc::new(Mutex::new(new_env)),
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

fn call_async_function_vm(
    params: Vec<Ident>,
    body: Program,
    args: Vec<Object>,
    closure_env: Arc<Mutex<Environment>>,
    module_registry: Arc<Mutex<ModuleRegistry>>,
    caller_globals: Arc<Mutex<Environment>>,
) -> std::pin::Pin<
    Box<dyn std::future::Future<Output = Result<Object, RuntimeError>> + Send + 'static>,
> {
    Box::pin(async move {
        let (chunk, param_count, local_names) = Compiler::compile_function_body(&params, &body, true);
        
        // For top-level async functions, closure_env is minimal (just builtins + captured vars)
        // We need to also include caller_globals so they can access sibling functions
        // 
        // Strategy: if closure_env has a parent, use it as-is (it's a nested function)
        // Otherwise, use caller_globals as the base
        let base_env = {
            let closure_guard = closure_env.lock().unwrap();
            let has_parent = closure_guard.has_parent();
            drop(closure_guard);
            
            if has_parent {
                // Nested function - use closure_env as-is (it has its parent chain intact)
                Arc::clone(&closure_env)
            } else {
                // Top-level function - use caller_globals as the base
                Arc::clone(&caller_globals)
            }
        };
        
        let mut closure_env_inner = Environment::new_with_outer(base_env);
        
        // Set parameters
        for (i, param) in params.iter().enumerate() {
            if i < args.len() {
                closure_env_inner.set_by_name(&param.name, args[i].clone());
            }
        }
        
        let slot_count = std::cmp::max(64, param_count + 10);
        
        let globals_with_locals = Arc::new(Mutex::new(closure_env_inner));
        let mut vm = VirtualMachine::new_with_slots(
            Arc::clone(&globals_with_locals),
            module_registry,
            slot_count,
            args,
        );
        
        // Set the local names so closures can capture them
        vm.set_root_local_names(local_names);
        
        // Set the root frame's closure environment to be the same as globals
        // This allows functions defined inside this async function to access the async VM's globals
        vm.set_root_closure_env(Arc::clone(&globals_with_locals));

        let result = vm.run(Arc::new(chunk)).await;
        result
    })
}

fn collect_captured_vars(body: &Program, params: &[Ident]) -> Vec<String> {
    let param_names: Vec<&str> = params.iter().map(|p| p.name.as_str()).collect();
    let mut captured = Vec::new();
    for stmt in body {
        collect_from_stmt(stmt, &param_names, &mut captured);
    }
    captured.sort();
    captured.dedup();
    captured
}

fn collect_from_stmt(stmt: &crate::ast::ast::Stmt, param_names: &[&str], captured: &mut Vec<String>) {
    use crate::ast::ast::Stmt;
    match stmt {
        Stmt::LetStmt(_, expr) => collect_from_expr(expr, param_names, captured),
        Stmt::AssignStmt(ident, expr) => {
            if !param_names.contains(&ident.name.as_str())
                && !captured.contains(&ident.name)
            {
                captured.push(ident.name.clone());
            }
            collect_from_expr(expr, param_names, captured);
        }
        Stmt::ExprStmt(e) | Stmt::ExprValueStmt(e) | Stmt::ReturnStmt(e) | Stmt::ThrowStmt(e) => {
            collect_from_expr(e, param_names, captured);
        }
        Stmt::FnStmt { params, body, .. } => {
            let inner_params: Vec<&str> = params.iter().map(|p| p.name.as_str()).collect();
            for stmt in body {
                collect_from_stmt(stmt, &inner_params, captured);
            }
        }
        _ => {}
    }
}

fn collect_from_expr(expr: &crate::ast::ast::Expr, param_names: &[&str], captured: &mut Vec<String>) {
    use crate::ast::ast::Expr;
    match expr {
        Expr::IdentExpr(ident) => {
            if !param_names.contains(&ident.name.as_str())
                && !captured.contains(&ident.name)
            {
                captured.push(ident.name.clone());
            }
        }
        Expr::FnExpr { params, body } | Expr::AsyncFnExpr { params, body } => {
            let inner_params: Vec<&str> = params.iter().map(|p| p.name.as_str()).collect();
            for stmt in body {
                collect_from_stmt(stmt, &inner_params, captured);
            }
        }
        Expr::IfExpr { cond, consequence, alternative } => {
            collect_from_expr(cond, param_names, captured);
            for stmt in consequence {
                collect_from_stmt(stmt, param_names, captured);
            }
            if let Some(alt) = alternative {
                for stmt in alt {
                    collect_from_stmt(stmt, param_names, captured);
                }
            }
        }
        Expr::WhileExpr { cond, body } => {
            collect_from_expr(cond, param_names, captured);
            for stmt in body {
                collect_from_stmt(stmt, param_names, captured);
            }
        }
        Expr::ForExpr { iterable, body, .. } => {
            collect_from_expr(iterable, param_names, captured);
            for stmt in body {
                collect_from_stmt(stmt, param_names, captured);
            }
        }
        Expr::CStyleForExpr { init, cond, update, body } => {
            if let Some(i) = init { collect_from_stmt(i, param_names, captured); }
            if let Some(c) = cond { collect_from_expr(c, param_names, captured); }
            if let Some(u) = update { collect_from_stmt(u, param_names, captured); }
            for stmt in body {
                collect_from_stmt(stmt, param_names, captured);
            }
        }
        Expr::TryCatchExpr { try_body, catch_body, finally_body, .. } => {
            for stmt in try_body {
                collect_from_stmt(stmt, param_names, captured);
            }
            if let Some(cb) = catch_body {
                for stmt in cb {
                    collect_from_stmt(stmt, param_names, captured);
                }
            }
            if let Some(fb) = finally_body {
                for stmt in fb {
                    collect_from_stmt(stmt, param_names, captured);
                }
            }
        }
        Expr::PrefixExpr(_, e) => collect_from_expr(e, param_names, captured),
        Expr::InfixExpr(_, l, r) => {
            collect_from_expr(l, param_names, captured);
            collect_from_expr(r, param_names, captured);
        }
        Expr::CallExpr { function, arguments } => {
            collect_from_expr(function, param_names, captured);
            for a in arguments {
                collect_from_expr(a, param_names, captured);
            }
        }
        Expr::ArrayExpr(es) => {
            for e in es { collect_from_expr(e, param_names, captured); }
        }
        Expr::HashExpr(kvs) => {
            for (k, v) in kvs {
                collect_from_expr(k, param_names, captured);
                collect_from_expr(v, param_names, captured);
            }
        }
        Expr::IndexExpr { array, index } => {
            collect_from_expr(array, param_names, captured);
            collect_from_expr(index, param_names, captured);
        }
        Expr::MethodCallExpr { object, arguments, .. } => {
            collect_from_expr(object, param_names, captured);
            for a in arguments {
                collect_from_expr(a, param_names, captured);
            }
        }
        Expr::StructLiteral { fields, .. } => {
            for (_, e) in fields { collect_from_expr(e, param_names, captured); }
        }
        Expr::FieldAccessExpr { object, .. } => {
            collect_from_expr(object, param_names, captured);
        }
        Expr::AwaitExpr(e) => collect_from_expr(e, param_names, captured),
        Expr::LitExpr(_) | Expr::ThisExpr | Expr::LitIndex(_) => {}
    }
}
