use std::{collections::HashMap, path::PathBuf};

use crate::{
    codegen::intrinsics::Intrinsic,
    error::{
        Error::{IOError, ParseError},
        IOError::InvalidPath,
        ParseError::*,
    },
    instruction::{self, Instruction, Keyword, Op, Program, Value},
};
use anyhow::{anyhow, Context, Result};
use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::{
        alphanumeric1, char, digit1, hex_digit1, multispace0, multispace1, satisfy,
    },
    combinator::eof,
    multi::{many0, many1},
    sequence::{delimited, preceded},
    IResult,
};
use nom_locate::LocatedSpan;

pub type Span<'a> = LocatedSpan<&'a str, &'a str>;

#[derive(Debug, Clone)]
pub struct Token {
    pub value: String,
    pub location: (String, usize, usize),
    pub ty: TokenType,
}

#[derive(Debug, Clone)]
pub enum TokenType {
    Intrinsic(Intrinsic),
    Name,
    Comment,
    Op,
    Keyword,
    Value(Value),
    Syscall(usize),
}

pub fn parse(source: String, name: &str, path: PathBuf) -> Result<Program> {
    let source = Span::new_extra(source.as_str(), name);
    let tokens = parse_program(source)?;

    Ok(Program {
        name: name.to_string(),
        base_path: path
            .parent()
            .ok_or(IOError(InvalidPath))
            .with_context(|| format!("Could not get parent of {:?}", path))?
            .to_path_buf(),
        instructions: tokens
            .iter()
            .map(|t| {
                Ok(match &t.ty {
                    TokenType::Intrinsic(i) => Instruction::Intrinsic(i.clone()),
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
                        _ => return Err(anyhow!("Syscall number {} is out of range (0-6)", n)),
                    }),
                    TokenType::Comment => {
                        return Err(ParseError(UnexpectedToken("comment".into())))
                            .with_context(|| "Comment should be filtered out")
                    }
                })
            })
            .collect::<Result<Vec<_>>>()?,
        macros: HashMap::new(),
    })
}

pub fn parse_program<'a>(input: Span<'a>) -> Result<Vec<Token>> {
    let mut input = input;
    let mut tokens = Vec::new();
    while let Ok((rem, token)) = delimited(
        multispace0,
        alt((
            parse_comment,
            parse_keyword,
            parse_syscalls,
            parse_intrinsic,
            parse_op,
            parse_value,
            parse_name,
        )),
        alt((multispace1, eof)),
    )(input)
    {
        tokens.push(token);
        input = rem;
    }

    if !input.fragment().is_empty() {
        return Err(ParseError(Incomplete))
            .with_context(|| format!("Remaining input: {}", input.fragment()));
    }

    tokens = tokens
        .iter()
        .filter(|t| match t.ty {
            TokenType::Comment => false,
            _ => true,
        })
        .cloned()
        .collect();
    Ok(tokens)
}

pub fn parse_syscalls<'a>(input: Span<'a>) -> IResult<Span<'a>, Token> {
    let (input, syscall) = preceded(tag("syscall"), digit1)(input)?;

    let token = Token {
        value: "syscall".to_owned() + syscall.fragment(),
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
    let (input, token) = alt((
        parse_int,
        parse_hex_int,
        parse_char,
        parse_string,
        parse_bool,
    ))(input)?;

    Ok((input, token))
}

pub fn parse_bool<'a>(input: Span<'a>) -> IResult<Span<'a>, Token> {
    let (input, value) = alt((tag("true"), tag("false")))(input)?;
    let value = value.fragment().parse::<bool>().unwrap();

    let token = Token {
        value: value.to_string(),
        location: (
            input.extra.to_string(),
            input.location_line() as usize,
            input.get_column(),
        ),
        ty: TokenType::Value(Value::Int(value as i64)),
    };
    Ok((input, token))
}

pub fn parse_string<'a>(input: Span<'a>) -> IResult<Span<'a>, Token> {
    let (input, value) = delimited(
        char('"'),
        many0(alt((special_char, satisfy(|c| c != '"')))),
        char('"'),
    )(input)?;
    let value = value.into_iter().collect::<String>();

    let token = Token {
        value: value.to_string(),
        location: (
            input.extra.to_string(),
            input.location_line() as usize,
            input.get_column(),
        ),
        ty: TokenType::Value(Value::Str(value.to_string())),
    };
    Ok((input, token))
}

pub fn parse_char<'a>(input: Span<'a>) -> IResult<Span<'a>, Token> {
    let (input, value) = delimited(
        char('\''),
        alt((special_char, satisfy(|c| c != '\'' && c != '\\'))),
        char('\''),
    )(input)?;

    if !value.is_ascii() {
        return Err(nom::Err::Error(nom::error::Error::new(
            input,
            nom::error::ErrorKind::Tag,
        )));
    }

    let token = Token {
        value: value.to_string(),
        location: (
            input.extra.to_string(),
            input.location_line() as usize,
            input.get_column(),
        ),
        ty: TokenType::Value(Value::Char(value as u8)),
    };
    Ok((input, token))
}

pub fn special_char<'a>(input: Span<'a>) -> IResult<Span<'a>, char> {
    let (input, c) = preceded(
        char('\\'),
        satisfy(|c| {
            c == 'n' || c == 'r' || c == 't' || c == '\\' || c == '\'' || c == '"' || c == '0'
        }),
    )(input)?;
    match c {
        'n' => Ok((input, '\n')),
        'r' => Ok((input, '\r')),
        't' => Ok((input, '\t')),
        '\\' => Ok((input, '\\')),
        '\'' => Ok((input, '\'')),
        '"' => Ok((input, '"')),
        '0' => Ok((input, '\0')),
        _ => Err(nom::Err::Error(nom::error::Error::new(
            input,
            nom::error::ErrorKind::Tag,
        ))),
    }
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

pub fn parse_intrinsic<'a>(input: Span<'a>) -> IResult<Span<'a>, Token> {
    let (input, instruction) = alphanumeric1(input)?;

    let intrinsic = match crate::codegen::intrinsics::Intrinsic::from_str(instruction.fragment()) {
        Ok(i) => i,
        Err(_) => {
            return Err(nom::Err::Error(nom::error::Error::new(
                input,
                nom::error::ErrorKind::Tag,
            )));
        }
    };
    let token = Token {
        value: instruction.fragment().to_string(),
        location: (
            input.extra.to_string(),
            input.location_line() as usize,
            input.get_column(),
        ),
        ty: TokenType::Intrinsic(intrinsic),
    };
    Ok((input, token))
}

pub fn parse_name<'a>(input: Span<'a>) -> IResult<Span<'a>, Token> {
    let extra = input.extra.to_owned();
    let line = input.location_line();
    let col = input.get_column();
    /* let (input, (start, name_rest)) = tuple((
        alt((satisfy(|c| c.is_alphabetic()), char('_'))),
        opt(many1(alt((satisfy(|c| c.is_alphanumeric()), char('_'))))),
    ))(input)?;
    let mut name = vec![start];
    if let Some(rest) = name_rest {
        name.extend(rest);
    } */
    // match any non whitespace character
    let (input, name) = many1(satisfy(|c| !c.is_whitespace()))(input)?;
    let token = Token {
        value: name.iter().collect(),
        location: (extra, line as usize, col - 1),
        ty: TokenType::Name,
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
        tag("include"),
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
        //tag("/"),
        //tag("%"),
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
        tag("!="),
        tag("<="),
        tag(">="),
        tag("<"),
        tag(">"),
        tag("="),
        tag(",64"),
        tag(".64"),
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
