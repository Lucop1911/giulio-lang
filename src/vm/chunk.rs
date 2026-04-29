//! Bytecode chunk — a unit of compiled code.
//!
//! A `Chunk` holds the raw bytecode for a single function, block, or
//! top-level program, along with its constant pool and source-line
//! mapping for error reporting.
//!
//! Chunks are reference-counted (`Arc<Chunk>`) so that closures can
//! share their body code across multiple invocations.

use crate::vm::obj::Object;
use crate::vm::instruction::{encode_instruction, Instruction};

/// A compiled unit of bytecode.
///
/// Each chunk represents either:
/// - The top-level program
/// - A function body
/// - A closure body
/// - A try/catch block (as a sub-chunk)
#[derive(Clone, Debug)]
pub struct Chunk {
    /// Raw bytecode instructions.
    pub code: Vec<u8>,
    /// Constant pool — literals, strings, and builtin references
    /// indexed by `u16` from `OpConstant`.
    pub constants: Vec<Object>,
    /// Source line for each byte in `code`.
    /// `lines[i]` is the source line of `code[i]`.
    pub lines: Vec<u16>,
}

impl Chunk {
    pub fn new() -> Self {
        Chunk {
            code: Vec::new(),
            constants: Vec::new(),
            lines: Vec::new(),
        }
    }

    /// Adds a constant to the pool and returns its index.
    ///
    /// Returns `None` if the pool is full (65536 entries).
    pub fn add_constant(&mut self, value: Object) -> Option<u16> {
        if self.constants.len() >= u16::MAX as usize {
            return None;
        }
        let index = self.constants.len() as u16;
        self.constants.push(value);
        Some(index)
    }

    /// Appends a single byte to the bytecode, recording its source line.
    pub fn write_byte(&mut self, byte: u8, line: u16) {
        self.code.push(byte);
        self.lines.push(line);
    }

    /// Encodes and appends an instruction, recording the source line
    /// for every byte (opcode + operands).
    pub fn write_instruction(&mut self, instr: Instruction, line: u16) {
        let start = self.code.len();
        encode_instruction(&mut self.code, instr);
        for _ in start..self.code.len() {
            self.lines.push(line);
        }
    }

    /// Returns the current bytecode length (used as a jump target).
    pub fn current_offset(&self) -> u16 {
        self.code.len() as u16
    }

    /// Patches a u16 operand at the given offset in the bytecode.
    ///
    /// Used for forward-jump address backpatching once the target
    /// address is known.
    pub fn patch_u16(&mut self, offset: usize, value: u16) {
        let bytes = value.to_be_bytes();
        self.code[offset] = bytes[0];
        self.code[offset + 1] = bytes[1];
    }
}

impl Default for Chunk {
    fn default() -> Self {
        Self::new()
    }
}
