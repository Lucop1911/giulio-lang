use std::{cell::RefCell, collections::HashMap, rc::Rc};

use num_bigint::BigInt;

use crate::{
    ast::ast::{Expr, Ident, ImportItems, Infix, Literal, Prefix, Program, Stmt},
    errors::RuntimeError,
    interpreter::{
        builtins::methods::BuiltinMethods, env::Environment, module_registry::ModuleRegistry, obj::{BuiltinFunction, Object}
    },
};

pub struct Evaluator {
    pub(crate) env: Rc<RefCell<Environment>>,
    module_registry: Rc<RefCell<ModuleRegistry>>,
}

impl Default for Evaluator {
    fn default() -> Self {
        let base_path = std::env::current_dir()
            .expect("failed to get current directory");

        let registry = Rc::new(RefCell::new(
            ModuleRegistry::new(base_path),
        ));

        Evaluator {
            env: Rc::new(RefCell::new(Environment::new())),
            module_registry: registry,
        }
    }
}

impl Evaluator {
    pub fn new(module_registry: Rc<RefCell<ModuleRegistry>>) -> Self {
        Evaluator {
            env: Rc::new(RefCell::new(Environment::new())),
            module_registry,
        }
    }

    fn returned(&mut self, object: Object) -> Object {
        match object {
            Object::ReturnValue(v) => *v,
            o => o,
        }
    }

    pub fn eval_program(&mut self, prog: Program) -> Object {
        let return_data = self.eval_blockstmt(prog);
        self.returned(return_data)
    }

    pub fn eval_blockstmt(&mut self, mut prog: Program) -> Object {
        match prog.len() {
            0 => Object::Null,
            1 => self.eval_statement(prog.remove(0)),
            _ => {
                let s = prog.remove(0);
                let object = self.eval_statement(s);
                match object {
                    Object::ReturnValue(_) | Object::Break | Object::Continue | Object::Error(_) => object,
                    _ => self.eval_blockstmt(prog)
                }
            }
        }
    }

    pub fn eval_statement(&mut self, stmt: Stmt) -> Object {
        match stmt {
            Stmt::ExprStmt(expr) => self.eval_expr(expr),
            Stmt::ReturnStmt(expr) => Object::ReturnValue(Box::new(self.eval_expr(expr))),
            Stmt::LetStmt(ident, expr) => {
                let object = self.eval_expr(expr);
                self.register_ident(ident, object)
            }
            Stmt::AssignStmt(ident, expr) => {
                // Check if variable exists
                let Ident(ref name) = ident;
                if self.env.borrow().get(name).is_none() {
                    return Object::Error(RuntimeError::UndefinedVariable(name.clone()));
                }
                // Reassign the variable
                let object = self.eval_expr(expr);
                self.register_ident(ident, object)
            }
            Stmt::StructStmt { name, fields, methods } => {
                self.eval_struct_def(name, fields, methods)
            }
            Stmt::ImportStmt { path, items } => {
                self.eval_import(path, items)
            }
            Stmt::BreakStmt => Object::Break,
            Stmt::ContinueStmt => Object::Continue,
        }
    }

    pub fn register_ident(&mut self, ident: Ident, object: Object) -> Object {
        let Ident(name) = ident;
        self.env.borrow_mut().set(&name, object.clone());
        object
    }

    pub fn eval_expr(&mut self, expr: Expr) -> Object {
        match expr {
            Expr::IdentExpr(i) => self.eval_ident(i),
            Expr::LitExpr(l) => self.eval_literal(l),
            Expr::PrefixExpr(prefix, expr) => self.eval_prefix(&prefix, *expr),
            Expr::InfixExpr(infix, expr1, expr2) => self.eval_infix(&infix, *expr1, *expr2),
            Expr::IfExpr {
                cond,
                consequence,
                alternative,
            } => self.eval_if(*cond, consequence, alternative),
            Expr::FnExpr { params, body } => self.eval_fn(params, body),
            Expr::CallExpr {
                function: fn_exp,
                arguments,
            } => self.eval_call(*fn_exp, arguments),
            Expr::ArrayExpr(exprs) => self.eval_array(exprs),
            Expr::HashExpr(hash_exprs) => self.eval_hash(hash_exprs),
            Expr::IndexExpr { array, index } => self.eval_index(*array, *index),
            Expr::MethodCallExpr { object, method, arguments } => {
                self.eval_method_call(*object, method, arguments)
            }
            Expr::StructLiteral { name, fields } => {
                self.eval_struct_literal(name, fields)
            }
            Expr::ThisExpr => {
                self.eval_this()
            }
            Expr::FieldAccessExpr { object, field } => {
                self.eval_field_access(*object, field)
            }
            Expr::WhileExpr { cond, body } => self.eval_while(cond, body),
            Expr::ForExpr { ident, iterable, body } => self.eval_for(ident, iterable, body),
        }
    }

    pub fn eval_this(&mut self) -> Object {
        match self.env.borrow().get("this") {
            Some(obj) => obj,
            None => Object::Error(RuntimeError::InvalidOperation(
                "'this' can only be used inside a method".to_string()
            )),
        }
    }

    pub fn eval_ident(&mut self, ident: Ident) -> Object {
        let Ident(name) = ident;
        let borrow_env = self.env.borrow();
        let var = borrow_env.get(&name);
        match var {
            Some(o) => o,
            None => Object::Error(RuntimeError::UndefinedVariable(name)),
        }
    }

    pub fn eval_literal(&mut self, literal: Literal) -> Object {
        match literal {
            Literal::IntLiteral(i) => Object::Integer(i),
            Literal::BigIntLiteral(big) => Object::BigInteger(big),
            Literal::FloatLitera(f) => Object::Float(f),
            Literal::BoolLiteral(b) => Object::Boolean(b),
            Literal::StringLiteral(s) => Object::String(s),
            Literal::NullLiteral => Object::Null,
        }
    }

    pub fn eval_prefix(&mut self, prefix: &Prefix, expr: Expr) -> Object {
        let object = self.eval_expr(expr);
        match *prefix {
            Prefix::Not => match self.obj_to_bool(object) {
                Ok(b) => Object::Boolean(!b),
                Err(err) => err,
            },
            Prefix::PrefixPlus => {
                match object {
                    Object::Integer(_) | Object::BigInteger(_) | Object::Float(_) => object,
                    Object::Error(e) => Object::Error(e),
                    o => Object::Error(RuntimeError::TypeMismatch {
                        expected: "integer".to_string(),
                        got: o.type_name(),
                    })
                }
            },
            Prefix::PrefixMinus => {
                match object {
                    Object::Integer(i) => {
                        match i.checked_neg() {
                            Some(result) => Object::Integer(result),
                            None => Object::BigInteger(-BigInt::from(i))
                        }
                    }
                    Object::BigInteger(big) => self.normalize_int(-big),
                    Object::Float(f) => Object::Float(-f),
                    Object::Error(e) => Object::Error(e),
                    o => Object::Error(RuntimeError::TypeMismatch {
                        expected: "integer".to_string(),
                        got: o.type_name(),
                    })
                }
            },
        }
    }

    pub fn eval_infix(&mut self, infix: &Infix, expr1: Expr, expr2: Expr) -> Object {
        let object1 = self.eval_expr(expr1);
        let object2 = self.eval_expr(expr2);
        
        match *infix {
            Infix::Plus => self.object_add(object1, object2),
            Infix::Minus => self.object_subtract(object1, object2),
            Infix::Divide => self.object_divide(object1, object2),
            Infix::Multiply => self.object_multiply(object1, object2),
            Infix::Modulo => self.object_modulo(object1, object2),
            Infix::Equal => Object::Boolean(object1 == object2),
            Infix::NotEqual => Object::Boolean(object1 != object2),
            Infix::GreaterThanEqual => self.object_compare_gte(object1, object2),
            Infix::GreaterThan => self.object_compare_gt(object1, object2),
            Infix::LessThanEqual => self.object_compare_lte(object1, object2),
            Infix::LessThan => self.object_compare_lt(object1, object2),
            Infix::And => {
                let b1 = self.obj_to_bool(object1);
                let b2 = self.obj_to_bool(object2);
                match (b1, b2) {
                    (Ok(b1), Ok(b2)) => Object::Boolean(b1 && b2),
                    (Err(err), _) | (_, Err(err)) => err,
                }
            }
            Infix::Or => {
                let b1 = self.obj_to_bool(object1);
                let b2 = self.obj_to_bool(object2);
                match (b1, b2) {
                    (Ok(b1), Ok(b2)) => Object::Boolean(b1 || b2),
                    (Err(err), _) | (_, Err(err)) => err,
                }
            }
        }
    }
    
    pub fn eval_if(&mut self, cond: Expr, conse: Program, maybe_alter: Option<Program>) -> Object {
        let object = self.eval_expr(cond);
        match self.obj_to_bool(object) {
            Ok(b) => {
                if b {
                    self.eval_blockstmt(conse)
                } else {
                    match maybe_alter {
                        Some(else_conse) => self.eval_blockstmt(else_conse),
                        _ => Object::Null,
                    }
                }
            }
            Err(err) => err,
        }
    }

    pub fn eval_fn(&mut self, params: Vec<Ident>, body: Program) -> Object {
        Object::Function(params, body, Rc::clone(&self.env))
    }

    pub fn eval_method(&mut self, params: Vec<Ident>, body: Program) -> Object {
        Object::Method(params, body, Rc::clone(&self.env))
    }

    pub fn eval_call(&mut self, fn_expr: Expr, args_expr: Vec<Expr>) -> Object {
        let fn_object = self.eval_expr(fn_expr);
        let fn_ = self.obj_to_func(fn_object);
        match fn_ {
            Object::Function(params, body, f_env) => {
                self.eval_fn_call(args_expr, params, body, &f_env)
            }
            Object::Builtin(_, min_params, max_params, b_fn) => {
                self.eval_builtin_call(args_expr, min_params, max_params, b_fn)
            }
            o_err => o_err,
        }
    }

    fn eval_fn_call(
        &mut self,
        args_expr: Vec<Expr>,
        params: Vec<Ident>,
        body: Program,
        f_env: &Rc<RefCell<Environment>>,
    ) -> Object {
        if args_expr.len() < params.len() {
            return Object::Error(RuntimeError::WrongNumberOfArguments {
                min: params.len(),
                max: params.len(),
                got: args_expr.len(),
            });
        }

        let args = args_expr
            .into_iter()
            .map(|e| self.eval_expr(e))
            .collect::<Vec<_>>();
        let old_env = Rc::clone(&self.env);
        let mut new_env = Environment::new_with_outer(Rc::clone(f_env));
        let zipped = params.into_iter().zip(args);
        for (Ident(name), o) in zipped {
            new_env.set(&name, o);
        }
        self.env = Rc::new(RefCell::new(new_env));
        let object = self.eval_blockstmt(body);
        self.env = old_env;
        self.returned(object)
    }

    pub fn eval_field_access(&mut self, object_expr: Expr, field_name: String) -> Object {
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

    pub fn eval_method_call(&mut self, object_expr: Expr, method_name: String, args_expr: Vec<Expr>) -> Object {
        let object = self.eval_expr(object_expr);
        
        if let Object::Struct { ref methods, .. } = object
            && let Some(method_obj) = methods.get(&method_name) {
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
                
                let modified_this = self.env.borrow().get("this").unwrap_or(object.clone());
                
                self.env = old_env;
                
                let final_result = match result {
                    Object::Null => modified_this,  // No explicit return, give back modified struct
                    Object::ReturnValue(val) => *val,  // Explicit return
                    other => other,  // Error or other value
                };
                
                return self.returned(final_result);
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

    pub fn eval_struct_def(&mut self, name: Ident, fields: Vec<(Ident, Expr)>, methods: Vec<(Ident, Expr)>) -> Object {
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

    pub fn eval_struct_literal(&mut self, name: Ident, field_assignments: Vec<(Ident, Expr)>) -> Object {
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

    pub fn eval_array(&mut self, exprs: Vec<Expr>) -> Object {
        let new_vec = exprs.into_iter().map(|e| self.eval_expr(e)).collect();
        Object::Array(new_vec)
    }

    pub fn eval_hash(&mut self, hs: Vec<(Expr, Expr)>) -> Object {
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

    pub fn eval_index(&mut self, target_exp: Expr, id_exp: Expr) -> Object {
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
}
