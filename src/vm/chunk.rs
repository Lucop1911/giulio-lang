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

/// Run-length encoded line number info.
/// Each entry is (byte_count, line_number).
/// This reduces memory from 1:1 with code bytes to ~1:10 typical.
#[derive(Clone, Debug)]
pub struct LineInfo {
    pub entries: Vec<(u32, u16)>,
}

impl LineInfo {
    pub fn new() -> Self {
        LineInfo { entries: Vec::new() }
    }

    pub fn add_line(&mut self, count: usize, line: u16) {
        if let Some((_, last_line)) = self.entries.last_mut() {
            if *last_line == line {
                self.entries.last_mut().unwrap().0 += count as u32;
                return;
            }
        }
        self.entries.push((count as u32, line));
    }

    pub fn get_line(&self, byte_offset: usize) -> Option<u16> {
        let mut offset = 0;
        for (count, line) in &self.entries {
            if offset + *count as usize > byte_offset {
                return Some(*line);
            }
            offset += *count as usize;
        }
        None
    }
}

impl Default for LineInfo {
    fn default() -> Self {
        Self::new()
    }
}

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
    /// Source line mapping using run-length encoding.
    /// Each entry is (byte_count, line_number).
    pub lines: LineInfo,
}

impl Chunk {
    pub fn new() -> Self {
        Chunk {
            code: Vec::new(),
            constants: Vec::new(),
            lines: LineInfo::new(),
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
        self.lines.add_line(1, line);
    }

    /// Encodes and appends an instruction, recording the source line
    /// for every byte (opcode + operands).
    pub fn write_instruction(&mut self, instr: Instruction, line: u16) {
        let start = self.code.len();
        encode_instruction(&mut self.code, instr);
        let byte_count = self.code.len() - start;
        self.lines.add_line(byte_count, line);
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
