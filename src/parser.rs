use std::{collections::HashMap, error::Error, str::FromStr};

use nom::{
    bytes::complete::take_while, character::complete::multispace0, combinator::eof,
    multi::many_till, IResult,
};
use nom_locate::LocatedSpan;

use crate::instruction::{Instruction, Intrinsic, Program, Value};

type Span<'a> = LocatedSpan<&'a str>;

#[derive(Debug, Clone)]
pub struct Token {
    pub value: String,
    pub line: usize,
    pub column: usize,
}

pub fn parse(source: String, name: &str) -> Result<Program, Box<dyn Error>> {
    let source = Span::new(&source);

    let tokens = match parse_program(source) {
        Ok((_, tokens)) => tokens,
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

pub fn parse_program<'a>(input: Span<'a>) -> IResult<Span<'a>, Vec<Token>> {
    let (input, (instructions, _)) = many_till(parse_instruction, eof)(input)?;
    Ok((input, instructions))
}

pub fn parse_instruction<'a>(input: Span<'a>) -> IResult<Span<'a>, Token> {
    let (input, _) = multispace0(input)?;
    let (input, instruction) = take_while(|c: char| !c.is_whitespace())(input)?;
    let (input, _) = multispace0(input)?;
    let token = Token {
        value: instruction.fragment().to_string(),
        line: instruction.location_line() as usize,
        column: instruction.get_utf8_column(),
    };
    Ok((input, token))
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
