use super::super::super::eval::Evaluator;
use crate::{
    ast::ast::{Expr, Ident, Program, SlotIndex},
    interpreter::obj::Object,
};
use std::sync::Arc;

impl Evaluator {
    pub fn eval_fn(&self, params: Vec<Ident>, body: Program) -> Object {
        let params_with_slots = Self::ensure_param_slots(params);
        let body = Self::ensure_body_slots(
            params_with_slots
                .iter()
                .map(|p| (p.name.clone(), p.slot))
                .collect(),
            body,
        );
        Object::Function(params_with_slots, body, Arc::clone(&self.context.env))
    }

    pub fn eval_method(&self, params: Vec<Ident>, body: Program) -> Object {
        let params_with_slots = Self::ensure_param_slots(params);
        let body = Self::ensure_body_slots(
            params_with_slots
                .iter()
                .map(|p| (p.name.clone(), p.slot))
                .collect(),
            body,
        );
        Object::Method(params_with_slots, body, Arc::clone(&self.context.env))
    }

    fn ensure_param_slots(mut params: Vec<Ident>) -> Vec<Ident> {
        for (i, param) in params.iter_mut().enumerate() {
            if param.slot.is_unset() {
                param.slot = SlotIndex(i as u16);
            }
        }
        params
    }

    fn ensure_body_slots(locals: Vec<(String, SlotIndex)>, body: Program) -> Program {
        let mut body = body;
        Self::process_body_for_slots(&locals, &mut body);
        body
    }

    fn process_body_for_slots(locals: &[(String, SlotIndex)], body: &mut Program) {
        for stmt in body.iter_mut() {
            Self::process_stmt_for_slots(locals, stmt);
        }
    }

    fn process_expr_for_slots(locals: &[(String, SlotIndex)], expr: &mut Expr) {
        match expr {
            Expr::IdentExpr(ident) => {
                if let Some((_, slot)) = locals.iter().rev().find(|(n, _)| n == &ident.name) {
                    ident.slot = *slot;
                }
            }
            Expr::InfixExpr(_, left, right) => {
                Self::process_expr_for_slots(locals, left);
                Self::process_expr_for_slots(locals, right);
            }
            Expr::PrefixExpr(_, e) => {
                Self::process_expr_for_slots(locals, e);
            }
            Expr::CallExpr {
                function,
                arguments,
            } => {
                Self::process_expr_for_slots(locals, function);
                for a in arguments {
                    Self::process_expr_for_slots(locals, a);
                }
            }
            Expr::IfExpr {
                cond,
                consequence,
                alternative,
            } => {
                Self::process_expr_for_slots(locals, cond);
                for s in consequence.iter_mut() {
                    Self::process_stmt_for_slots(locals, s);
                }
                if let Some(alt) = alternative {
                    for s in alt.iter_mut() {
                        Self::process_stmt_for_slots(locals, s);
                    }
                }
            }
            Expr::ArrayExpr(es) => {
                for e in es {
                    Self::process_expr_for_slots(locals, e);
                }
            }
            Expr::HashExpr(kvs) => {
                for (k, v) in kvs {
                    Self::process_expr_for_slots(locals, k);
                    Self::process_expr_for_slots(locals, v);
                }
            }
            Expr::IndexExpr { array, index } => {
                Self::process_expr_for_slots(locals, array);
                Self::process_expr_for_slots(locals, index);
            }
            Expr::MethodCallExpr {
                object, arguments, ..
            } => {
                Self::process_expr_for_slots(locals, object);
                for a in arguments {
                    Self::process_expr_for_slots(locals, a);
                }
            }
            Expr::FieldAccessExpr { object, .. } => {
                Self::process_expr_for_slots(locals, object);
            }
            Expr::AwaitExpr(e) => Self::process_expr_for_slots(locals, e),
            Expr::WhileExpr { cond, body } => {
                Self::process_expr_for_slots(locals, cond);
                for s in body.iter_mut() {
                    Self::process_stmt_for_slots(locals, s);
                }
            }
            Expr::ForExpr { iterable, body, .. } => {
                Self::process_expr_for_slots(locals, iterable);
                for s in body.iter_mut() {
                    Self::process_stmt_for_slots(locals, s);
                }
            }
            // FnExpr/AsyncFnExpr create new scopes — compute_slots handles them.
            // LitExpr, ThisExpr have no idents to slot.
            _ => {}
        }
    }

    fn process_stmt_for_slots(locals: &[(String, SlotIndex)], stmt: &mut crate::ast::ast::Stmt) {
        use crate::ast::ast::Stmt;
        match stmt {
            Stmt::ExprStmt(e)
            | Stmt::ExprValueStmt(e)
            | Stmt::ReturnStmt(e)
            | Stmt::ThrowStmt(e) => {
                Self::process_expr_for_slots(locals, e);
            }
            Stmt::LetStmt(ident, e) => {
                Self::process_expr_for_slots(locals, e);
                if let Some((_, slot)) = locals.iter().rev().find(|(n, _)| n == &ident.name) {
                    ident.slot = *slot;
                }
            }
            Stmt::AssignStmt(ident, e) => {
                Self::process_expr_for_slots(locals, e);
                if let Some((_, slot)) = locals.iter().rev().find(|(n, _)| n == &ident.name) {
                    ident.slot = *slot;
                }
            }
            _ => {}
        }
    }
}
