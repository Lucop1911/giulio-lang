//! Constant pool for function-local literal optimization.
//!
//! During the compiler pass, literal values (`LitExpr`) inside function
//! bodies are replaced with `LitIndex` references into a per-function
//! `ConstantPool`. This avoids re-allocating and re-wrapping the same
//! literal `Object` on every evaluation.
//!
//! The pool is stored alongside the function body in the `Object::Function`
//! variant and accessed via `eval_expr` during interpretation.

use crate::ast::ast::{Expr, Literal, Program, Stmt};
use crate::vm::obj::Object;

#[derive(Clone, Default)]
pub struct ConstantPool(pub Vec<Object>);

impl ConstantPool {
    pub fn new() -> Self {
        ConstantPool(Vec::new())
    }

    pub fn add(&mut self, obj: Object) -> usize {
        self.0.push(obj);
        self.0.len() - 1
    }

    pub fn get(&self, index: usize) -> Option<&Object> {
        self.0.get(index)
    }

    pub fn from_program(program: &Program) -> (Program, ConstantPool) {
        let mut pool = ConstantPool::new();
        let processed_program = program
            .iter()
            .map(|stmt| Self::process_stmt(stmt, &mut pool))
            .collect();

        (processed_program, pool)
    }

    fn process_stmt(stmt: &Stmt, pool: &mut ConstantPool) -> Stmt {
        match stmt {
            Stmt::LetStmt(ident, expr) => {
                Stmt::LetStmt(ident.clone(), Self::process_expr(expr, pool))
            }
            Stmt::MultiLetStmt { idents, values } => Stmt::MultiLetStmt {
                idents: idents.clone(),
                values: values.iter().map(|v| Self::process_expr(v, pool)).collect(),
            },
            Stmt::AssignStmt(ident, expr) => {
                Stmt::AssignStmt(ident.clone(), Self::process_expr(expr, pool))
            }
            Stmt::TupleAssignStmt { targets, values } => Stmt::TupleAssignStmt {
                targets: targets.clone(),
                values: values.iter().map(|v| Self::process_expr(v, pool)).collect(),
            },
            Stmt::FieldAssignStmt {
                object,
                field,
                value,
            } => Stmt::FieldAssignStmt {
                object: Box::new(Self::process_expr(object, pool)),
                field: field.clone(),
                value: Box::new(Self::process_expr(value, pool)),
            },
            Stmt::IndexAssignStmt {
                target,
                index,
                value,
            } => Stmt::IndexAssignStmt {
                target: Box::new(Self::process_expr(target, pool)),
                index: Box::new(Self::process_expr(index, pool)),
                value: Box::new(Self::process_expr(value, pool)),
            },
            Stmt::ReturnStmt(expr) => Stmt::ReturnStmt(Self::process_expr(expr, pool)),
            Stmt::ExprStmt(expr) => Stmt::ExprStmt(Self::process_expr(expr, pool)),
            Stmt::ExprValueStmt(expr) => Stmt::ExprValueStmt(Self::process_expr(expr, pool)),
            Stmt::FnStmt { name, params, body } => Stmt::FnStmt {
                name: name.clone(),
                params: params.clone(),
                body: body.clone(),
            },
            Stmt::StructStmt {
                name,
                fields,
                methods,
            } => Stmt::StructStmt {
                name: name.clone(),
                fields: fields
                    .iter()
                    .map(|(i, e)| (i.clone(), Self::process_expr(e, pool)))
                    .collect(),
                methods: methods
                    .iter()
                    .map(|(i, e)| (i.clone(), Self::process_expr(e, pool)))
                    .collect(),
            },
            Stmt::ImportStmt { path, items } => Stmt::ImportStmt {
                path: path.clone(),
                items: items.clone(),
            },
            Stmt::BreakStmt => Stmt::BreakStmt,
            Stmt::ContinueStmt => Stmt::ContinueStmt,
            Stmt::ThrowStmt(expr) => Stmt::ThrowStmt(Self::process_expr(expr, pool)),
        }
    }

    fn process_expr(expr: &Expr, pool: &mut ConstantPool) -> Expr {
        match expr {
            Expr::IdentExpr(ident) => Expr::IdentExpr(ident.clone()),
            Expr::LitExpr(literal) => {
                let obj = match literal {
                    Literal::IntLiteral(i) => Object::Integer(*i),
                    Literal::BigIntLiteral(b) => Object::BigInteger(b.clone()),
                    Literal::FloatLiteral(f) => Object::Float(*f),
                    Literal::BoolLiteral(b) => Object::Boolean(*b),
                    Literal::StringLiteral(s) => Object::String(s.clone()),
                    Literal::NullLiteral => Object::Null,
                };
                let index = pool.add(obj);
                Expr::LitIndex(index)
            }
            Expr::LitIndex(idx) => Expr::LitIndex(*idx),
            Expr::PrefixExpr(op, e) => {
                Expr::PrefixExpr(op.clone(), Box::new(Self::process_expr(e, pool)))
            }
            Expr::InfixExpr(op, l, r) => Expr::InfixExpr(
                op.clone(),
                Box::new(Self::process_expr(l, pool)),
                Box::new(Self::process_expr(r, pool)),
            ),
            Expr::IfExpr {
                cond,
                consequence,
                alternative,
            } => Expr::IfExpr {
                cond: Box::new(Self::process_expr(cond, pool)),
                consequence: consequence
                    .iter()
                    .map(|s| Self::process_stmt(s, pool))
                    .collect(),
                alternative: alternative
                    .as_ref()
                    .map(|alt| alt.iter().map(|s| Self::process_stmt(s, pool)).collect()),
            },
            Expr::FnExpr { params, body } => Expr::FnExpr {
                params: params.clone(),
                body: body.clone(),
            },
            Expr::CallExpr {
                function,
                arguments,
            } => Expr::CallExpr {
                function: Box::new(Self::process_expr(function, pool)),
                arguments: arguments
                    .iter()
                    .map(|a| Self::process_expr(a, pool))
                    .collect(),
            },
            Expr::ArrayExpr(arr) => {
                Expr::ArrayExpr(arr.iter().map(|e| Self::process_expr(e, pool)).collect())
            }
            Expr::HashExpr(pairs) => Expr::HashExpr(
                pairs
                    .iter()
                    .map(|(k, v)| (Self::process_expr(k, pool), Self::process_expr(v, pool)))
                    .collect(),
            ),
            Expr::IndexExpr { array, index } => Expr::IndexExpr {
                array: Box::new(Self::process_expr(array, pool)),
                index: Box::new(Self::process_expr(index, pool)),
            },
            Expr::MethodCallExpr {
                object,
                method,
                arguments,
            } => Expr::MethodCallExpr {
                object: Box::new(Self::process_expr(object, pool)),
                method: method.clone(),
                arguments: arguments
                    .iter()
                    .map(|a| Self::process_expr(a, pool))
                    .collect(),
            },
            Expr::StructLiteral { name, fields } => Expr::StructLiteral {
                name: name.clone(),
                fields: fields
                    .iter()
                    .map(|(i, e)| (i.clone(), Self::process_expr(e, pool)))
                    .collect(),
            },
            Expr::ThisExpr => Expr::ThisExpr,
            Expr::FieldAccessExpr { object, field } => Expr::FieldAccessExpr {
                object: Box::new(Self::process_expr(object, pool)),
                field: field.clone(),
            },
            Expr::WhileExpr { cond, body } => Expr::WhileExpr {
                cond: Box::new(Self::process_expr(cond, pool)),
                body: body.iter().map(|s| Self::process_stmt(s, pool)).collect(),
            },
            Expr::ForExpr {
                ident,
                iterable,
                body,
            } => Expr::ForExpr {
                ident: ident.clone(),
                iterable: Box::new(Self::process_expr(iterable, pool)),
                body: body.iter().map(|s| Self::process_stmt(s, pool)).collect(),
            },
            Expr::CStyleForExpr {
                init,
                cond,
                update,
                body,
            } => Expr::CStyleForExpr {
                init: init.as_ref().map(|s| Box::new(Self::process_stmt(s, pool))),
                cond: cond.as_ref().map(|e| Box::new(Self::process_expr(e, pool))),
                update: update
                    .as_ref()
                    .map(|s| Box::new(Self::process_stmt(s, pool))),
                body: body.iter().map(|s| Self::process_stmt(s, pool)).collect(),
            },
            Expr::TryCatchExpr {
                try_body,
                catch_ident,
                catch_body,
                finally_body,
            } => Expr::TryCatchExpr {
                try_body: try_body
                    .iter()
                    .map(|s| Self::process_stmt(s, pool))
                    .collect(),
                catch_ident: catch_ident.clone(),
                catch_body: catch_body
                    .as_ref()
                    .map(|b| b.iter().map(|s| Self::process_stmt(s, pool)).collect()),
                finally_body: finally_body
                    .as_ref()
                    .map(|b| b.iter().map(|s| Self::process_stmt(s, pool)).collect()),
            },
            Expr::AsyncFnExpr { params, body } => Expr::AsyncFnExpr {
                params: params.clone(),
                body: body.clone(),
            },
            Expr::AwaitExpr(e) => Expr::AwaitExpr(Box::new(Self::process_expr(e, pool))),
        }
    }
}
