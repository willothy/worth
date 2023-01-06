use std::{collections::HashMap, fmt::Display};

use strum_macros::{EnumString, IntoStaticStr};

use crate::intrinsics;

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
    And,
    Or,
    Xor,
    Not,
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
}

intrinsics!(Dump, Panic, Dup, Mem);

impl Display for Intrinsic {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let intrinsic: &'static str = self.into();
        write!(f, "{}", intrinsic)
    }
}
