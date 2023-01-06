use std::{collections::HashMap, error::Error, str::FromStr};

use nom::{bytes::complete::take_while1, character::complete::multispace0, IResult};
use nom_locate::LocatedSpan;

use crate::instruction::{Instruction, Intrinsic, Program, Value};

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
        let (input, instruction) = take_while1(|c: char| !c.is_whitespace())(input)?;
        let (input, _) = multispace0(input)?;
        let token = Token {
            value: instruction.fragment().to_string(),
            location: (self.file.clone(), 0, 0),
        };
        Ok((input, token))
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
            "." => Instruction::Intrinsic(Intrinsic::Dump),
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
