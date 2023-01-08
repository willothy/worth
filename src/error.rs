use thiserror::Error;

pub trait BoolError {
    fn to_err(self) -> anyhow::Result<(), ()>;
}

impl BoolError for bool {
    fn to_err(self) -> anyhow::Result<(), ()> {
        if self {
            Ok(())
        } else {
            Err(())
        }
    }
}

#[derive(Error, Debug)]
pub enum Error<'e> {
    #[error("[Compile Error] {0}")]
    CompileError(CompileError),
    #[error("[Parse Error] {0}")]
    ParseError(ParseError),
    #[error("[Preprocessor Error] {0}")]
    PreprocessorError(PreprocessorError),
    #[error("[Runtime Error] {0}")]
    RuntimeError(RuntimeError),
    #[error("[Runner Error] {0}")]
    RunnerError(RunnerError),
    #[error("[Typecheck Error] {0}")]
    TypecheckError(TypecheckError<'e>),
    #[error("[IO Error] {0}")]
    IOError(IOError),
}

#[derive(Error, Debug)]
pub enum TypecheckError<'e> {
    #[error("Stack Underflow")]
    StackUnderflow,
    #[error("Invalid type for operation {0}")]
    InvalidTypeForOp(&'e str),
}

#[derive(Error, Debug)]
pub enum RunnerError {
    #[error("Failed to invoke program: {0}")]
    InvokeError(std::io::Error),
    #[error("Program exited with non-zero status: {0}")]
    NonZeroStatus(usize),
}

#[derive(Error, Debug)]
pub enum IOError {
    #[error("{0}")]
    Inherited(#[from] std::io::Error),
    #[error("Invalid filename")]
    InvalidFilename,
    #[error("Invalid path")]
    InvalidPath,
    #[error("No file extension")]
    NoFileExtension,
}

#[derive(Error, Debug)]
pub enum CompileError {
    #[error("Nasm failed t: {0}")]
    NasmInvokeError(std::io::Error),
    #[error("Nasm compile error")]
    NasmCompileError,
    #[error("Nasm invoke error: {0}")]
    LdInvokeError(std::io::Error),
    #[error("Ld linker error")]
    LdLinkError,
    #[error("Unexpected token: {0}")]
    UnexpectedToken(String),
}

#[derive(Error, Debug)]
pub enum ParseError {
    #[error("Failed to parse program")]
    Incomplete,
    #[error("Unknown operator")]
    UnknownOperator,
    #[error("Unknown keyword")]
    UnknownKeyword,
    #[error("Unexpected token: {0}")]
    UnexpectedToken(String),
}

#[derive(Error, Debug)]
pub enum PreprocessorError {
    #[error("Unexpected keyword")]
    UnexpectedKeyword,
    #[error("Invalid include {0}")]
    InvalidInclude(String),
    #[error("Invalid filename {0}")]
    InvalidFilename(String),
    #[error("File not found {0}")]
    IncludeNotFound(String),
    #[error("Encountered recursive macro")]
    TooManyMacroExpansions,
    #[error("Recursive include")]
    RecursiveInclude,
}

#[derive(Error, Debug)]
pub enum RuntimeError {
    #[error("IO Error")]
    IOError,
    #[error("Stack underflow")]
    StackUnderflow,
    #[error("String capacity exceeded")]
    StringCapacityExceeded,
    #[error("Invalid memory access")]
    InvalidMemoryAccess,
    #[error("Macro not expanded")]
    MacroNotExpanded,
    #[error("Name not resolved")]
    NameNotResolved,
}
