use std::collections::HashMap;

use crate::codegen::intrinsics::Intrinsic;

#[derive(Debug, Clone)]
pub struct Program {
    pub name: String,
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

#[derive(Debug, Clone)]
pub enum Op {
    Add,
    Sub,
    Mul,
    Div,
    Mod,
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
}

impl Op {
    pub(crate) fn from_str(value: &str) -> Result<Self, String> {
        match value {
            "+" => Ok(Op::Add),
            "-" => Ok(Op::Sub),
            "*" => Ok(Op::Mul),
            "/" => Ok(Op::Div),
            "%" | "divmod" => Ok(Op::Mod),
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
            "." => Ok(Op::Store),
            "," => Ok(Op::Load),
            op => Err(format!("Unknown operator {}", op)),
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
}

impl Keyword {
    pub(crate) fn from_str(value: &str) -> Result<Self, String> {
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
            _ => Err(format!("Unknown keyword: {}", value)),
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

#[derive(Debug, Clone)]
pub enum Instruction {
    Push(Value),
    Intrinsic(Intrinsic),
    Op(Op),
    Keyword(Keyword),
    Name(String),
    Syscall(SyscallKind),
}
