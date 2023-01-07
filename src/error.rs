use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("[Compile Error] {0}")]
    CompileError(CompileError),
    #[error("[Parse error] {0}")]
    ParseError(ParseError),
    #[error("[Preprocessor Error] {0}")]
    PreprocessorError(PreprocessorError),
    #[error("[Runtime Error] {0}")]
    RuntimeError(RuntimeError),
    #[error("[IO Error] {0}")]
    IOError(IOError),
}

#[derive(Error, Debug)]
pub enum IOError {
    #[error("{0}")]
    Inherited(#[from] std::io::Error),
    #[error("Could not load file")]
    FileLoadError,
}

#[derive(Error, Debug)]
pub enum CompileError {
    #[error("Nasm invoke error")]
    NasmInvokeError,
}

#[derive(Error, Debug)]
pub enum ParseError {
    #[error("Failed to parse program")]
    Incomplete,
    #[error("Unexpected token: {0}")]
    UnexpectedToken(String),
    #[error("Unknown operator")]
    UnknownOperator,
    #[error("Unknown keyword")]
    UnknownKeyword,
    #[error("Unexpected end of file")]
    UnexpectedEof,
    #[error("Unexpected end of line")]
    UnexpectedEol,
    #[error("Unexpected character: {0}")]
    UnexpectedChar(char),
    #[error("Unexpected end of string")]
    UnexpectedEos,
}

#[derive(Error, Debug)]
pub enum PreprocessorError {
    #[error("Unexpected keyword")]
    UnexpectedKeyword,
}

#[derive(Error, Debug)]
pub enum RuntimeError {
    #[error("Invalid instruction pointer: {0}")]
    InvalidInstructionPointer(usize),
}
