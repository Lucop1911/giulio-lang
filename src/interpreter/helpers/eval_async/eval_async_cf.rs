use std::sync::{Arc, Mutex};
use crate::{
    ast::ast::{Expr, Ident, Program, Stmt},
    errors::RuntimeError,
    interpreter::{
        env::Environment, obj::Object, helpers::type_converters::obj_to_bool
    },
};

use super::super::super::eval::Evaluator;

impl Evaluator {
    pub fn async_eval_if(&mut self, cond: Expr, conse: Program, maybe_alter: Option<Program>) -> impl Future<Output = Object> + Send + '_ {
        let self_clone = self.clone();
        async move {
            let object = self_clone.eval_expr(cond).await;
            match obj_to_bool(object) {
                Ok(b) => {
                    if b {
                        self_clone.eval_blockstmt(&conse).await
                    } else {
                        match maybe_alter {
                            Some(else_conse) => self_clone.eval_blockstmt(&else_conse).await,
                            _ => Object::Null,
                        }
                    }
                }
                Err(err) => err,
            }
        }
    }

    pub fn async_eval_while(&mut self, cond: Box<Expr>, body: Program) -> impl Future<Output = Object> + Send + '_ {
        let self_clone = self.clone();
        async move {
            loop {
                let cond_result = self_clone.eval_expr((*cond).clone()).await;
                match obj_to_bool(cond_result) {
                    Ok(true) => {
                        let result = self_clone.eval_blockstmt(&body).await;
                        match result {
                            Object::Break => return Object::Null,
                            Object::Continue => continue,
                            Object::ReturnValue(_) => return result,
                            Object::Error(_) => return result,
                            _ => {}
                        }
                    }
                    Ok(false) => return Object::Null,
                    Err(e) => return e,
                }
            }
        }
    }

    pub fn async_eval_for(&mut self, ident: Vec<Ident>, iterable: Box<Expr>, body: Program) -> impl Future<Output = Object> + Send + '_  {
        let self_clone = self.clone();
        async move {
            let iter_obj = self_clone.eval_expr(*iterable).await;

            let items = match iter_obj {
                Object::Array(arr) => arr,
                Object::String(s) => {
                    s.chars().map(|c| Object::String(c.to_string())).collect()
                }
                _ => {
                    return Object::Error(RuntimeError::InvalidOperation(format!("cannot iterate over {}", iter_obj.type_name())))
                }
            };

            for item in items {
                let Object::Array(values) = item else {
                    return Object::Error(RuntimeError::TypeMismatch {
                        expected: "array".to_string(),
                        got: item.type_name(),
                    });
                };
                
                if values.len() != ident.len() {
                    return Object::Error(RuntimeError::InvalidOperation(
                        format!("destructuring mismatch: {} variables, {} values", ident.len(), values.len())
                    ));
                }
                
                for (id, value) in ident.iter().zip(values.into_iter()) {
                    self_clone.env.lock().unwrap().set_by_name(&id.name, value);
                }

                let result = self_clone.eval_blockstmt(&body).await;
                match result {
                    Object::Break => return Object::Null,
                    Object::Continue => continue,
                    Object::ReturnValue(_) => return result,
                    Object::Error(_) => return  result,
                    _ => {}
                }
            }
            Object::Null
        }
    }

    pub fn async_eval_c_style_for(
        &mut self,
        init: Option<Box<Stmt>>,
        cond: Option<Box<Expr>>,
        update: Option<Box<Stmt>>,
        body: Program,
    ) -> impl Future<Output = Object> + Send + '_  {
        let self_clone = self.clone();
        
        async move {
            let result = self_clone.eval_c_style_for_sync(init, cond, update, body);
            result
        }
    }

    pub fn eval_c_style_for_sync(
        &self,
        init: Option<Box<Stmt>>,
        cond: Option<Box<Expr>>,
        update: Option<Box<Stmt>>,
        body: Program,
    ) -> Object {
        let mut env_guard = self.env.lock().unwrap();
        
        if let Some(init_stmt) = init {
            let result = self.eval_statement_sync(&mut env_guard, &init_stmt);
            if let Object::Error(_) = result {
                return result;
            }
        }
        
        loop {
            let should_continue = if let Some(ref cond_expr) = cond {
                match self.eval_expr_sync(&mut env_guard, cond_expr) {
                    Object::Boolean(b) => b,
                    Object::Error(e) => return Object::Error(e),
                    _ => return Object::Error(RuntimeError::TypeMismatch {
                        expected: "boolean".to_string(),
                        got: "non-boolean".to_string(),
                    }),
                }
            } else {
                true
            };
            
            if !should_continue {
                break;
            }
            
            if !body.is_empty() {
                let result = self.eval_blockstmt_sync(&mut env_guard, &body);
                match result {
                    Object::Break => return Object::Null,
                    Object::Continue => {},
                    Object::ReturnValue(_) => return result,
                    Object::Error(_) => return result,
                    _ => {}
                }
            }
            
            if let Some(ref update_stmt) = update {
                let result = self.eval_statement_sync(&mut env_guard, update_stmt);
                if let Object::Error(_) = result {
                    return result;
                }
            }
        }
        
        Object::Null
    }

    pub fn async_eval_try_catch_expr(
        &mut self, 
        try_body: Program, 
        catch_ident: Option<Ident>, 
        catch_body: Option<Program>, 
        finally_body: Option<Program>
    ) -> impl Future<Output = Object> + Send + '_  {
        let mut self_clone = self.clone();
        async move {
            let mut try_result = self_clone.eval_blockstmt(&try_body).await;
            let final_result: Object;
        
            let caught_exception_obj = match try_result.clone() {
                Object::ThrownValue(ex) => Some(*ex),
                Object::Error(err) => Some(Object::Error(err)),
                _ => None,
            };

            if let Some(exception) = caught_exception_obj {
                if let Some(Ident { name: e_name, .. }) = catch_ident {
                    if let Some(c_body) = catch_body {
                        let old_env = Arc::clone(&self_clone.env);
                        let old_context_env = Arc::clone(&self_clone.context.env);
                        // Allocate slots for any let-bindings in the catch body.
                        // The catch variable itself is stored by name.
                        let num_slots = Environment::count_slots(&[], &c_body);
                        let mut new_env = Environment::new_function_env(Arc::clone(&self_clone.env), num_slots);
                        new_env.set_by_name(&e_name, exception);
                        let new_env_arc = Arc::new(Mutex::new(new_env));
                        self_clone.env = Arc::clone(&new_env_arc);
                        self_clone.context.env = Arc::clone(&new_env_arc);

                        try_result = self_clone.eval_blockstmt(&c_body).await;
                        self_clone.env = old_env;
                        self_clone.context.env = old_context_env;
                    }
                }
            }
        
            if let Some(f_body) = finally_body {
                let finally_result = self_clone.eval_blockstmt(&f_body).await;
                match finally_result {
                    Object::ReturnValue(_) | Object::Break | Object::Continue | Object::ThrownValue(_) | Object::Error(_) => {
                        final_result = finally_result;
                    },
                    _ => {
                        final_result = try_result;
                    }
                }
            } else {
                final_result = try_result;
            }

            if let Object::ThrownValue(_) = final_result {
                return final_result;
            }
        
            match final_result {
                Object::ReturnValue(_) | Object::Break | Object::Continue | Object::Error(_) => final_result,
                _ => self_clone.returned(final_result), 
            }
        }
    }
}