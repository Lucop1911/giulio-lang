//! Statement compilation: let, assign, return, import, expression statements.

use crate::ast::ast::{Expr, Ident, ImportItems, SlotIndex};
use crate::runtime::obj::Object;
use crate::vm::compiler::Compiler;
use crate::vm::instruction::Instruction;

/// Compiles a `let name = expr;` statement.
///
/// For top-level lets (global scope), emits `SetGlobal`.
/// For function-local lets, emits `SetLocal` using the pre-computed slot.
pub fn compile_let_stmt(compiler: &mut Compiler, ident: &Ident, expr: &Expr, line: u16) {
    compiler.compile_expression(expr, line);

    if ident.slot != SlotIndex::UNSET {
        compiler.emit(Instruction::SetLocal(ident.slot.0 as u8), line);
    } else {
        let idx = compiler
            .chunk
            .add_constant(Object::String(ident.name.clone()));
        if let Some(idx) = idx {
            compiler.emit(Instruction::SetGlobal(idx as u16), line);
        }
    }
}

/// Compiles a multi-let destructuring: `let (a, b) = (expr1, expr2);`.
pub fn compile_multi_let(compiler: &mut Compiler, idents: &[Ident], values: &[Expr], line: u16) {
    // Compile each value and assign to each ident
    for (ident, value) in idents.iter().zip(values.iter()) {
        compiler.compile_expression(value, line);
        if ident.slot != SlotIndex::UNSET {
            compiler.emit(Instruction::SetLocal(ident.slot.0 as u8), line);
        } else {
            let idx = compiler
                .chunk
                .add_constant(Object::String(ident.name.clone()));
            if let Some(idx) = idx {
                compiler.emit(Instruction::SetGlobal(idx as u16), line);
            }
        }
    }
}

/// Compiles a simple assignment: `name = expr;`.
pub fn compile_assign(compiler: &mut Compiler, ident: &Ident, expr: &Expr, line: u16) {
    compiler.compile_expression(expr, line);

    if ident.slot != SlotIndex::UNSET {
        compiler.emit(Instruction::SetLocal(ident.slot.0 as u8), line);
    } else {
        let idx = compiler
            .chunk
            .add_constant(Object::String(ident.name.clone()));
        if let Some(idx) = idx {
            compiler.emit(Instruction::SetGlobal(idx as u16), line);
        }
    }
}

/// Compiles a tuple assignment: `(a, b) = (expr1, expr2);`.
///
/// All values are evaluated first (left-to-right), then assigned
/// left-to-right. This ensures swap semantics work: `(a, b) = (b, a)`.
pub fn compile_tuple_assign(
    compiler: &mut Compiler,
    targets: &[Ident],
    values: &[Expr],
    line: u16,
) {
    // Evaluate all values first
    for value in values.iter() {
        compiler.compile_expression(value, line);
    }
    // Assign from right to left so that leftmost value is at stack bottom
    // and rightmost is at stack top. Pop each value and assign to target.
    for target in targets.iter().rev() {
        // The values are stacked left-to-right.
        // We need to assign targets[i] = values[i].
        // Since we're iterating in reverse, we need to get values[i] from the stack.
        // The stack has: [v0, v1, ..., vn-1] (vn-1 on top)
        // We iterate targets from right to left: target[n-1], target[n-2], ...
        // target[n-1] should get vn-1 (top of stack)
        if target.slot != SlotIndex::UNSET {
            compiler.emit(Instruction::SetLocal(target.slot.0 as u8), line);
        } else {
            let idx = compiler
                .chunk
                .add_constant(Object::String(target.name.clone()));
            if let Some(idx) = idx {
                compiler.emit(Instruction::SetGlobal(idx as u16), line);
            }
        }
    }
}

/// Compiles a field assignment: `obj.field = expr;`.
pub fn compile_field_assign(
    compiler: &mut Compiler,
    object: &Expr,
    field: &str,
    value: &Expr,
    line: u16,
) {
    compiler.compile_expression(object, line);
    compiler.compile_expression(value, line);
    // Store field name as constant for the VM to look up
    let idx = compiler
        .chunk
        .add_constant(Object::String(field.to_string()));
    if let Some(idx) = idx {
        compiler.emit(Instruction::Constant(idx), line);
        compiler.emit(Instruction::SetField, line);
    }
}

/// Compiles an index assignment: `arr[i] = expr;`.
pub fn compile_index_assign(
    compiler: &mut Compiler,
    target: &Expr,
    index: &Expr,
    value: &Expr,
    line: u16,
) {
    compiler.compile_expression(target, line);
    compiler.compile_expression(index, line);
    compiler.compile_expression(value, line);
    compiler.emit(Instruction::SetIndex, line);
}

/// Compiles a `return expr;` statement.
pub fn compile_return_stmt(compiler: &mut Compiler, expr: &Expr, line: u16) {
    compiler.compile_expression(expr, line);
    compiler.emit(Instruction::ReturnValue, line);
}

/// Compiles an `import path::{items};` statement.
///
/// Emits `ImportModule` for the path, then `GetExport` for each
/// imported name, and stores each export as a global variable.
pub fn compile_import_stmt(
    compiler: &mut Compiler,
    path: &[String],
    items: &ImportItems,
    line: u16,
) {
    let module_path = path.join("::");

    // Push module path constant
    let path_idx = compiler.chunk.add_constant(Object::String(module_path));
    if let Some(path_idx) = path_idx {
        compiler.emit(Instruction::ImportModule(path_idx as u16), line);
    }

    match items {
        ImportItems::All => {
            // Store the module object as a global using the last path component
            let module_name = path.last().cloned().unwrap_or_default();
            let var_idx = compiler.chunk.add_constant(Object::String(module_name));
            if let Some(var_idx) = var_idx {
                compiler.emit(Instruction::SetGlobal(var_idx as u16), line);
            }
        }
        ImportItems::Specific(names) => {
            for name in names {
                let name_idx = compiler.chunk.add_constant(Object::String(name.clone()));
                if let Some(name_idx) = name_idx {
                    compiler.emit(Instruction::Constant(name_idx), line);
                    compiler.emit(Instruction::GetExport, line);
                    // Store as global
                    let var_idx = compiler.chunk.add_constant(Object::String(name.clone()));
                    if let Some(var_idx) = var_idx {
                        compiler.emit(Instruction::SetGlobal(var_idx as u16), line);
                    }
                }
            }
        }
        ImportItems::Single(name) => {
            let name_idx = compiler.chunk.add_constant(Object::String(name.clone()));
            if let Some(name_idx) = name_idx {
                compiler.emit(Instruction::Constant(name_idx), line);
                compiler.emit(Instruction::GetExport, line);
                let var_idx = compiler.chunk.add_constant(Object::String(name.clone()));
                if let Some(var_idx) = var_idx {
                    compiler.emit(Instruction::SetGlobal(var_idx as u16), line);
                }
            }
        }
    }
}
