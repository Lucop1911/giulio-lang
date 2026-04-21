use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub enum CompilationError {
    ConstantPoolOverflow,
}

impl fmt::Display for CompilationError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            CompilationError::ConstantPoolOverflow => {
                write!(f, "Constant pool overflow (max 65536 entries)")
            }
        }
    }
}

impl std::error::Error for CompilationError {}
