//! Collection compilation: arrays, hashes, indexing, struct literals, field access,
//! method calls, and struct declarations.

use crate::ast::ast::{Expr, Ident};
use crate::vm::obj::Object;
use crate::vm::compiler::Compiler;
use crate::vm::instruction::Instruction;

/// Compiles an array literal: `[e1, e2, e3]`.
pub fn compile_array_expr(compiler: &mut Compiler, elements: &[Expr], line: u16) {
    for element in elements {
        compiler.compile_expression(element, line);
    }
    compiler.emit(Instruction::BuildArray(elements.len() as u16), line);
}

/// Compiles a hash literal: `{k1: v1, k2: v2}`.
pub fn compile_hash_expr(compiler: &mut Compiler, pairs: &[(Expr, Expr)], line: u16) {
    for (key, value) in pairs {
        compiler.compile_expression(key, line);
        compiler.compile_expression(value, line);
    }
    compiler.emit(Instruction::BuildHash(pairs.len() as u16), line);
}

/// Compiles an index expression: `arr[i]` or `hash[key]`.
pub fn compile_index_expr(compiler: &mut Compiler, array: &Expr, index: &Expr, line: u16) {
    compiler.compile_expression(array, line);
    compiler.compile_expression(index, line);
    compiler.emit(Instruction::Index, line);
}

/// Compiles a method call: `obj.method(args...)`.
pub fn compile_method_call(
    compiler: &mut Compiler,
    object: &Expr,
    method: &str,
    arguments: &[Expr],
    line: u16,
) {
    compiler.compile_expression(object, line);

    let method_idx = compiler
        .chunk
        .add_constant(Object::String(method.to_string()));
    if let Some(method_idx) = method_idx {
        compiler.emit(Instruction::Constant(method_idx), line);
    }

    for arg in arguments {
        compiler.compile_expression(arg, line);
    }

    compiler.emit(Instruction::CallMethod(arguments.len() as u8), line);
}

/// Compiles a struct literal: `Name { field1: e1, field2: e2 }`.
pub fn compile_struct_literal(
    compiler: &mut Compiler,
    name: &Ident,
    fields: &[(Ident, Expr)],
    line: u16,
) {
    let name_idx = compiler
        .chunk
        .add_constant(Object::String(name.name.clone()));
    if let Some(name_idx) = name_idx {
        compiler.emit(Instruction::Constant(name_idx), line);
    }

    for (_, expr) in fields {
        compiler.compile_expression(expr, line);
    }

    compiler.emit(Instruction::BuildStruct(fields.len() as u8), line);
}

/// Compiles a field access: `obj.field`.
pub fn compile_field_access(compiler: &mut Compiler, object: &Expr, field: &str, line: u16) {
    compiler.compile_expression(object, line);
    let field_idx = compiler
        .chunk
        .add_constant(Object::String(field.to_string()));
    if let Some(field_idx) = field_idx {
        compiler.emit(Instruction::Constant(field_idx), line);
        compiler.emit(Instruction::GetField, line);
    }
}

/// Compiles a struct declaration: `struct Name { fields..., methods... }`.
pub fn compile_struct_stmt(
    compiler: &mut Compiler,
    name: &Ident,
    fields: &[(Ident, Expr)],
    methods: &[(Ident, Expr)],
    line: u16,
) {
    use ahash::AHasher;
    use std::hash::BuildHasherDefault;
    type HashMap<K, V> = std::collections::HashMap<K, V, BuildHasherDefault<AHasher>>;

    let mut field_map: HashMap<String, Object> = HashMap::default();
    let mut method_map: HashMap<String, Object> = HashMap::default();

    for (ident, _expr) in fields {
        compiler.compile_expression(_expr, line);
        field_map.insert(ident.name.clone(), Object::Null);
    }

    for (ident, _expr) in methods {
        if let Expr::FnExpr { params, body } = _expr {
            let (_fn_chunk, _param_count, _local_names) =
                crate::vm::compiler::Compiler::compile_function_body(params, body, false);
            method_map.insert(ident.name.clone(), Object::Null);
        }
    }

    let struct_obj = Object::Struct {
        name: name.name.clone(),
        fields: field_map,
        methods: method_map,
        constants: crate::vm::obj::ConstantPool::new(),
    };

    let struct_idx = compiler.chunk.add_constant(struct_obj);
    if let Some(struct_idx) = struct_idx {
        compiler.emit(Instruction::Constant(struct_idx), line);
    }

    let name_idx = compiler
        .chunk
        .add_constant(Object::String(name.name.clone()));
    if let Some(name_idx) = name_idx {
        compiler.emit(Instruction::SetGlobal(name_idx as u16), line);
    }
}
