use thiserror::Error;

use crate::instruction::Instruction;

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
pub enum Error {
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
    TypecheckError(TypecheckError),
    #[error("[IO Error] {0}")]
    IOError(IOError),
}

#[derive(Error, Debug)]
pub enum TypecheckError {
    #[error("Stack Underflow")]
    StackUnderflow,
    #[error("Invalid type for operation {0}")]
    InvalidTypeForOp(String),
    #[error("Unhandled items on stack")]
    InvalidStack,
    #[error("Include found in program")]
    IncludeInCode,
    #[error("Macro found in program")]
    MacroInCode,
    #[error("Invalid end encountered")]
    InvalidEnd,
    #[error("Invalid else encountered")]
    InvalidElse,
    #[error("Invalid loop encountered")]
    InvalidLoop,
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
    #[error("Unexpected keyword {0}")]
    UnexpectedKeyword(String),
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
    #[error("Buffer overflow")]
    BufferOverflow,
}

pub fn err_spread(program: &Vec<Instruction>, ip: usize, secondary: Option<usize>) -> String {
    let spread_len = if secondary.is_some() && ip > secondary.unwrap() {
        ip - secondary.unwrap() + 1
    } else {
        6
    };
    let start = if spread_len > ip { 0 } else { ip - spread_len };
    let end = (ip + spread_len).min(program.len());
    let spread = start..end;
    let mut nest_level = 0;
    let mut prev_was_newline = false;
    let output = program[spread.clone()]
        .iter()
        .enumerate()
        .map(|(idx, i)| {
            let mut out = String::new();
            let kind_str = i.kind.to_string();
            let err_location = {
                let len = spread.len();
                if len % 2 == 0 {
                    len / 2 + 1
                } else {
                    len / 2
                }
            };

            if kind_str == "else" || kind_str == "end" {
                out += "\n";
            }

            if prev_was_newline {
                if idx == err_location && nest_level > 0 {
                    for _ in 0..nest_level - 1 {
                        out.insert_str(0, "    ");
                    }
                } else {
                    for _ in 0..nest_level {
                        out.insert_str(0, "    ");
                    }
                }
            }

            out += &if idx == err_location {
                format!("\x1b[31;1m>>> {}\x1b[0m", i.kind.to_string())
            } else if secondary.is_some() && i.ip == secondary.unwrap() {
                format!("\x1b[33m{}\x1b[0m", i.kind.to_string())
            } else {
                i.kind.to_string()
            };

            if kind_str == "while" || kind_str == "if" {
                nest_level += 1;
            } else if kind_str == "end" && nest_level != 0 {
                nest_level -= 1;
            }
            if kind_str == "do" || kind_str == "end" || kind_str == "else" {
                prev_was_newline = true;
                out.push('\n');
            } else {
                prev_was_newline = false;
                out.push(' ');
            }
            out
        })
        .collect::<Vec<_>>();
    output.join("")
}

pub fn err_loc(loc: &(String, usize, usize)) -> String {
    format!("{}:{}:{}", loc.0, loc.1, loc.2)
}
