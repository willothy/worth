use std::collections::HashMap;
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
    #[error("Unexpected macro end")]
    UnexpectedMacroEnd,
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

pub struct FmtToken<'a> {
    pub prefix: String,
    pub value: String,
    pub postfix: String,
    pub loc: &'a (String, usize, usize),
}

pub trait RenderFmt {
    fn render(&self, start_line: usize, line_numbers: bool, trim_empty: bool) -> String;
}

impl<'a> RenderFmt for Vec<FmtToken<'a>> {
    fn render(&self, start_line: usize, line_numbers: bool, trim_empty: bool) -> String {
        let mut prog = self
            .iter()
            .map(|t| format!("{}{}{}", t.prefix, t.value, t.postfix))
            .collect::<Vec<_>>()
            .join("")
            .lines()
            .enumerate()
            .filter_map(|(idx, line)| {
                let line_num = start_line + idx;

                if idx == 0 && line.trim().is_empty() && trim_empty {
                    None
                } else if idx == self.len() - 1 && line.trim().is_empty() && trim_empty {
                    None
                } else if line_numbers {
                    Some(format!("{:>4} | {}\n", line_num, line.trim_end()))
                } else {
                    Some(format!("{}\n", line.trim_end()))
                }
            })
            .collect::<String>();
        prog.insert_str(0, "\n\n");
        prog
    }
}

pub fn fmt_program<'a>(program: &'a [Instruction]) -> Vec<FmtToken<'a>> {
    let mut nest_level = 0;
    let mut prev_caused_newline = false;
    let mut last_was_include = false;
    let mut line_len = 0;
    program
        .iter()
        .enumerate()
        .map(|(idx, inst)| {
            let mut prefix = String::new();
            let mut postfix = String::from(" ");
            let kind_str = match &inst.kind {
                crate::instruction::InstructionKind::Push(v) => match v {
                    crate::instruction::Value::Str(s) => format!("\"{}\"", s),
                    v => v.to_string(),
                },
                k => k.to_string(),
            };
            let kind_str = kind_str.as_str();

            if prev_caused_newline && kind_str.trim() != "end" {
                for _ in 0..nest_level {
                    prefix.insert_str(0, "    ");
                }
            }

            match kind_str {
                "if" => {
                    prev_caused_newline = false;
                    nest_level += 1;
                }
                "else" => {
                    prev_caused_newline = true;
                }
                "elif" => {
                    prefix.insert_str(0, "\n");
                    prev_caused_newline = false;
                }
                "while" => {
                    prefix.insert_str(0, "\n");
                    prev_caused_newline = false;
                    nest_level += 1;
                }
                "do" => {
                    postfix = "\n".to_owned();
                    prev_caused_newline = true;
                }
                "end" => {
                    prefix.insert_str(0, "\n");
                    postfix = "\n\n".to_owned();
                    if nest_level > 0 {
                        nest_level -= 1;
                    }
                    prev_caused_newline = true;
                }
                _ => {
                    prev_caused_newline = false;
                }
            }

            if !prev_caused_newline {
                line_len += prefix.len() + kind_str.len();
                if line_len > 30 {
                    postfix = postfix.replacen(" ", "\n", 1);
                    line_len = 0;
                    prev_caused_newline = true;
                }
            }

            FmtToken {
                prefix,
                value: kind_str.to_owned(),
                postfix,
                loc: &inst.loc,
            }
        })
        .collect::<Vec<_>>()
}

pub enum Highlight {
    Warning,
    Error,
}

pub fn highlight_program<'a>(
    program: &mut Vec<FmtToken<'a>>,
    highlights: HashMap<usize, Highlight>,
) {
    program.iter_mut().enumerate().for_each(|(ip, tok)| {
        if let Some(highlight) = highlights.get(&ip) {
            match highlight {
                Highlight::Warning => {
                    tok.value = format!("{}{}{}", "\x1b[33m", tok.value, "\x1b[0m");
                }
                Highlight::Error => {
                    tok.value = format!("{}{}{}", "\x1b[91m\x1b[1m>>> ", tok.value, "\x1b[0m");
                }
            }
        }
    });
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

    let first_line = program[start].loc.1 - 1;

    let mut tokens = fmt_program(&program[spread]);
    let mut highlights = HashMap::new();
    highlights.insert(ip - start, Highlight::Error);
    if let Some(secondary) = secondary {
        highlights.insert(secondary - start, Highlight::Warning);
    }
    highlight_program(&mut tokens, highlights);
    tokens.render(first_line, true, true)
}

pub fn err_loc(loc: &(String, usize, usize)) -> String {
    format!("{}:{}:{}", loc.0, loc.1, loc.2)
}

pub fn kw_str(kw: &str) -> &str {
    match kw {
        "whiledo" => "while ... do",
        "ifdo" => "if ... do",
        "elifdo" => "elif ... do",
        kw => kw,
    }
}

#[macro_export]
macro_rules! err {
    ($program:ident, $kind:expr, $msg:expr, $ip:expr) => {
        return Err($kind).with_context(|| {
            use crate::error::{err_loc, err_spread};
            format!(
                "[{}] {}\n{}\n",
                err_loc(&$program.instructions[$ip].loc),
                $msg,
                err_spread(&$program.instructions, $ip, None)
            )
        })
    };
    ($program:ident, $kind:expr, $msg:expr, $ip:expr, $last_ip:expr) => {
        return Err($kind).with_context(|| {
            use crate::error::{err_loc, err_spread};
            format!(
                "[{}] {}\n{}\n",
                err_loc(&$program.instructions[$ip].loc),
                $msg,
                err_spread(&$program.instructions, $ip, $last_ip)
            )
        })
    };
}
