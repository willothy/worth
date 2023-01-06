use std::{collections::HashMap, error::Error, str::FromStr};

use nom::{
    bytes::complete::take_while1, character::complete::multispace0, combinator::opt, IResult,
};
use nom_locate::LocatedSpan;

use crate::{
    codegen::intrinsics::Intrinsic,
    instruction::{Instruction, Program, Value},
};

type Span<'a> = LocatedSpan<&'a str>;

#[derive(Debug, Clone)]
pub struct Token {
    pub value: String,
    pub location: (String, usize, usize),
}

pub fn parse(source: String, name: &str) -> Result<Program, Box<dyn Error>> {
    let source = Span::new(&source);
    let parser = Parser {
        file: name.to_string() + ".worth",
    };
    let tokens = match parser.parse_program(source) {
        Ok((remaining, tokens)) => {
            if !remaining.fragment().is_empty() {
                return Err(format!("Failed to parse program: {:?}", remaining).into());
            }
            tokens
        }
        Err(e) => return Err(format!("Failed to parse program: {:?}", e).into()),
    };

    Ok(Program {
        name: name.to_string(),
        instructions: tokens
            .iter()
            .map(|t| Instruction::from(t.value.as_str()))
            .collect(),
        macros: HashMap::new(),
    })
}

struct Parser {
    file: String,
}

impl Parser {
    pub fn parse_program<'a>(&self, input: Span<'a>) -> IResult<Span<'a>, Vec<Token>> {
        let mut instructions = Vec::new();
        let mut input = input;
        while let Ok((remaining, inst)) = self.parse_instruction(input) {
            instructions.push(inst);
            input = remaining;
        }
        Ok((input, instructions))
    }

    pub fn parse_instruction<'a>(&self, input: Span<'a>) -> IResult<Span<'a>, Token> {
        let (input, _) = multispace0(input)?;
        //println!("1: {:?}", input.fragment());
        let (input, _) = opt(Self::comment)(input)?;
        //println!("2: {:?}", input.fragment());
        let (input, instruction) = take_while1(|c: char| !c.is_whitespace())(input)?;
        let (input, _) = multispace0(input)?;
        let token = Token {
            value: instruction.fragment().to_string(),
            location: (self.file.clone(), 0, 0),
        };
        Ok((input, token))
    }

    pub fn comment<'a>(input: Span<'a>) -> IResult<Span<'a>, Span<'a>> {
        let (input, _) = nom::bytes::complete::tag("//")(input)?;
        let (input, comment) = nom::bytes::complete::take_until("\n")(input)?;
        let (input, _) = nom::bytes::complete::tag("\n")(input)?;
        Ok((input, comment))
    }
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
            "." => Instruction::Store,
            "," => Instruction::Load,
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
            "macro" => Instruction::Macro,
            "syscall0" => Instruction::Syscall0,
            "syscall1" => Instruction::Syscall1,
            "syscall2" => Instruction::Syscall2,
            "syscall3" => Instruction::Syscall3,
            "syscall4" => Instruction::Syscall4,
            "syscall5" => Instruction::Syscall5,
            "syscall6" => Instruction::Syscall6,
            name => {
                if let Ok(intrinsic) = Intrinsic::from_str(name) {
                    Instruction::Intrinsic(intrinsic)
                } else {
                    Instruction::Name(name.to_owned())
                }
            }
        }
    }
}
