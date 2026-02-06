use std::{cell::RefCell,rc::Rc};

use num_bigint::BigInt;

use crate::{
    ast::ast::{Expr, Ident, Infix, Literal, Prefix, Program, Stmt},
    errors::RuntimeError,
    interpreter::{
        env::Environment, module_registry::ModuleRegistry, obj::Object
    },
};
use crate::interpreter::helpers::eval_helpers::EvalHelper;

pub struct Evaluator {
    pub(crate) env: Rc<RefCell<Environment>>,
    pub(crate) module_registry: Rc<RefCell<ModuleRegistry>>,
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

    pub(crate) fn returned(&mut self, object: Object) -> Object {
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
            1 => {
                let stmt = prog.remove(0);
                let result = self.eval_statement(stmt);
                // Only unwrap return values at function level, not at block level
                match result {
                    Object::ReturnValue(_) | Object::Break | 
                    Object::Continue | Object::Error(_) => result,
                    other => other  // Last statement's value (or Null) is returned
                }
            }
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
            Stmt::ExprStmt(expr) => {
                let result = self.eval_expr(expr);
                // Propagate control flow objects (return, break, continue, error)
                match result {
                    Object::ReturnValue(_) | Object::Break | Object::Continue | Object::Error(_) => result,
                    _ => Object::Null  // Expression statements don't produce values in normal flow
                }
            }
            Stmt::ExprValueStmt(expr) => {
                self.eval_expr(expr)
            }
            Stmt::ReturnStmt(expr) => Object::ReturnValue(Box::new(self.eval_expr(expr))),
            Stmt::LetStmt(ident, expr) => {
                let object = self.eval_expr(expr);
                self.register_ident(ident, object)
            }
            Stmt::FnStmt { name, params, body } => {
                let fn_obj = Object::Function(params, body, Rc::clone(&self.env));
                self.register_ident(name, fn_obj)
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
            Stmt::FieldAssignStmt { object, field, value } => {
                self.eval_field_assign(*object, field, *value)
            }
            Stmt::IndexAssignStmt { target, index, value } => {
                self.eval_index_assign(*target, *index, *value)
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
            Expr::CStyleForExpr { init, cond, update, body } => {
                self.eval_c_style_for(init, cond, update, body)
            }
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
            Object::BuiltinStd(_, min_params, max_params, s_fn) => {
                self.eval_std_call(args_expr, min_params, max_params, s_fn)
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
}