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
    #[allow(unused)]
    Char(u8),
    #[allow(unused)]
    Ptr(String), // Label or variable name
}

#[derive(Debug, Clone)]
pub enum Instruction {
    Push(Value),
    Intrinsic(Intrinsic),
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
    Macro,
    Name(String),
    Store,
    Load,
    Syscall0,
    Syscall1,
    Syscall2,
    Syscall3,
    Syscall4,
    Syscall5,
    Syscall6,
}
