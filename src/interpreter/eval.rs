use std::future::Future;
use std::pin::Pin;
use std::sync::{Arc, Mutex};

use crate::{
    ast::ast::{Expr, Ident, Program, Stmt},
    errors::RuntimeError,
    interpreter::{
        env::Environment, module_registry::ModuleRegistry, obj::Object
    },
};

pub type EvalFuture = Pin<Box<dyn Future<Output = Object> + Send + 'static>>;

pub struct Evaluator {
    pub(crate) env: Arc<Mutex<Environment>>,
    pub(crate) module_registry: Arc<Mutex<ModuleRegistry>>,
    pub(crate) in_async_context: bool,
}

impl Clone for Evaluator {
    fn clone(&self) -> Self {
        Evaluator {
            env: Arc::clone(&self.env),
            module_registry: Arc::clone(&self.module_registry),
            in_async_context: self.in_async_context,
        }
    }
}

impl Default for Evaluator {
    fn default() -> Self {
        let base_path = std::env::current_dir()
            .expect("failed to get current directory");

        let registry = Arc::new(Mutex::new(
            ModuleRegistry::new(base_path),
        ));

        Evaluator {
            env: Arc::new(Mutex::new(Environment::new())),
            module_registry: registry,
            in_async_context: false,
        }
    }
}

impl Evaluator {
    pub fn new(module_registry: Arc<Mutex<ModuleRegistry>>) -> Self {
        Evaluator {
            env: Arc::new(Mutex::new(Environment::new())),
            module_registry,
            in_async_context: false,
        }
    }

    pub(crate) fn returned(&mut self, object: Object) -> Object {
        match object {
            Object::ReturnValue(v) => *v,
            o => o,
        }
    }

    pub fn eval_program(&mut self, prog: Program) -> impl Future<Output = Object> + Send + '_  {
        let mut self_clone = self.clone();
        async move {
            let return_data = self_clone.eval_blockstmt(&prog).await;
            self_clone.returned(return_data)
        }
    }

    pub fn eval_blockstmt<'a>(&'a mut self, prog: &'a Program) -> impl Future<Output = Object> + Send + 'a {
        let mut self_clone = self.clone();
        async move {
            let mut result = Object::Null;

            for stmt in prog.iter() {
                result = self_clone.eval_statement(stmt.clone()).await;
                match result {
                    Object::ReturnValue(_) | Object::Break | Object::Continue | Object::Error(_) | Object::ThrownValue(_) => {
                        return result;
                    }
                    _ => {}
                }
            }
            result
        }
    }

    pub fn eval_statement(&mut self, stmt: Stmt) -> EvalFuture {
        let mut self_clone = self.clone();
        Box::pin(async move {
            match stmt {
                Stmt::ExprStmt(expr) => {
                    let result = self_clone.eval_expr(expr).await;
                    // Propagate control flow objects (return, break, continue, error, thrown)
                    match result {
                        Object::ReturnValue(_) | Object::Break | Object::Continue | Object::Error(_) | Object::ThrownValue(_) => result,
                        _ => result  // Expression statements now produce values in normal flow
                    }
                }
                Stmt::ExprValueStmt(expr) => {
                    self_clone.eval_expr(expr).await
                }
                Stmt::ReturnStmt(expr) => Object::ReturnValue(Box::new(self_clone.eval_expr(expr).await)),
                Stmt::LetStmt(ident, expr) => {
                    let object = self_clone.eval_expr(expr).await;
                    self_clone.register_ident(ident, object)
                }
                Stmt::FnStmt { name, params, body } => {
                    let fn_obj = Object::Function(params, body, Arc::clone(&self_clone.env));
                    self_clone.register_ident(name, fn_obj)
                }
                Stmt::AssignStmt(ident, expr) => {
                    // Check if variable exists
                    let Ident(ref name) = ident;
                    if self_clone.env.lock().unwrap().get(name).is_none() {
                        return Object::Error(RuntimeError::UndefinedVariable(name.clone()));
                    }
                    // Reassign the variable
                    let object = self_clone.eval_expr(expr).await;
                    self_clone.register_ident(ident, object)
                }
                Stmt::FieldAssignStmt { object, field, value } => {
                    self_clone.async_eval_field_assign(*object, field, *value).await
                }
                Stmt::IndexAssignStmt { target, index, value } => {
                    self_clone.async_eval_index_assign(*target, *index, *value).await
                }
                Stmt::StructStmt { name, fields, methods } => {
                    self_clone.async_eval_struct_def(name, fields, methods).await
                }
                Stmt::ImportStmt { path, items } => {
                    self_clone.async_eval_import(path, items).await
                }
                Stmt::BreakStmt => Object::Break,
                Stmt::ContinueStmt => Object::Continue,
                Stmt::ThrowStmt(expr) => Object::ThrownValue(Box::new(self_clone.eval_expr(expr).await)),
            }
        })
    }

    pub fn eval_expr(&mut self, expr: Expr) -> EvalFuture {
        let mut self_clone = self.clone();
        Box::pin(async move {
            match expr {
                Expr::IdentExpr(i) => self_clone.eval_ident(i),
                Expr::LitExpr(l) => self_clone.eval_literal(l),
                Expr::PrefixExpr(prefix, expr) => self_clone.async_eval_prefix(prefix, *expr).await,
                Expr::InfixExpr(infix, expr1, expr2) => self_clone.async_eval_infix(infix, *expr1, *expr2).await,
                Expr::IfExpr {
                    cond,
                    consequence,
                    alternative,
                } => self_clone.async_eval_if(*cond, consequence, alternative).await,
                Expr::FnExpr { params, body } => self_clone.eval_fn(params, body),
                Expr::AsyncFnExpr { params, body } => {
                    Object::AsyncFunction(params, body, Arc::clone(&self_clone.env))
                },
                Expr::AwaitExpr(expr) => {
                    let future_obj = self_clone.eval_expr(*expr).await;
                    match future_obj {
                        Object::Future(future_arc) => {
                            let future_to_await = {
                                let mut future_opt_guard = future_arc.lock().unwrap();
                                let future = future_opt_guard.take();
                                drop(future_opt_guard); // Explicitly drop the guard
                                if let Some(f) = future { f } else {
                                    return Object::Error(RuntimeError::InvalidOperation(
                                        "Cannot await a future that has already been awaited".to_string()
                                    ));
                                }
                            };
                            match future_to_await.await {
                                Ok(obj) => obj,
                                Err(e) => Object::Error(e),
                            }
                        }
                        _ => Object::Error(RuntimeError::InvalidOperation(
                            format!("Cannot await non-future type: {}", future_obj.type_name())
                        )),
                    }
                },
                Expr::CallExpr {
                    function: fn_exp,
                    arguments,
                } => self_clone.async_eval_call(*fn_exp, arguments).await,
                Expr::ArrayExpr(exprs) => self_clone.async_eval_array(exprs).await,
                Expr::HashExpr(hash_exprs) => self_clone.async_eval_hash(hash_exprs).await,
                Expr::IndexExpr { array, index } => self_clone.async_eval_index(*array, *index).await,
                Expr::MethodCallExpr { object, method, arguments } => {
                    self_clone.async_eval_method_call(*object, method, arguments).await
                }
                Expr::StructLiteral { name, fields } => {
                    self_clone.async_eval_struct_literal(name, fields).await
                }
                Expr::ThisExpr => {
                    self_clone.eval_this()
                }
                Expr::FieldAccessExpr { object, field } => {
                    self_clone.async_eval_field_access(*object, field).await
                }
                Expr::WhileExpr { cond, body } => self_clone.async_eval_while(cond, body).await,
                Expr::ForExpr { ident, iterable, body } => self_clone.async_eval_for(ident, iterable, body).await,
                Expr::CStyleForExpr { init, cond, update, body } => {
                    self_clone.async_eval_c_style_for(init, cond, update, body).await
                }
                Expr::TryCatchExpr { try_body, catch_ident, catch_body, finally_body } => {
                    self_clone.async_eval_try_catch_expr(try_body, catch_ident, catch_body, finally_body).await
                }
            }
        })
    }

    pub fn eval_expr_sync(&mut self, expr: Expr) -> Object {
        match expr {
            Expr::IdentExpr(i) => self.eval_ident(i),
            Expr::LitExpr(l) => self.eval_literal(l),
            Expr::PrefixExpr(prefix, expr) => self.eval_prefix(prefix, *expr),
            Expr::InfixExpr(infix, expr1, expr2) => self.eval_infix(infix, *expr1, *expr2),
            Expr::IfExpr { cond, consequence, alternative } => {
                let cond_result = self.eval_expr_sync(*cond);
                match self.obj_to_bool(cond_result) {
                    Ok(true) => self.eval_blockstmt_sync(&consequence),
                    Ok(false) => {
                        match alternative {
                            Some(alt) => self.eval_blockstmt_sync(&alt),
                            None => Object::Null,
                        }
                    }
                    Err(e) => e,
                }
            }
            Expr::FnExpr { params, body } => self.eval_fn(params, body),
            Expr::AsyncFnExpr { params, body } => {
                Object::AsyncFunction(params, body, Arc::clone(&self.env))
            },
            Expr::AwaitExpr(_) => {
                Object::Error(RuntimeError::InvalidOperation(
                    "await not allowed in sync context".to_string()
                ))
            },
            Expr::CallExpr { .. } => {
                Object::Error(RuntimeError::InvalidOperation(
                    "function calls not allowed in sync context".to_string()
                ))
            }
            Expr::ArrayExpr(exprs) => {
                let mut new_vec = Vec::new();
                for e in exprs {
                    new_vec.push(self.eval_expr_sync(e));
                }
                Object::Array(new_vec)
            }
            Expr::HashExpr(hash_exprs) => {
                use std::hash::BuildHasherDefault;
                use ahash::{AHasher, HashMapExt};
                type HashMap<K, V> = std::collections::HashMap<K, V, BuildHasherDefault<AHasher>>;
                let mut hashmap = HashMap::new();
                
                for (key_expr, val_expr) in hash_exprs {
                    let key = self.eval_expr_sync(key_expr);
                    let val = self.eval_expr_sync(val_expr);
                    
                    match &key {
                        Object::Integer(_) | Object::Boolean(_) | Object::String(_) => {
                            hashmap.insert(key, val);
                        }
                        Object::Error(e) => return Object::Error(e.clone()),
                        _ => return Object::Error(RuntimeError::NotHashable(key.type_name())),
                    }
                }
                
                Object::Hash(hashmap)
            }
            Expr::IndexExpr { array, index } => {
                let target = self.eval_expr_sync(*array);
                let idx = self.eval_expr_sync(*index);
                match target {
                    Object::Array(arr) => match self.obj_to_int(idx) {
                        Ok(index_number) => {
                            if index_number < 0 {
                                return Object::Error(RuntimeError::IndexOutOfBounds {
                                    index: index_number,
                                    length: arr.len(),
                                });
                            }
                            let idx = index_number as usize;
                            if idx >= arr.len() {
                                return Object::Error(RuntimeError::IndexOutOfBounds {
                                    index: index_number,
                                    length: arr.len(),
                                });
                            }
                            arr.into_iter().nth(idx).unwrap_or(Object::Null)
                        }
                        Err(err) => err,
                    },
                    Object::Hash(mut hash) => {
                        let name = self.obj_to_hash(idx);
                        match name {
                            Object::Error(_) => name,
                            _ => hash.remove(&name).unwrap_or(Object::Null),
                        }
                    }
                    o => Object::Error(RuntimeError::NotHashable(o.type_name())),
                }
            }
            Expr::MethodCallExpr { .. } => {
                Object::Error(RuntimeError::InvalidOperation(
                    "method calls not allowed in sync context".to_string()
                ))
            }
            Expr::StructLiteral { .. } => {
                Object::Error(RuntimeError::InvalidOperation(
                    "struct literals not allowed in sync context".to_string()
                ))
            }
            Expr::ThisExpr => self.eval_this(),
            Expr::FieldAccessExpr { .. } => {
                Object::Error(RuntimeError::InvalidOperation(
                    "field access not allowed in sync context".to_string()
                ))
            }
            Expr::WhileExpr { .. } => {
                Object::Error(RuntimeError::InvalidOperation(
                    "while loops not allowed in sync context".to_string()
                ))
            }
            Expr::ForExpr { .. } => {
                Object::Error(RuntimeError::InvalidOperation(
                    "for loops not allowed in sync context".to_string()
                ))
            }
            Expr::CStyleForExpr { .. } => {
                Object::Error(RuntimeError::InvalidOperation(
                    "for loops not allowed in sync context".to_string()
                ))
            }
            Expr::TryCatchExpr { .. } => {
                Object::Error(RuntimeError::InvalidOperation(
                    "try-catch not allowed in sync context".to_string()
                ))
            }
        }
    }

    pub fn eval_blockstmt_sync(&mut self, prog: &Program) -> Object {
        let mut result = Object::Null;

        for stmt in prog.iter() {
            result = self.eval_statement_sync(stmt.clone());
            match result {
                Object::ReturnValue(_) | Object::Break | Object::Continue | Object::Error(_) | Object::ThrownValue(_) => {
                    return result;
                }
                _ => {}
            }
        }
        result
    }

    pub fn eval_statement_sync(&mut self, stmt: Stmt) -> Object {
        match stmt {
            Stmt::ExprStmt(expr) => self.eval_expr_sync(expr),
            Stmt::ExprValueStmt(expr) => self.eval_expr_sync(expr),
            Stmt::ReturnStmt(expr) => Object::ReturnValue(Box::new(self.eval_expr_sync(expr))),
            Stmt::LetStmt(ident, expr) => {
                let object = self.eval_expr_sync(expr);
                self.register_ident(ident, object)
            }
            Stmt::FnStmt { name, params, body } => {
                let fn_obj = Object::Function(params, body, Arc::clone(&self.env));
                self.register_ident(name, fn_obj)
            }
            Stmt::AssignStmt(ident, expr) => {
                let Ident(ref name) = ident;
                if self.env.lock().unwrap().get(name).is_none() {
                    return Object::Error(RuntimeError::UndefinedVariable(name.clone()));
                }
                let object = self.eval_expr_sync(expr);
                self.register_ident(ident, object)
            }
            _ => Object::Error(RuntimeError::InvalidOperation(
                "statement not allowed in sync context".to_string()
            )),
        }
    }

}