use std::{collections::HashMap, str::FromStr};

use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::{alphanumeric1, digit1, hex_digit1, multispace0, multispace1},
    error::ParseError,
    sequence::preceded,
    IResult,
};
use nom_locate::LocatedSpan;

use crate::{
    codegen::intrinsics::Intrinsic,
    instruction::{self, Instruction, Keyword, Op, Program, Value},
};

type Span<'a> = LocatedSpan<&'a str, &'a str>;

#[derive(Debug, Clone)]
pub struct Token {
    pub value: String,
    pub location: (String, usize, usize),
    pub ty: TokenType,
}

#[derive(Debug, Clone)]
pub enum TokenType {
    Intrinsic,
    Name,
    Comment,
    Op,
    Keyword,
    Value(Value),
    Syscall(usize),
    Empty,
}

#[derive(Debug, Clone)]
pub enum ValueType {
    Int,
    Char,
    Str,
    Ptr,
}

pub fn parse(source: String, name: &str) -> Result<Program, String> {
    let source = Span::new_extra(source.as_str(), name);
    let tokens = parse_program(source)?;

    Ok(Program {
        name: name.to_string(),
        instructions: tokens
            .iter()
            .map(|t| {
                Ok(match &t.ty {
                    TokenType::Intrinsic => Instruction::Intrinsic(Intrinsic::from_str(&t.value)?),
                    TokenType::Name => Instruction::Name(t.value.clone()),
                    TokenType::Op => Instruction::Op(Op::from_str(&t.value)?),
                    TokenType::Keyword => Instruction::Keyword(Keyword::from_str(&t.value)?),
                    TokenType::Value(v) => Instruction::Push(v.clone()),
                    TokenType::Syscall(n) => Instruction::Syscall(match *n {
                        0 => instruction::SyscallKind::Syscall0,
                        1 => instruction::SyscallKind::Syscall1,
                        2 => instruction::SyscallKind::Syscall2,
                        3 => instruction::SyscallKind::Syscall3,
                        4 => instruction::SyscallKind::Syscall4,
                        5 => instruction::SyscallKind::Syscall5,
                        6 => instruction::SyscallKind::Syscall6,
                        _ => return Err(format!("Syscall number {} is out of range (0-6)", n)),
                    }),
                    TokenType::Comment => unreachable!("Comment should be filtered out"),
                    TokenType::Empty => unreachable!("Empty should be filtered out"),
                })
            })
            .collect::<Result<Vec<_>, String>>()?,
        macros: HashMap::new(),
    })
}

pub fn parse_program<'a>(input: Span<'a>) -> Result<Vec<Token>, String> {
    let mut input = input;
    let mut tokens = Vec::new();
    while let Ok((rem, token)) = alt((
        parse_keyword,
        parse_value,
        parse_op,
        parse_syscalls,
        parse_intrinsic,
        parse_comment,
        parse_empty,
    ))(input)
    {
        tokens.push(token);
        input = rem;
    }

    if !input.fragment().is_empty() {
        return Err(format!(
            "Failed to parse program:\nRemaining fragment: {}",
            input.fragment()
        ));
    }

    tokens = tokens
        .iter()
        .filter(|t| match t.ty {
            TokenType::Empty => false,
            TokenType::Comment => false,
            _ => true,
        })
        .cloned()
        .collect();
    println!("Tokens: {:#?}", tokens);
    Ok(tokens)
}

pub fn parse_syscalls<'a>(input: Span<'a>) -> IResult<Span<'a>, Token> {
    let (input, syscall) = preceded(tag("syscall"), digit1)(input)?;

    let token = Token {
        value: syscall.fragment().to_string(),
        location: (
            input.extra.to_string(),
            input.location_line() as usize,
            input.get_column(),
        ),
        ty: TokenType::Syscall(syscall.fragment().parse::<usize>().unwrap()),
    };
    Ok((input, token))
}

pub fn parse_value<'a>(input: Span<'a>) -> IResult<Span<'a>, Token> {
    let (input, token) = alt((parse_int, parse_hex_int))(input)?;

    Ok((input, token))
}

pub fn parse_int<'a>(input: Span<'a>) -> IResult<Span<'a>, Token> {
    let (input, value) = digit1(input)?;

    let token = Token {
        value: value.fragment().to_string(),
        location: (
            input.extra.to_string(),
            input.location_line() as usize,
            input.get_column(),
        ),
        ty: TokenType::Value(Value::Int(value.fragment().parse::<i64>().unwrap())),
    };
    Ok((input, token))
}

pub fn parse_hex_int<'a>(input: Span<'a>) -> IResult<Span<'a>, Token> {
    let (input, value) = preceded(alt((tag("0x"), tag("0X"))), hex_digit1)(input)?;

    let value_num = i64::from_str_radix(value.fragment(), 16).unwrap();
    let token = Token {
        value: value_num.to_string(),
        location: (
            input.extra.to_string(),
            input.location_line() as usize,
            input.get_column(),
        ),
        ty: TokenType::Value(Value::Int(value_num)),
    };

    Ok((input, token))
}

pub fn parse_empty<'a>(input: Span<'a>) -> IResult<Span<'a>, Token> {
    let (input, _) = multispace1(input)?;
    Ok((
        input,
        Token {
            value: "".to_string(),
            location: (
                input.extra.to_string(),
                input.location_line() as usize,
                input.get_column(),
            ),
            ty: TokenType::Empty,
        },
    ))
}

pub fn parse_intrinsic<'a>(input: Span<'a>) -> IResult<Span<'a>, Token> {
    let (input, instruction) = alphanumeric1(input)?;

    let token = Token {
        value: instruction.fragment().to_string(),
        location: (
            input.extra.to_string(),
            input.location_line() as usize,
            input.get_column(),
        ),
        ty: TokenType::Intrinsic,
    };
    Ok((input, token))
}

pub fn parse_keyword<'a>(input: Span<'a>) -> IResult<Span<'a>, Token> {
    let (input, keyword) = alt((
        tag("while"),
        tag("do"),
        tag("if"),
        tag("else"),
        tag("macro"),
        tag("end"),
    ))(input)?;
    Ok((
        input,
        Token {
            value: keyword.fragment().to_string(),
            location: (
                input.extra.to_string(),
                input.location_line() as usize,
                input.get_column(),
            ),
            ty: TokenType::Keyword,
        },
    ))
}

fn ops1<'a>(input: Span<'a>) -> IResult<Span<'a>, Span<'a>> {
    let (input, instruction) = alt((
        tag("+"),
        tag("-"),
        tag("*"),
        tag("/"),
        tag("%"),
        tag("divmod"),
        tag("&"),
        tag("band"),
        tag("|"),
        tag("bor"),
        tag("^"),
        tag("bxor"),
        tag("~"),
        tag("<<"),
        tag("shl"),
        tag(">>"),
        tag("shr"),
    ))(input)?;

    Ok((input, instruction))
}

fn ops2<'a>(input: Span<'a>) -> IResult<Span<'a>, Span<'a>> {
    let (input, instruction) = alt((
        tag("="),
        tag("!="),
        tag("<"),
        tag(">"),
        tag("<="),
        tag(">="),
        tag("."),
        tag(","),
    ))(input)?;
    Ok((input, instruction))
}

pub fn parse_op<'a>(input: Span<'a>) -> IResult<Span<'a>, Token> {
    let (input, instruction) = alt((ops1, ops2))(input)?;
    Ok((
        input,
        Token {
            value: instruction.fragment().to_string(),
            location: (
                input.extra.to_string(),
                input.location_line() as usize,
                input.get_column(),
            ),
            ty: TokenType::Op,
        },
    ))
}

pub fn parse_comment<'a>(input: Span<'a>) -> IResult<Span<'a>, Token> {
    let (input, _) = nom::bytes::complete::tag("//")(input)?;
    let (input, _) = multispace0(input)?;
    let (input, _) = nom::bytes::complete::take_while(|c: char| c != '\n')(input)?;
    Ok((
        input,
        Token {
            value: "".to_string(),
            location: (
                input.extra.to_string(),
                input.location_line() as usize,
                input.get_column(),
            ),
            ty: TokenType::Comment,
        },
    ))
}

/* impl From<&str> for Instruction {
    fn from(value: &str) -> Self {
        if let Ok(val) = value.parse::<i64>() {
            return Instruction::Push(Value::Int(val));
        }
        match value {
            "+" => Instruction::Op(Op::Add),
            "-" => Instruction::Op(Op::Sub),
            "*" => Instruction::Op(Op::Mul),
            "/" => Instruction::Op(Op::Div),
            "%" | "divmod" => Instruction::Op(Op::Mod),
            "&" | "band" => Instruction::Op(Op::BitwiseAnd),
            "|" | "bor" => Instruction::Op(Op::BitwiseOr),
            "^" | "bxor" => Instruction::Op(Op::BitwiseXor),
            "~" => Instruction::Op(Op::BitwiseNot),
            "<<" | "shl" => Instruction::Op(Op::Shl),
            ">>" | "shr" => Instruction::Op(Op::Shr),
            "=" => Instruction::Op(Op::Eq),
            "!=" => Instruction::Op(Op::Neq),
            "<" => Instruction::Op(Op::Lt),
            ">" => Instruction::Op(Op::Gt),
            "<=" => Instruction::Op(Op::Lte),
            ">=" => Instruction::Op(Op::Gte),
            "." => Instruction::Op(Op::Store),
            "," => Instruction::Op(Op::Load),
            "while" => Instruction::Keyword(Keyword::While {
                self_ip: 0,
                do_ip: 0,
            }),
            "do" => Instruction::Keyword(Keyword::Do { end_ip: 0 }),
            "if" => Instruction::Keyword(Keyword::If { else_ip: 0 }),
            "else" => Instruction::Keyword(Keyword::Else {
                else_ip: 0,
                end_ip: 0,
            }),
            "end" => Instruction::Keyword(Keyword::End {
                self_ip: 0,
                while_ip: None,
            }),
            "macro" => Instruction::Keyword(Keyword::Macro),
            "syscall0" => Instruction::Syscall(SyscallKind::Syscall0),
            "syscall1" => Instruction::Syscall(SyscallKind::Syscall1),
            "syscall2" => Instruction::Syscall(SyscallKind::Syscall2),
            "syscall3" => Instruction::Syscall(SyscallKind::Syscall3),
            "syscall4" => Instruction::Syscall(SyscallKind::Syscall4),
            "syscall5" => Instruction::Syscall(SyscallKind::Syscall5),
            "syscall6" => Instruction::Syscall(SyscallKind::Syscall6),
            name => {
                if let Ok(intrinsic) = Intrinsic::from_str(name) {
                    Instruction::Intrinsic(intrinsic)
                } else {
                    Instruction::Name(name.to_owned())
                }
            }
        }
    }
} */
