use std::{collections::HashMap, mem::ManuallyDrop};
use thiserror::Error;

use crate::{
    instruction::{Instruction, InstructionKind, Value},
    parser::{Token, TokenType},
};

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
    #[error("Unclosed {0} block")]
    UnclosedBlock(String),
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
    pub color: String,
    pub value: String,
    pub postcolor: String,
    pub postfix: String,
    pub loc: &'a (String, usize, usize),
    pub kind: FmtTokenKind,
    pub indent_level: usize,
}

pub enum FmtTokenKind {
    Token(TokenType),
    Instruction(InstructionKind),
}

pub trait RenderFmt {
    fn render(&self, start_line: usize, line_numbers: bool, err: bool) -> String;
    fn format(&mut self) -> &mut Self;
}

impl<'a> RenderFmt for Vec<FmtToken<'a>> {
    fn render(&self, start_line: usize, line_numbers: bool, err: bool) -> String {
        let mut curr_line_no = 0;
        let mut lines = Vec::new();
        let mut line = String::new();

        for inst in self {
            if inst.loc.1 != curr_line_no {
                lines.push(line.trim_end_matches(' ').to_owned());
                line = String::new();
                if if inst.loc.1.to_string().len() >= curr_line_no {
                    inst.loc.1.to_string().len() - curr_line_no
                } else {
                    0
                } > 1
                    && lines.len() > 0
                    && line_numbers
                {
                    let len = {
                        if inst.loc.1.to_string().len() > 0 {
                            inst.loc.1.to_string().len() - 1
                        } else {
                            0
                        }
                    };
                    lines.push(format!("{:.>len$}↓| ...", ""))
                }
                curr_line_no = inst.loc.1;
            }
            if line.is_empty() {
                if line_numbers {
                    let len = curr_line_no.to_string().len();
                    line += &format!("{:.<len$}| ", curr_line_no);
                }
            }
            if err {
                line.push_str(&inst.prefix.replace("\n", ""));
            } else {
                line.push_str(&inst.prefix.replacen("\n", "", 1));
            }
            if line.trim_matches('\n').len() == 0
                && !matches!(
                    inst.kind,
                    FmtTokenKind::Instruction(InstructionKind::Keyword(_))
                        | FmtTokenKind::Token(TokenType::Keyword)
                )
            {
                line.push_str(&" ".repeat(inst.indent_level * 4));
            }
            line.push_str(&inst.color);
            line.push_str(&inst.value);
            line.push_str(&inst.postcolor);
            if err {
                line.push_str(&inst.postfix.replace("\n", ""));
            } else {
                line.push_str(&inst.postfix.replacen("\n", "", 1));
            }
        }
        lines.push(line.trim_end_matches(' ').to_owned());
        lines.join("\n").trim_start().to_owned()
    }

    fn format(&mut self) -> &mut Self {
        let program = self;

        let mut indent = 0;
        let mut prev_newline = false;

        let mut ip = 0;
        let program_len = program.len();
        while ip < program_len {
            let mut tok = &mut program[ip];
            let curr_prev_newline = prev_newline;
            let curr_indent = indent;

            use FmtTokenKind::*;
            match &tok.kind {
                Token(TokenType::Keyword) | Instruction(InstructionKind::Keyword(_)) => {
                    match tok.value.as_str() {
                        "while" => {
                            if !curr_prev_newline {
                                tok.prefix = "\n\n".to_owned();
                            }
                            tok.postfix = " ".to_owned();
                            indent += 1;
                            prev_newline = false;
                        }
                        "if" => {
                            if !curr_prev_newline {
                                tok.prefix = "\n".to_owned();
                            }
                            tok.prefix.push_str(&" ".repeat(curr_indent * 4));
                            tok.postfix = " ".to_owned();
                            indent += 1;
                            prev_newline = false;
                        }
                        "else" => {
                            if !curr_prev_newline {
                                tok.prefix = "\n".to_owned();
                                tok.prefix.push_str(&" ".repeat((curr_indent - 1) * 4));
                            } else {
                                tok.prefix.push_str(&" ".repeat((curr_indent - 1) * 4));
                            }
                            tok.postfix = "\n".to_owned();
                            prev_newline = true;
                        }
                        "elif" => {
                            if !curr_prev_newline {
                                tok.prefix = "\n".to_owned();
                                tok.prefix.push_str(&" ".repeat((curr_indent - 1) * 4));
                            } else {
                                tok.prefix.push_str(&" ".repeat((curr_indent - 1) * 4));
                            }
                            tok.postfix = " ".to_owned();
                            prev_newline = false;
                        }
                        "else if" => {
                            if !curr_prev_newline {
                                tok.prefix = "\n".to_owned();
                                tok.prefix.push_str(&" ".repeat((curr_indent - 1) * 4));
                            }
                            tok.postfix = " ".to_owned();
                            prev_newline = false;
                        }
                        "end" => {
                            if !curr_prev_newline {
                                tok.prefix = "\n".to_owned();
                                tok.prefix.push_str(&" ".repeat((curr_indent - 1) * 4));
                            } else {
                                tok.prefix.push_str(&" ".repeat((curr_indent - 1) * 4));
                            }
                            tok.postfix = "\n".to_owned();
                            prev_newline = true;
                            if indent > 0 {
                                indent -= 1;
                            }
                            if ip + 1 < program_len {
                                tok = &mut program[ip + 1];
                                if !matches!(tok.kind, FmtTokenKind::Token(TokenType::Keyword)) {
                                    tok = &mut program[ip];
                                    tok.postfix.push_str("\n");
                                }
                            }
                        }
                        "do" => {
                            tok.postfix = "\n".to_owned();
                            prev_newline = true;
                        }
                        "macro" => {
                            tok.prefix = "\n".to_owned();
                            tok.postfix = " ".to_owned();
                            ip += 1;
                            tok = &mut program[ip];
                            if let FmtTokenKind::Token(TokenType::Name) = tok.kind {
                                tok.postfix = "\n".to_owned();
                                prev_newline = true;
                                indent += 1;
                            } else {
                                panic!()
                            }
                        }
                        "include" => {
                            if !curr_prev_newline {
                                tok.prefix = "\n".to_owned();
                                tok.prefix.push_str(&" ".repeat(curr_indent * 4));
                            }
                            tok.postfix = " ".to_owned();
                            ip += 1;
                            tok = &mut program[ip];
                            if let FmtTokenKind::Token(TokenType::Value(Value::Str(_))) = tok.kind {
                                tok.postfix = "\n\n".to_owned();
                                prev_newline = true;
                            } else {
                                panic!()
                            }
                        }
                        igl => {
                            unreachable!("Unexpected keyword {}", igl);
                        }
                    }
                }
                _ => {
                    tok.postfix = " ".to_owned();
                    prev_newline = false;
                    if curr_prev_newline {
                        tok.prefix.push_str(&" ".repeat(curr_indent * 4));
                    }
                }
            }
            tok.indent_level = curr_indent;
            ip += 1;
        }

        program
    }
}

pub trait AsFmt<'a> {
    fn as_fmt(&'a self) -> Vec<FmtToken<'a>>;
}

impl<'a> AsFmt<'a> for &'a [Instruction] {
    fn as_fmt(&self) -> Vec<FmtToken<'a>> {
        let mut fmt_tokens = Vec::new();
        for token in self.iter() {
            let token_str = match &token.kind {
                InstructionKind::Push(val) => match val {
                    Value::Str(s) => format!("\"{}\"", s),
                    other => other.to_string(),
                },
                InstructionKind::Intrinsic(i) => i.to_string(),
                InstructionKind::Op(op) => op.to_string(),
                InstructionKind::Keyword(kw) => kw.to_string(),
                InstructionKind::Name(name) => name.to_string(),
                InstructionKind::Syscall(syscall) => syscall.to_string(),
            };

            fmt_tokens.push(FmtToken {
                indent_level: 0,
                prefix: String::new(),
                color: String::new(),
                value: token_str.clone(),
                postcolor: String::new(),
                postfix: String::new(),
                loc: &token.loc,
                kind: FmtTokenKind::Instruction(token.kind.clone()),
            });
        }
        fmt_tokens
    }
}

impl<'a> AsFmt<'a> for Vec<Token> {
    fn as_fmt(&'a self) -> Vec<FmtToken<'a>> {
        let mut fmt_tokens = Vec::new();
        for token in self.iter() {
            let token_str = match &token.ty {
                TokenType::Intrinsic(i) => i.to_string(),
                TokenType::Name => token.value.clone(),
                TokenType::Comment => token.value.clone(),
                TokenType::Op => token.value.clone(),
                TokenType::Keyword => token.value.clone(),
                TokenType::Value(v) => match v {
                    Value::Str(s) => format!("\"{}\"", s),
                    v => v.to_string(),
                },
                TokenType::Syscall(_) => todo!(),
            };

            fmt_tokens.push(FmtToken {
                indent_level: 0,
                prefix: String::new(),
                color: String::new(),
                value: token_str.clone(),
                postcolor: String::new(),
                postfix: String::new(),
                loc: &token.location,
                kind: FmtTokenKind::Token(token.ty.clone()),
            });
        }
        fmt_tokens
    }
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
                    tok.color = "\x1b[33m".to_string();
                }
                Highlight::Error => {
                    tok.color = "\x1b[91m\x1b[1m".to_string();
                }
            }
            tok.postcolor = "\x1b[0m".to_string();
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
    let spread = &program[start..end];

    let first_line = program[start].loc.1 - 1;

    let mut tokens = spread.as_fmt();
    let mut highlights = HashMap::new();
    highlights.insert(ip - start, Highlight::Error);
    if let Some(secondary) = secondary {
        highlights.insert(secondary - start, Highlight::Warning);
    }
    highlight_program(&mut tokens, highlights);
    tokens.format().render(first_line, true, true)
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
