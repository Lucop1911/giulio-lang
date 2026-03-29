use std::future::Future;
use std::pin::Pin;
use std::sync::{Arc, Mutex};
use crate::interpreter::eval_context::EvalContext;
use crate::interpreter::helpers::eval_sync::eval_expressions::{
    eval_ident_sync, eval_infix_sync, eval_prefix_sync, register_ident_sync,
};
use crate::interpreter::helpers::type_converters::{obj_to_bool, obj_to_hash, obj_to_int};
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
    pub(crate) context: EvalContext,
}

impl Clone for Evaluator {
    fn clone(&self) -> Self {
        Evaluator {
            env: Arc::clone(&self.env),
            module_registry: Arc::clone(&self.module_registry),
            in_async_context: self.in_async_context,
            context: self.context.clone(),
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

        let env = Arc::new(Mutex::new(Environment::new()));
        let context = EvalContext::new(Arc::clone(&env), Arc::clone(&registry));
        Evaluator {
            env,
            module_registry: registry,
            in_async_context: false,
            context,
        }
    }
}

impl Evaluator {
    pub fn new(module_registry: Arc<Mutex<ModuleRegistry>>) -> Self {
        let env = Arc::new(Mutex::new(Environment::new()));
        let context = EvalContext::new(Arc::clone(&env), Arc::clone(&module_registry));
        Evaluator {
            env,
            module_registry,
            in_async_context: false,
            context,
        }
    }

    pub(crate) fn returned(&mut self, object: Object) -> Object {
        match object {
            Object::ReturnValue(v) => *v,
            o => o,
        }
    }

    pub fn eval_program(&self, prog: Program) -> EvalFuture {
        let mut self_clone = self.clone();
        Box::pin(async move {
            let return_data = self_clone.eval_blockstmt(&prog).await;
            self_clone.returned(return_data)
        })
    }

    pub fn eval_blockstmt(&self, prog: &Program) -> impl Future<Output = Object> + Send + '_ {
        let self_clone = self.clone();
        let prog_clone = (*prog).clone();
        Box::pin(async move {
            let mut result = Object::Null;

            for stmt in prog_clone.iter() {
                result = self_clone.eval_statement(stmt.clone()).await;
                match result {
                    Object::ReturnValue(_) | Object::Break | Object::Continue | Object::Error(_) | Object::ThrownValue(_) => {
                        return result;
                    }
                    _ => {}
                }
            }
            result
        })
    }

    pub fn eval_statement(&self, stmt: Stmt) -> EvalFuture {
        let mut self_clone = self.clone();
        Box::pin(async move {
            match stmt {
                Stmt::ExprStmt(expr) => {
                    let result = self_clone.eval_expr(expr).await;
                    match result {
                        Object::ReturnValue(_) | Object::Break | Object::Continue | Object::Error(_) | Object::ThrownValue(_) => result,
                        _ => result
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
                Stmt::MultiLetStmt { idents, values } => {
                    let mut objects = Vec::new();
                    for (id, expr) in idents.into_iter().zip(values.into_iter()) {
                        let object = self_clone.eval_expr(expr).await;
                        self_clone.register_ident(id, object.clone());
                        objects.push(object);
                    }
                    Object::Array(objects)
                }
                Stmt::FnStmt { name, mut params, body } => {
                    for (i, param) in params.iter_mut().enumerate() {
                        if param.slot.is_unset() {
                            param.slot = crate::ast::ast::SlotIndex(i as u16);
                        }
                    }
                    let fn_obj = Object::Function(params, body, Arc::clone(&self_clone.env));
                    self_clone.register_ident(name, fn_obj)
                }
                Stmt::AssignStmt(ident, expr) => {
                    let Ident { ref name, .. } = ident;
                    if self_clone.context.env.lock().unwrap().get_by_name(name).is_none() {
                        return Object::Error(RuntimeError::UndefinedVariable(name.clone()));
                    }
                    let object = self_clone.eval_expr(expr).await;
                    self_clone.register_ident(ident, object)
                }
                Stmt::TupleAssignStmt { targets, values } => {
                    self_clone.async_eval_tuple_assign(targets, values).await
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

    pub fn eval_expr(&self, expr: Expr) -> EvalFuture {
        let mut self_clone = self.clone();
        Box::pin(async move {
            match expr {
                Expr::IdentExpr(i) => self_clone.eval_ident(i),
                Expr::LitExpr(l) => self_clone.eval_literal(&l),
                Expr::PrefixExpr(prefix, expr) => {
                    let obj = self_clone.eval_expr(*expr).await;
                    self_clone.eval_prefix(prefix, obj)
                }
                Expr::InfixExpr(infix, expr1, expr2) => {
                    let obj1 = self_clone.eval_expr(*expr1).await;
                    let obj2 = self_clone.eval_expr(*expr2).await;
                    self_clone.eval_infix(infix, obj1, obj2)
                }
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
                                drop(future_opt_guard);
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

    /* 
    eval_expr_sync is used for simple expressions in which i know there's no async code like:
        - c-style for loops init, conditions and updates,
        - prefix and infix operations
        - array, hash and index accesses)
    everything else is async and wrapped in async move
    */

    pub fn eval_expr_sync(&self, env: &mut Environment, expr: &Expr) -> Object {
        match expr {
            Expr::IdentExpr(i) => eval_ident_sync(env, i),
            Expr::LitExpr(l) => self.eval_literal(l),
            Expr::PrefixExpr(prefix, expr) => {
                let obj = Self::eval_expr_sync(self, env, expr);
                eval_prefix_sync(env, prefix, obj)
            }
            Expr::InfixExpr(infix, expr1, expr2) => {
                let obj1 = Self::eval_expr_sync(self, env, expr1);
                let obj2 = Self::eval_expr_sync(self, env, expr2);
                eval_infix_sync(env, infix, obj1, obj2)
            }
            Expr::IfExpr { cond, consequence, alternative } => {
                let cond_result = Self::eval_expr_sync(self, env, cond);
                match obj_to_bool(cond_result) {
                    Ok(true) => Self::eval_blockstmt_sync(self, env, consequence),
                    Ok(false) => {
                        match alternative {
                            Some(alt) => Self::eval_blockstmt_sync(self, env, alt),
                            None => Object::Null,
                        }
                    }
                    Err(e) => e,
                }
            }
            Expr::FnExpr { params, body } => self.eval_fn(params.clone(), body.clone()),
            Expr::AsyncFnExpr { params, body } => {
                Object::AsyncFunction(params.clone(), body.clone(), Arc::clone(&self.context.env))
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
                    new_vec.push(Self::eval_expr_sync(self, env, e));
                }
                Object::Array(new_vec)
            }
            Expr::HashExpr(hash_exprs) => {
                use std::hash::BuildHasherDefault;
                use ahash::{AHasher, HashMapExt};
                type HashMap<K, V> = std::collections::HashMap<K, V, BuildHasherDefault<AHasher>>;
                let mut hashmap = HashMap::new();
                
                for (key_expr, val_expr) in hash_exprs {
                    let key = Self::eval_expr_sync(self, env, key_expr);
                    let val = Self::eval_expr_sync(self, env, val_expr);
                    
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
                let target = Self::eval_expr_sync(self, env, array);
                let idx = Self::eval_expr_sync(self, env, index);
                match target {
                    Object::Array(arr) => match obj_to_int(idx) {
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
                        let name = obj_to_hash(idx);
                        match name {
                            Object::Error(_) => name,
                            _ => hash.remove(&name).unwrap_or(Object::Null),
                        }
                    }
                    o => Object::Error(RuntimeError::NotHashable(o.type_name())),
                }
            }
            Expr::ThisExpr => self.eval_this(),

            /* Safety nets for expressions that aren't expected in sync 
            contexts like method calls and for-loop init clauses */

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

    pub fn eval_blockstmt_sync(&self, env: &mut Environment, prog: &Program) -> Object {
        let mut result = Object::Null;

        for stmt in prog.iter() {
            result = Self::eval_statement_sync(self, env, stmt);
            match result {
                Object::ReturnValue(_) | Object::Break | Object::Continue | Object::Error(_) | Object::ThrownValue(_) => {
                    return result;
                }
                _ => {}
            }
        }
        result
    }

    pub fn eval_statement_sync(&self, env: &mut Environment, stmt: &Stmt) -> Object {
        match stmt {
            Stmt::ExprStmt(expr) => Self::eval_expr_sync(self, env, expr),
            Stmt::ExprValueStmt(expr) => Self::eval_expr_sync(self, env, expr),
            Stmt::ReturnStmt(expr) => Object::ReturnValue(Box::new(Self::eval_expr_sync(self, env, expr))),
            Stmt::LetStmt(ident, expr) => {
                let object = Self::eval_expr_sync(self, env, expr);
                register_ident_sync(env, ident.clone(), object)
            }
            Stmt::MultiLetStmt { idents, values } => {
                let mut result = Object::Null;
                for (id, expr) in idents.iter().zip(values.iter()) {
                    let object = Self::eval_expr_sync(self, env, expr);
                    result = register_ident_sync(env, id.clone(), object);
                }
                result
            }
            Stmt::FnStmt { name, params, body } => {
                let mut params = params.clone();
                for (i, param) in params.iter_mut().enumerate() {
                    if param.slot.is_unset() {
                        param.slot = crate::ast::ast::SlotIndex(i as u16);
                    }
                }
                let fn_obj = Object::Function(params, body.clone(), Arc::clone(&self.context.env));
                register_ident_sync(env, name.clone(), fn_obj)
            }
            Stmt::AssignStmt(ident, expr) => {
                let name = ident.name.clone();
                if env.get_by_name(&name).is_none() {
                    return Object::Error(RuntimeError::UndefinedVariable(name));
                }
                let object = Self::eval_expr_sync(self, env, expr);
                register_ident_sync(env, ident.clone(), object)
            }
            _ => Object::Error(RuntimeError::InvalidOperation(
                "statement not allowed in sync context".to_string()
            )),
        }
    }

}