use std::rc::Rc;
use std::cell::RefCell;
use std::collections::HashMap;

use crate::ast::ast::{Expr, Ident, ImportItems, Program, Stmt};
use crate::errors::RuntimeError;
use crate::interpreter::eval::Evaluator;
use crate::interpreter::obj::{BuiltinFunction, Object, StdFunction};
use crate::interpreter::env::Environment;
use crate::interpreter::builtins::methods::BuiltinMethods;

pub trait EvalHelper {
    fn eval_struct_def(&mut self, name: Ident, fields: Vec<(Ident, Expr)>, methods: Vec<(Ident, Expr)>) -> Object;
    fn eval_struct_literal(&mut self, name: Ident, field_assignments: Vec<(Ident, Expr)>) -> Object;
    fn eval_field_access(&mut self, object_expr: Expr, field_name: String) -> Object;
    fn eval_field_assign(&mut self, object_expr: Expr, field_name: String, value_expr: Expr) -> Object;
    fn eval_index_assign(&mut self, target_expr: Expr, index_expr: Expr, value_expr: Expr) -> Object;
    fn eval_method_call(&mut self, object_expr: Expr, method_name: String, args_expr: Vec<Expr>) -> Object;
    fn eval_fn_call_direct(&mut self, args: Vec<Object>, params: Vec<Ident>, body: Program) -> Object;
    fn eval_import(&mut self, path: Vec<String>, items: ImportItems) -> Object;
    fn eval_while(&mut self, cond: Box<Expr>, body: Program) -> Object;
    fn eval_for(&mut self, ident: Ident, iterable: Box<Expr>, body: Program) -> Object;
    fn eval_c_style_for(&mut self, init: Option<Box<Stmt>>, cond: Option<Box<Expr>>, update: Option<Box<Stmt>>, body: Program) -> Object;
    fn eval_array(&mut self, exprs: Vec<Expr>) -> Object;
    fn eval_hash(&mut self, hs: Vec<(Expr, Expr)>) -> Object;
    fn eval_index(&mut self, target_exp: Expr, id_exp: Expr) -> Object;
    fn eval_builtin_call(&mut self, args_expr: Vec<Expr>, min_params: usize, max_params: usize, b_fn: BuiltinFunction) -> Object;
    fn eval_std_call(&mut self, args_expr: Vec<Expr>, min_params: usize, max_params: usize, s_fn: StdFunction) -> Object;
}

impl EvalHelper for Evaluator {
    fn eval_struct_def(&mut self, name: Ident, fields: Vec<(Ident, Expr)>, methods: Vec<(Ident, Expr)>) -> Object {
        let Ident(struct_name) = name.clone();
        
        let mut default_fields = HashMap::new();
        for (Ident(field_name), expr) in fields {
            let value = self.eval_expr(expr);
            default_fields.insert(field_name, value);
        }
        
        let mut struct_methods = HashMap::new();
        for (Ident(method_name), expr) in methods {
            struct_methods.insert(method_name, expr);
        }

        let struct_obj = Object::Struct {
            name: struct_name.clone(),
            fields: default_fields,
            methods: struct_methods.into_iter().map(|(k, expr)| (k, self.eval_expr(expr))).collect(),
        };
        
        self.env.borrow_mut().set(&struct_name, struct_obj.clone());
        
        Object::Null
    }

    fn eval_struct_literal(&mut self, name: Ident, field_assignments: Vec<(Ident, Expr)>) -> Object {
        let Ident(struct_name) = name;
        
        let struct_def = match self.env.borrow().get(&struct_name) {
            Some(Object::Struct { fields, methods, .. }) => (fields, methods),
            Some(_) => return Object::Error(RuntimeError::InvalidOperation(
                format!("{} is not a struct", struct_name)
            )),
            None => return Object::Error(RuntimeError::UndefinedVariable(struct_name)),
        };
        
        let (default_fields, methods) = struct_def;
        
        let mut instance_fields = default_fields.clone();
        
        // Override with provided field assignments
        for (Ident(field_name), expr) in field_assignments {
            let value = self.eval_expr(expr);
            instance_fields.insert(field_name, value);
        }
        
        Object::Struct {
            name: struct_name,
            fields: instance_fields,
            methods,
        }
    }

    fn eval_field_access(&mut self, object_expr: Expr, field_name: String) -> Object {
        let object = self.eval_expr(object_expr);
        
        match object {
            Object::Struct { fields, .. } => {
                match fields.get(&field_name) {
                    Some(value) => value.clone(),
                    None => Object::Error(RuntimeError::InvalidOperation(
                        format!("struct has no field '{}'", field_name)
                    )),
                }
            }
            other => Object::Error(RuntimeError::InvalidOperation(
                format!("{} does not have fields", other.type_name())
            )),
        }
    }

    fn eval_field_assign(&mut self, object_expr: Expr, field_name: String, value_expr: Expr) -> Object {
        // Evaluate the value first
        let value = self.eval_expr(value_expr);
        
        // Special case: if the object is 'this', we need to update it in the environment
        if let Expr::ThisExpr = object_expr {
            let current_this = self.env.borrow().get("this");
            match current_this {
                Some(Object::Struct { name, mut fields, methods }) => {
                    fields.insert(field_name, value.clone());
                    let updated_struct = Object::Struct { name, fields, methods };
                    self.env.borrow_mut().set("this", updated_struct);
                    return value;
                }
                Some(other) => {
                    return Object::Error(RuntimeError::InvalidOperation(
                        format!("{} does not have fields", other.type_name())
                    ));
                }
                None => {
                    return Object::Error(RuntimeError::InvalidOperation(
                        "'this' is not defined in current scope".to_string()
                    ));
                }
            }
        }
        
        // For now, we only support 'this.field = value'
        Object::Error(RuntimeError::InvalidOperation(
            "Can only assign to 'this.field', not other object fields".to_string()
        ))
    }

    fn eval_index_assign(&mut self, target_expr: Expr, index_expr: Expr, value_expr: Expr) -> Object {
        // Evaluate the index and value
        let index = self.eval_expr(index_expr);
        let value = self.eval_expr(value_expr);
        
        // If it's an identifier or "this", we can update it in the environment        
        match target_expr {
            Expr::IdentExpr(Ident(ref var_name)) => {
                // Get the current value from the environment
                let current_value = match self.env.borrow().get(var_name) {
                    Some(val) => val,
                    None => return Object::Error(RuntimeError::UndefinedVariable(var_name.clone())),
                };
                
                // Update the appropriate data structure
                let updated_value = match current_value {
                    Object::Array(mut arr) => {
                        // Handle array index assignment
                        match self.obj_to_int(index.clone()) {
                            Ok(idx_num) => {
                                if idx_num < 0 {
                                    return Object::Error(RuntimeError::IndexOutOfBounds {
                                        index: idx_num,
                                        length: arr.len(),
                                    });
                                }
                                let idx = idx_num as usize;
                                if idx >= arr.len() {
                                    return Object::Error(RuntimeError::IndexOutOfBounds {
                                        index: idx_num,
                                        length: arr.len(),
                                    });
                                }
                                arr[idx] = value.clone();
                                Object::Array(arr)
                            }
                            Err(err) => return err,
                        }
                    }
                    Object::Hash(mut hash) => {
                        // Handle hash key assignment
                        let key = self.obj_to_hash(index.clone());
                        if let Object::Error(_) = key {
                            return key;
                        }
                        hash.insert(key, value.clone());
                        Object::Hash(hash)
                    }
                    other => {
                        return Object::Error(RuntimeError::InvalidOperation(
                            format!("Cannot index into {}", other.type_name())
                        ));
                    }
                };
                
                // Update the variable in the environment
                self.env.borrow_mut().set(var_name, updated_value);
                value
            }
            Expr::ThisExpr => {
                // Handle this[index] = value
                let current_this = self.env.borrow().get("this");
                match current_this {
                    Some(Object::Array(mut arr)) => {
                        match self.obj_to_int(index.clone()) {
                            Ok(idx_num) => {
                                if idx_num < 0 {
                                    return Object::Error(RuntimeError::IndexOutOfBounds {
                                        index: idx_num,
                                        length: arr.len(),
                                    });
                                }
                                let idx = idx_num as usize;
                                if idx >= arr.len() {
                                    return Object::Error(RuntimeError::IndexOutOfBounds {
                                        index: idx_num,
                                        length: arr.len(),
                                    });
                                }
                                arr[idx] = value.clone();
                                self.env.borrow_mut().set("this", Object::Array(arr));
                                value
                            }
                            Err(err) => err,
                        }
                    }
                    Some(Object::Hash(mut hash)) => {
                        let key = self.obj_to_hash(index.clone());
                        if let Object::Error(_) = key {
                            return key;
                        }
                        hash.insert(key, value.clone());
                        self.env.borrow_mut().set("this", Object::Hash(hash));
                        value
                    }
                    Some(other) => {
                        Object::Error(RuntimeError::InvalidOperation(
                            format!("Cannot index into {}", other.type_name())
                        ))
                    }
                    None => {
                        Object::Error(RuntimeError::InvalidOperation(
                            "'this' is not defined in current scope".to_string()
                        ))
                    }
                }
            }
            _ => {
                Object::Error(RuntimeError::InvalidOperation(
                    "Can only assign to variable[index] or this[index], not complex expressions".to_string()
                ))
            }
        }
    }

    fn eval_method_call(&mut self, object_expr: Expr, method_name: String, args_expr: Vec<Expr>) -> Object {
        // If the method is called on an identifier, we'll need its name to update it later.
        let var_name_if_ident = if let Expr::IdentExpr(Ident(ref name)) = object_expr {
            Some(name.clone())
        } else {
            None
        };
        
        let object = self.eval_expr(object_expr);

        if let Object::Struct { ref methods, .. } = object {
            if let Some(method_obj) = methods.get(&method_name) {
                let old_env = Rc::clone(&self.env);
                let mut new_env = Environment::new_with_outer(Rc::clone(&self.env));
                new_env.set("this", object.clone());
                
                self.env = Rc::new(RefCell::new(new_env));
                
                let result = match method_obj {
                    Object::Function(params, body, _) => {
                        let args = args_expr.into_iter().map(|e| self.eval_expr(e)).collect();
                        self.eval_fn_call_direct(args, params.clone(), body.clone())
                    }
                    _ => {
                        self.env = old_env;
                        return Object::Error(RuntimeError::NotCallable(method_name));
                    }
                };
                
                // After the call, 'this' might have been modified.
                let modified_this = self.env.borrow().get("this").unwrap_or(object.clone());
                
                self.env = old_env;
                
                // Update env with modified struct if needed
                if let Some(var_name) = var_name_if_ident {
                    self.env.borrow_mut().set(&var_name, modified_this.clone());
                }

                let final_result = match result {
                    Object::Null => modified_this,  // No explicit return, give back modified struct
                    Object::ReturnValue(val) => *val,  // Explicit return
                    other => other,  // Error or other value
                };
                
                return self.returned(final_result);
            }
        }
        
        // Fall back to builtin methods
        let args = args_expr.into_iter().map(|e| self.eval_expr(e)).collect();
        match BuiltinMethods::call_method(object, &method_name, args) {
            Ok(obj) => obj,
            Err(e) => Object::Error(e),
        }
    }

    fn eval_fn_call_direct(
        &mut self,
        args: Vec<Object>,
        params: Vec<Ident>,
        body: Program,
    ) -> Object {
        if args.len() != params.len() {
            return Object::Error(RuntimeError::WrongNumberOfArguments {
                min: params.len(),
                max: params.len(),
                got: args.len(),
            });
        }

        let zipped = params.into_iter().zip(args);
        for (Ident(name), o) in zipped {
            self.env.borrow_mut().set(&name, o);
        }
        
        self.eval_blockstmt(body)
    }

    fn eval_import(&mut self, path: Vec<String>, items: ImportItems) -> Object {
        // Load the module
        let module = match self.module_registry.borrow_mut().load_module(&path) {
            Ok(m) => m,
            Err(e) => return Object::Error(e),
        };
        
        // Import items into current environment
        match items {
            ImportItems::All => {
                // Import all exports
                for (name, obj) in module.exports {
                    self.env.borrow_mut().set(&name, obj);
                }
            }
            ImportItems::Specific(names) => {
                // Import specific items
                for name in names {
                    if let Some(obj) = module.exports.get(&name) {
                        self.env.borrow_mut().set(&name, obj.clone());
                    } else {
                        return Object::Error(RuntimeError::InvalidOperation(
                            format!("Module {} has no export '{}'", module.name, name)
                        ));
                    }
                }
            }
            ImportItems::Single(name) => {
                // Import single item
                if let Some(obj) = module.exports.get(&name) {
                    self.env.borrow_mut().set(&name, obj.clone());
                } else {
                    return Object::Error(RuntimeError::InvalidOperation(
                        format!("Module {} has no export '{}'", module.name, name)
                    ));
                }
            }
        }
        
        Object::Null
    }

    fn eval_while(&mut self, cond: Box<Expr>, body: Program) -> Object {
        loop {
            let cond_result = self.eval_expr(*cond.clone());
            match self.obj_to_bool(cond_result) {
                Ok(true) => {
                    let result = self.eval_blockstmt(body.clone());
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

    fn eval_for(&mut self, ident: Ident, iterable: Box<Expr>, body: Program) -> Object {
        let iter_obj = self.eval_expr(*iterable);

        let items = match iter_obj {
            Object::Array(arr) => arr,
            Object::String(s) => {
                s.chars().map(|c| Object::String(c.to_string())).collect()
            }
            _ => {
                return Object::Error(RuntimeError::InvalidOperation(format!("cannot iterate over {}", iter_obj.type_name())))
            }
        };

        let Ident(var_name) = ident;

        for item in items {
            self.env.borrow_mut().set(&var_name, item);

            let result = self.eval_blockstmt(body.clone());
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

    fn eval_c_style_for(
        &mut self,
        init: Option<Box<Stmt>>,
        cond: Option<Box<Expr>>,
        update: Option<Box<Stmt>>,
        body: Program,
    ) -> Object {
        // Execute initialization statement if present
        if let Some(init_stmt) = init {
            let result = self.eval_statement(*init_stmt);
            if let Object::Error(_) = result {
                return result;
            }
        }
        
        loop {
            // Check condition (if no condition, loop forever like C)
            let should_continue = if let Some(ref cond_expr) = cond {
                match self.eval_expr(cond_expr.as_ref().clone()) {
                    Object::Boolean(b) => b,
                    Object::Error(e) => return Object::Error(e),
                    _ => return Object::Error(RuntimeError::TypeMismatch {
                        expected: "boolean".to_string(),
                        got: "non-boolean".to_string(),
                    }),
                }
            } else {
                true // No condition means infinite loop
            };
            
            if !should_continue {
                break;
            }
            
            // Execute body
            let result = self.eval_blockstmt(body.clone());
            match result {
                Object::Break => return Object::Null,
                Object::Continue => {},
                Object::ReturnValue(_) => return result,
                Object::Error(_) => return result,
                _ => {}
            }
            
            // Execute update statement if present
            if let Some(ref update_stmt) = update {
                let result = self.eval_statement(update_stmt.as_ref().clone());
                if let Object::Error(_) = result {
                    return result;
                }
            }
        }
        
        Object::Null
    }

    fn eval_array(&mut self, exprs: Vec<Expr>) -> Object {
        let new_vec = exprs.into_iter().map(|e| self.eval_expr(e)).collect();
        Object::Array(new_vec)
    }

    fn eval_hash(&mut self, hs: Vec<(Expr, Expr)>) -> Object {
        let mut hashmap = HashMap::new();
        
        for (key_expr, val_expr) in hs {
            let key = self.eval_expr(key_expr);
            let val = self.eval_expr(val_expr);
            
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

    fn eval_index(&mut self, target_exp: Expr, id_exp: Expr) -> Object {
        let target = self.eval_expr(target_exp);
        let index = self.eval_expr(id_exp);
        match target {
            Object::Array(arr) => match self.obj_to_int(index) {
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
                let name = self.obj_to_hash(index);
                match name {
                    Object::Error(_) => name,
                    _ => hash.remove(&name).unwrap_or(Object::Null),
                }
            }
            o => Object::Error(RuntimeError::NotHashable(o.type_name())),
        }
    }

    fn eval_builtin_call(
        &mut self,
        args_expr: Vec<Expr>,
        min_params: usize,
        max_params: usize,
        b_fn: BuiltinFunction,
    ) -> Object {
        if args_expr.len() < min_params || args_expr.len() > max_params {
            return Object::Error(RuntimeError::WrongNumberOfArguments {
                min: min_params,
                max: max_params,
                got: args_expr.len(),
            });
        }

        let args = args_expr
            .into_iter()
            .map(|e| self.eval_expr(e))
            .collect::<Vec<_>>();
        
        match b_fn(args) {
            Ok(obj) => obj,
            Err(e) => Object::Error(RuntimeError::InvalidArguments(e)),
        }
    }

    fn eval_std_call(
        &mut self,
        args_expr: Vec<Expr>,
        min_params: usize,
        max_params: usize,
        s_fn: StdFunction,
    ) -> Object {
        if args_expr.len() < min_params || args_expr.len() > max_params {
            return Object::Error(RuntimeError::WrongNumberOfArguments {
                min: min_params,
                max: max_params,
                got: args_expr.len(),
            });
        }

        let args = args_expr
            .into_iter()
            .map(|e| self.eval_expr(e))
            .collect::<Vec<_>>();
        
        match s_fn(args) {
            Ok(obj) => obj,
            Err(e) => Object::Error(e),
        }
    }
}