use std::{cell::RefCell, rc::Rc};

use crate::{
    ast::ast::{Expr, Ident, Infix, Literal, Prefix, Program, Stmt},
    errors::RuntimeError,
    interpreter::{
        builtins::methods::BuiltinMethods, env::Environment, obj::{BuiltinFunction, Object}
    },
};

pub struct Evaluator {
    env: Rc<RefCell<Environment>>,
}

impl Default for Evaluator {
    fn default() -> Self {
        Self::new()
    }
}

impl Evaluator {
    pub fn new() -> Self {
        Evaluator {
            env: Rc::new(RefCell::new(Environment::new())),
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
                if object.is_returned() {
                    object
                } else {
                    self.eval_blockstmt(prog)
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
            Literal::BoolLiteral(b) => Object::Boolean(b),
            Literal::StringLiteral(s) => Object::String(s),
        }
    }

    pub fn eval_prefix(&mut self, prefix: &Prefix, expr: Expr) -> Object {
        let object = self.eval_expr(expr);
        match *prefix {
            Prefix::Not => match self.otb(object) {
                Ok(b) => Object::Boolean(!b),
                Err(err) => err,
            },
            Prefix::PrefixPlus => match self.oti(object) {
                Ok(i) => Object::Integer(i),
                Err(err) => err,
            },
            Prefix::PrefixMinus => match self.oti(object) {
                Ok(i) => Object::Integer(-i),
                Err(err) => err,
            },
        }
    }

    pub fn eval_infix(&mut self, infix: &Infix, expr1: Expr, expr2: Expr) -> Object {
        let object1 = self.eval_expr(expr1);
        let object2 = self.eval_expr(expr2);
        
        match *infix {
            Infix::Plus => self.object_add(object1, object2),
            Infix::Minus => {
                let i1 = self.oti(object1);
                let i2 = self.oti(object2);
                match (i1, i2) {
                    (Ok(i1), Ok(i2)) => Object::Integer(i1 - i2),
                    (Err(err), _) | (_, Err(err)) => err,
                }
            }
            Infix::Divide => {
                let i1 = self.oti(object1);
                let i2 = self.oti(object2);
                match (i1, i2) {
                    (Ok(_), Ok(0)) => Object::Error(RuntimeError::DivisionByZero),
                    (Ok(i1), Ok(i2)) => Object::Integer(i1 / i2),
                    (Err(err), _) | (_, Err(err)) => err,
                }
            }
            Infix::Multiply => {
                let i1 = self.oti(object1);
                let i2 = self.oti(object2);
                match (i1, i2) {
                    (Ok(i1), Ok(i2)) => Object::Integer(i1 * i2),
                    (Err(err), _) | (_, Err(err)) => err,
                }
            }
            Infix::Equal => Object::Boolean(object1 == object2),
            Infix::NotEqual => Object::Boolean(object1 != object2),
            Infix::GreaterThanEqual => {
                let i1 = self.oti(object1);
                let i2 = self.oti(object2);
                match (i1, i2) {
                    (Ok(i1), Ok(i2)) => Object::Boolean(i1 >= i2),
                    (Err(err), _) | (_, Err(err)) => err,
                }
            }
            Infix::GreaterThan => {
                let i1 = self.oti(object1);
                let i2 = self.oti(object2);
                match (i1, i2) {
                    (Ok(i1), Ok(i2)) => Object::Boolean(i1 > i2),
                    (Err(err), _) | (_, Err(err)) => err,
                }
            }
            Infix::LessThanEqual => {
                let i1 = self.oti(object1);
                let i2 = self.oti(object2);
                match (i1, i2) {
                    (Ok(i1), Ok(i2)) => Object::Boolean(i1 <= i2),
                    (Err(err), _) | (_, Err(err)) => err,
                }
            }
            Infix::LessThan => {
                let i1 = self.oti(object1);
                let i2 = self.oti(object2);
                match (i1, i2) {
                    (Ok(i1), Ok(i2)) => Object::Boolean(i1 < i2),
                    (Err(err), _) | (_, Err(err)) => err,
                }
            }
            Infix::And => {
                let b1 = self.otb(object1);
                let b2 = self.otb(object2);
                match (b1, b2) {
                    (Ok(b1), Ok(b2)) => Object::Boolean(b1 && b2),
                    (Err(err), _) | (_, Err(err)) => err,
                }
            }
            Infix::Or => {
                let b1 = self.otb(object1);
                let b2 = self.otb(object2);
                match (b1, b2) {
                    (Ok(b1), Ok(b2)) => Object::Boolean(b1 || b2),
                    (Err(err), _) | (_, Err(err)) => err,
                }
            }
        }
    }

    pub fn eval_if(&mut self, cond: Expr, conse: Program, maybe_alter: Option<Program>) -> Object {
        let object = self.eval_expr(cond);
        match self.otb(object) {
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
        let fn_ = self.otf(fn_object);
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
        for (_, (Ident(name), o)) in zipped.enumerate() {
            new_env.set(&name, o);
        }
        self.env = Rc::new(RefCell::new(new_env));
        let object = self.eval_blockstmt(body);
        self.env = old_env;
        self.returned(object)
    }

    pub fn eval_method_call(&mut self, object_expr: Expr, method_name: String, args_expr: Vec<Expr>) -> Object {
        let object = self.eval_expr(object_expr);
        let args = args_expr.into_iter().map(|e| self.eval_expr(e)).collect();
        
        match BuiltinMethods::call_method(object, &method_name, args) {
            Ok(obj) => obj,
            Err(e) => Object::Error(e),
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

    pub fn eval_array(&mut self, exprs: Vec<Expr>) -> Object {
        let new_vec = exprs.into_iter().map(|e| self.eval_expr(e)).collect();
        Object::Array(new_vec)
    }

    pub fn object_add(&mut self, object1: Object, object2: Object) -> Object {
        match (object1, object2) {
            (Object::Integer(i1), Object::Integer(i2)) => Object::Integer(i1 + i2),
            (Object::String(s1), Object::String(s2)) => Object::String(s1 + &s2),
            (Object::Error(e), _) | (_, Object::Error(e)) => Object::Error(e),
            (x, y) => Object::Error(RuntimeError::InvalidOperation(format!(
                "cannot add {:?} and {:?}",
                x.type_name(),
                y.type_name()
            ))),
        }
    }

    pub fn eval_hash(&mut self, hs: Vec<(Literal, Expr)>) -> Object {
        let hashmap = hs.into_iter().map(|pair| self.eval_pair(pair)).collect();
        Object::Hash(hashmap)
    }

    fn eval_pair(&mut self, tuple: (Literal, Expr)) -> (Object, Object) {
        let (l, e) = tuple;
        let hash = self.l2h(l);
        let object = self.eval_expr(e);
        (hash, object)
    }

    pub fn eval_index(&mut self, target_exp: Expr, id_exp: Expr) -> Object {
        let target = self.eval_expr(target_exp);
        let index = self.eval_expr(id_exp);
        match target {
            Object::Array(arr) => match self.oti(index) {
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
                let name = self.oth(index);
                match name {
                    Object::Error(_) => name,
                    _ => hash.remove(&name).unwrap_or(Object::Null),
                }
            }
            o => Object::Error(RuntimeError::NotIndexable(o.type_name())),
        }
    }

    pub fn otb(&mut self, object: Object) -> Result<bool, Object> {
        match object {
            Object::Boolean(b) => Ok(b),
            Object::Error(e) => Err(Object::Error(e)),
            o => Err(Object::Error(RuntimeError::TypeMismatch {
                expected: "boolean".to_string(),
                got: o.type_name(),
            })),
        }
    }

    pub fn oti(&mut self, object: Object) -> Result<i64, Object> {
        match object {
            Object::Integer(i) => Ok(i),
            Object::Error(e) => Err(Object::Error(e)),
            o => Err(Object::Error(RuntimeError::TypeMismatch {
                expected: "integer".to_string(),
                got: o.type_name(),
            })),
        }
    }

    pub fn otf(&mut self, object: Object) -> Object {
        match object {
            Object::Function(_, _, _) | Object::Builtin(_, _, _, _) => object,
            Object::Error(e) => Object::Error(e),
            o => Object::Error(RuntimeError::NotCallable(o.type_name())),
        }
    }

    pub fn oth(&mut self, object: Object) -> Object {
        match object {
            Object::Integer(i) => Object::Integer(i),
            Object::Boolean(b) => Object::Boolean(b),
            Object::String(s) => Object::String(s),
            Object::Error(e) => Object::Error(e),
            x => Object::Error(RuntimeError::NotHashable(x.type_name())),
        }
    }

    pub fn l2h(&mut self, literal: Literal) -> Object {
        let object = self.eval_literal(literal);
        self.oth(object)
    }
}