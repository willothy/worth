use std::{collections::HashMap, fmt::Display, path::PathBuf};

use crate::{
    codegen::intrinsics::Intrinsic,
    error::{Error::ParseError, ParseError::*},
};

use anyhow::{Context, Result};

#[derive(Debug, Clone)]
pub struct Program {
    pub name: String,
    pub base_path: PathBuf,
    pub instructions: Vec<Instruction>,
    pub macros: HashMap<String, Macro>,
}

#[derive(Debug, Clone)]
pub struct Macro {
    pub name: String,
    pub body: Vec<Instruction>,
    pub loc: (usize, usize),
    pub uses: Vec<usize>,
}

#[derive(Debug, Clone)]
pub enum Value {
    Int(i64),
    Str(String),
    Char(u8),
    #[allow(dead_code)]
    Ptr(String), // Label or variable name
}

impl Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Int(value) => write!(f, "{}", value),
            Value::Str(value) => write!(f, "{}", snailquote::escape(value)),
            Value::Char(value) => write!(f, "{}", value),
            Value::Ptr(value) => write!(f, "{}", value),
        }
    }
}

#[derive(Debug, Clone)]
pub enum Op {
    Add,
    Sub,
    Mul,
    Div,
    DivMod,
    BitwiseAnd,
    BitwiseOr,
    BitwiseXor,
    BitwiseNot,
    Shl,
    Shr,
    Eq,
    Neq,
    Lt,
    Gt,
    Lte,
    Gte,
    Store,
    Load,
    Load64,
    Store64,
    Mod,
}

impl Op {
    pub(crate) fn from_str(value: &str) -> Result<Self> {
        match value {
            "+" => Ok(Op::Add),
            "-" => Ok(Op::Sub),
            "*" => Ok(Op::Mul),
            "/" | "div" => Ok(Op::Div),
            "%" | "mod" => Ok(Op::Mod),
            "divmod" => Ok(Op::DivMod),
            "&" | "band" => Ok(Op::BitwiseAnd),
            "|" | "bor" => Ok(Op::BitwiseOr),
            "^" | "bxor" => Ok(Op::BitwiseXor),
            "~" => Ok(Op::BitwiseNot),
            "<<" | "shl" => Ok(Op::Shl),
            ">>" | "shr" => Ok(Op::Shr),
            "=" => Ok(Op::Eq),
            "!=" => Ok(Op::Neq),
            "<" => Ok(Op::Lt),
            ">" => Ok(Op::Gt),
            "<=" => Ok(Op::Lte),
            ">=" => Ok(Op::Gte),
            ".64" => Ok(Op::Store64),
            ",64" => Ok(Op::Load64),
            "." => Ok(Op::Store),
            "," => Ok(Op::Load),
            op => Err(ParseError(UnknownOperator))
                .with_context(|| format!("Unknown operator: {}", op)),
        }
    }
}

impl std::fmt::Display for Op {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Op::Add => write!(f, "+"),
            Op::Sub => write!(f, "-"),
            Op::Mul => write!(f, "*"),
            Op::Div => write!(f, "/"),
            Op::Mod => write!(f, "mod"),
            Op::DivMod => write!(f, "divmod"),
            Op::BitwiseAnd => write!(f, "band"),
            Op::BitwiseOr => write!(f, "bor"),
            Op::BitwiseXor => write!(f, "bxor"),
            Op::BitwiseNot => write!(f, "~"),
            Op::Shl => write!(f, "shl"),
            Op::Shr => write!(f, "shr"),
            Op::Eq => write!(f, "="),
            Op::Neq => write!(f, "!="),
            Op::Lt => write!(f, "<"),
            Op::Gt => write!(f, ">"),
            Op::Lte => write!(f, "<="),
            Op::Gte => write!(f, ">="),
            Op::Store => write!(f, "."),
            Op::Load => write!(f, ","),
            Op::Load64 => write!(f, ",64"),
            Op::Store64 => write!(f, ".64"),
        }
    }
}

#[derive(Debug, Clone)]
pub enum Keyword {
    While {
        self_ip: usize,
        do_ip: usize,
    },
    Do {
        end_ip: usize,
    },
    If {
        else_ip: usize,
    },
    Else {
        else_ip: usize,
        end_ip: usize,
    },
    End {
        self_ip: usize,
        while_ip: Option<usize>,
    },
    Macro,
    Include,
}

impl Keyword {
    pub(crate) fn from_str(value: &str) -> Result<Self> {
        match value {
            "while" => Ok(Keyword::While {
                self_ip: 0,
                do_ip: 0,
            }),
            "do" => Ok(Keyword::Do { end_ip: 0 }),
            "if" => Ok(Keyword::If { else_ip: 0 }),
            "else" => Ok(Keyword::Else {
                else_ip: 0,
                end_ip: 0,
            }),
            "end" => Ok(Keyword::End {
                self_ip: 0,
                while_ip: None,
            }),
            "macro" => Ok(Keyword::Macro),
            "include" => Ok(Keyword::Include),
            kw => {
                Err(ParseError(UnknownKeyword)).with_context(|| format!("Unknown keyword: {}", kw))
            }
        }
    }
}

impl std::fmt::Display for Keyword {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Keyword::While { .. } => write!(f, "while"),
            Keyword::Do { .. } => write!(f, "do"),
            Keyword::If { .. } => write!(f, "if"),
            Keyword::Else { .. } => write!(f, "else"),
            Keyword::End { .. } => write!(f, "end"),
            Keyword::Macro => write!(f, "macro"),
            Keyword::Include => write!(f, "include"),
        }
    }
}

#[derive(Debug, Clone)]
pub enum SyscallKind {
    Syscall0,
    Syscall1,
    Syscall2,
    Syscall3,
    Syscall4,
    Syscall5,
    Syscall6,
}

impl std::fmt::Display for SyscallKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SyscallKind::Syscall0 => write!(f, "syscall0"),
            SyscallKind::Syscall1 => write!(f, "syscall1"),
            SyscallKind::Syscall2 => write!(f, "syscall2"),
            SyscallKind::Syscall3 => write!(f, "syscall3"),
            SyscallKind::Syscall4 => write!(f, "syscall4"),
            SyscallKind::Syscall5 => write!(f, "syscall5"),
            SyscallKind::Syscall6 => write!(f, "syscall6"),
        }
    }
}

#[derive(Debug, Clone)]
pub enum Instruction {
    Push(Value),
    Intrinsic(Intrinsic),
    Op(Op),
    Keyword(Keyword),
    Name(String),
    Syscall(SyscallKind),
}

impl std::fmt::Display for Instruction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Instruction::Push(Value::Int(i)) => write!(f, "{}", i),
            Instruction::Push(Value::Str(s)) => write!(f, "{}", snailquote::escape(s)),
            Instruction::Push(Value::Char(c)) => write!(f, "'{}'", c),
            Instruction::Push(Value::Ptr(s)) => write!(f, "{}", s),
            Instruction::Intrinsic(i) => write!(f, "{}", i),
            Instruction::Op(o) => write!(f, "{}", o),
            Instruction::Keyword(k) => write!(f, "{}", k),
            Instruction::Name(n) => write!(f, "{}", n),
            Instruction::Syscall(s) => write!(f, "{}", s),
        }
    }
}
