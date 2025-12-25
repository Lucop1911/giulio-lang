use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub enum LangError {
    Lexer(LexerError),
    Parser(ParserError),
    Runtime(RuntimeError),
}

#[derive(Debug, Clone, PartialEq)]
pub enum LexerError {
    InvalidToken(String),
    UnexpectedCharacter(char),
    UnterminatedString,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ParserError {
    UnexpectedToken(String),
    ExpectedToken { expected: String, found: String },
    InvalidExpression(String),
    UnexpectedEOF,
}

#[derive(Debug, Clone, PartialEq)]
pub enum RuntimeError {
    TypeMismatch { expected: String, got: String },
    UndefinedVariable(String),
    InvalidOperation(String),
    DivisionByZero,
    ModuloByZero,
    IndexOutOfBounds { index: i64, length: usize },
    WrongNumberOfArguments { min: usize, max: usize, got: usize },
    NotCallable(String),
    NotHashable(String),
    NotIndexable(String),
    EmptyArray,
    InvalidArguments(String),
}

impl fmt::Display for LangError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            LangError::Lexer(e) => write!(f, "Lexer Error: {}", e),
            LangError::Parser(e) => write!(f, "Parser Error: {}", e),
            LangError::Runtime(e) => write!(f, "Runtime Error: {}", e),
        }
    }
}

impl fmt::Display for LexerError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            LexerError::InvalidToken(s) => write!(f, "Invalid token: {}", s),
            LexerError::UnexpectedCharacter(c) => write!(f, "Unexpected character: '{}'", c),
            LexerError::UnterminatedString => write!(f, "Unterminated string literal"),
        }
    }
}

impl fmt::Display for ParserError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ParserError::UnexpectedToken(s) => write!(f, "Unexpected token: {}", s),
            ParserError::ExpectedToken { expected, found } => {
                write!(f, "Expected '{}', found '{}'", expected, found)
            }
            ParserError::InvalidExpression(s) => write!(f, "Invalid expression: {}", s),
            ParserError::UnexpectedEOF => write!(f, "Unexpected end of file"),
        }
    }
}

impl fmt::Display for RuntimeError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            RuntimeError::TypeMismatch { expected, got } => {
                write!(f, "Type mismatch: expected {}, got {}", expected, got)
            }
            RuntimeError::UndefinedVariable(name) => {
                write!(f, "Undefined variable: '{}'", name)
            }
            RuntimeError::InvalidOperation(op) => write!(f, "Invalid operation: {}", op),
            RuntimeError::DivisionByZero => write!(f, "Division by zero"),
            RuntimeError::ModuloByZero => write!(f, "Modulo by zero"),
            RuntimeError::IndexOutOfBounds { index, length } => {
                write!(f, "Index {} out of bounds for array of length {}", index, length)
            }
            RuntimeError::WrongNumberOfArguments { min, max, got } => {
                if min != max {
                    write!(f, "Wrong number of arguments: min {}, max: {} got {}", min, max, got)
                } else {
                    write!(f, "Wrong number of arguments: expected {} got {}", min, got)
                }
            }
            RuntimeError::NotCallable(s) => write!(f, "{} is not callable", s),
            RuntimeError::NotHashable(s) => write!(f, "{} is not hashable", s),
            RuntimeError::NotIndexable(s) => write!(f, "{} is not indexable", s),
            RuntimeError::EmptyArray => write!(f, "Cannot perform operation on empty array"),
            RuntimeError::InvalidArguments(s) => write!(f, "Invalid arguments: {}", s),
        }
    }
}

impl std::error::Error for LangError {}
impl std::error::Error for LexerError {}
impl std::error::Error for ParserError {}
impl std::error::Error for RuntimeError {}