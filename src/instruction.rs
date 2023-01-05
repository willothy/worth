use std::{fmt::Display, str::FromStr};

use strum_macros::{EnumString, IntoStaticStr};

use crate::intrinsics;

#[derive(Debug, Clone)]
pub struct Program {
    pub name: String,
    pub instructions: Vec<Instruction>,
}

#[derive(Debug, Clone)]
pub struct Token {
    pub value: String,
    pub line: usize,
    pub column: usize,
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
}

impl From<&str> for Instruction {
    fn from(value: &str) -> Self {
        if let Ok(val) = value.parse::<i64>() {
            return Instruction::Push(Value::Int(val));
        }
        match value {
            "+" => Instruction::Add,
            "-" => Instruction::Sub,
            "*" => Instruction::Mul,
            "/" => Instruction::Div,
            "%" => Instruction::Mod,
            "&" => Instruction::And,
            "|" => Instruction::Or,
            "^" => Instruction::Xor,
            "~" => Instruction::Not,
            "<<" => Instruction::Shl,
            ">>" => Instruction::Shr,
            "=" => Instruction::Eq,
            "!=" => Instruction::Neq,
            "<" => Instruction::Lt,
            ">" => Instruction::Gt,
            "<=" => Instruction::Lte,
            ">=" => Instruction::Gte,
            "while" => Instruction::While {
                self_ip: 0,
                do_ip: 0,
            },
            "do" => Instruction::Do { end_ip: 0 },
            "if" => Instruction::If { else_ip: 0 },
            "else" => Instruction::Else {
                else_ip: 0,
                end_ip: 0,
            },
            "end" => Instruction::End {
                self_ip: 0,
                while_ip: None,
            },
            "." => Instruction::Intrinsic(Intrinsic::Dump),
            intrinsic => {
                if let Ok(intrinsic) = Intrinsic::from_str(intrinsic) {
                    Instruction::Intrinsic(intrinsic)
                } else {
                    panic!("Unknown instruction: {}", value)
                }
            }
        }
    }
}
/*
#[derive(Debug, IntoStaticStr, EnumString, Clone)]
#[strum(ascii_case_insensitive)]
pub enum Intrinsic {
    Dump,
    Panic,
    Dup,
} */

intrinsics!(Dump, Panic, Dup);

impl Display for Intrinsic {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let intrinsic: &'static str = self.into();
        write!(f, "{}", intrinsic)
    }
}
