use std::{collections::HashMap, path::PathBuf};

use crate::{
    codegen::intrinsics::Intrinsic,
    error::{
        Error::{IOError, ParseError},
        IOError::InvalidPath,
        ParseError::*,
    },
    instruction::{self, Instruction, InstructionKind, Keyword, Op, Program, Value},
};
use anyhow::{anyhow, Context, Result};
use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::{char, digit1, hex_digit1, multispace0, multispace1, satisfy},
    combinator::{eof, opt},
    multi::{many0, many1},
    sequence::{delimited, preceded, tuple},
    FindSubstring, FindToken, IResult,
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
    let fname = name.to_string() + ".porth";
    let source = Span::new_extra(source.as_str(), &fname);
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
            .filter(|t| match t.ty {
                TokenType::Comment => false,
                _ => true,
            })
            .map(|t| {
                let ty = match &t.ty {
                    TokenType::Intrinsic(i) => InstructionKind::Intrinsic(i.clone()),
                    TokenType::Name => InstructionKind::Name(t.value.clone()),
                    TokenType::Op => InstructionKind::Op(Op::from_str(&t.value)?),
                    TokenType::Keyword => InstructionKind::Keyword(Keyword::from_str(&t.value)?),
                    TokenType::Value(v) => InstructionKind::Push(v.clone()),
                    TokenType::Syscall(n) => InstructionKind::Syscall(match *n {
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
                };
                let inst = Instruction {
                    kind: ty,
                    loc: t.location.clone(),
                    ip: 0,
                };
                Ok(inst)
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
            parse_value,
            parse_op,
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

    Ok(tokens)
}

pub fn parse_syscalls<'a>(base_input: Span<'a>) -> IResult<Span<'a>, Token> {
    let (input, syscall) = preceded(tag("syscall"), digit1)(base_input)?;

    let loc = (
        base_input.extra.to_string(),
        base_input.location_line() as usize,
        base_input
            .get_line_beginning()
            .find_substring(syscall.fragment().as_bytes())
            .unwrap(),
    );

    let token = Token {
        value: "syscall".to_owned() + syscall.fragment(),
        location: loc,
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

pub fn parse_bool<'a>(base_input: Span<'a>) -> IResult<Span<'a>, Token> {
    let (input, value) = alt((tag("true"), tag("false")))(base_input)?;
    let bool_value = value.fragment().parse::<bool>().unwrap();

    let loc = (
        base_input.extra.to_string(),
        base_input.location_line() as usize,
        base_input
            .get_line_beginning()
            .find_substring(value.fragment().as_bytes())
            .unwrap(),
    );

    let token = Token {
        value: bool_value.to_string(),
        location: loc,
        ty: TokenType::Value(Value::Bool(bool_value)),
    };
    Ok((input, token))
}

pub fn parse_string<'a>(base_input: Span<'a>) -> IResult<Span<'a>, Token> {
    let (input, value) = delimited(
        char('"'),
        many0(alt((special_char, satisfy(|c| c != '"')))),
        char('"'),
    )(base_input)?;
    let value = value.into_iter().collect::<String>();

    let loc = (
        base_input.extra.to_string(),
        base_input.location_line() as usize,
        base_input
            .get_line_beginning()
            .find_substring(value.as_bytes())
            .unwrap(),
    );

    let token = Token {
        value: value.to_string(),
        location: loc,
        ty: TokenType::Value(Value::Str(value.to_string())),
    };
    Ok((input, token))
}

pub fn parse_char<'a>(base_input: Span<'a>) -> IResult<Span<'a>, Token> {
    let (input, value) = delimited(
        char('\''),
        alt((special_char, satisfy(|c| c != '\'' && c != '\\'))),
        char('\''),
    )(base_input)?;
    let val_str = value.to_string();

    let loc = (
        base_input.extra.to_string(),
        base_input.location_line() as usize,
        base_input
            .get_line_beginning()
            .find_substring(
                format!(
                    "'{}'",
                    match value {
                        '\n' => "\\n",
                        '\t' => "\\t",
                        '\r' => "\\r",
                        '\\' => "\\\\",
                        '\'' => "\\'",
                        '\"' => "\\\"",
                        '\0' => "\\0",
                        _ => val_str.as_str(),
                    }
                )
                .as_bytes(),
            )
            .unwrap(),
    );

    if !value.is_ascii() {
        return Err(nom::Err::Error(nom::error::Error::new(
            input,
            nom::error::ErrorKind::Tag,
        )));
    }

    let token = Token {
        value: value.to_string(),
        location: loc,
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

pub fn parse_int<'a>(base_input: Span<'a>) -> IResult<Span<'a>, Token> {
    let (input, (negative, value)) = tuple((opt(char('-')), digit1))(base_input)?;

    let mut fragment = value.fragment().to_string();

    if negative.is_some() {
        fragment.insert(0, '-');
    }

    let loc = (
        base_input.extra.to_string(),
        base_input.location_line() as usize,
        base_input
            .get_line_beginning()
            .find_substring(fragment.as_bytes())
            .unwrap(),
    );

    let token = Token {
        value: fragment.clone(),
        location: loc,
        ty: TokenType::Value(Value::Int(fragment.parse::<i64>().unwrap())),
    };
    Ok((input, token))
}

pub fn parse_hex_int<'a>(base_input: Span<'a>) -> IResult<Span<'a>, Token> {
    let (input, value) = preceded(alt((tag("0x"), tag("0X"))), hex_digit1)(base_input)?;
    let loc = (
        base_input.extra.to_string(),
        base_input.location_line() as usize,
        base_input
            .get_line_beginning()
            .find_substring(value.fragment().as_bytes())
            .unwrap(),
    );
    let value_num = i64::from_str_radix(value.fragment(), 16).unwrap();
    let token = Token {
        value: value_num.to_string(),
        location: loc,
        ty: TokenType::Value(Value::Int(value_num)),
    };

    Ok((input, token))
}

pub fn parse_intrinsic<'a>(base_input: Span<'a>) -> IResult<Span<'a>, Token> {
    let (input, instruction) = many1(satisfy(|c: char| !c.is_whitespace()))(base_input)?;
    let loc = (
        base_input.extra.to_string(),
        base_input.location_line() as usize,
        base_input
            .get_line_beginning()
            .find_substring(instruction.iter().collect::<String>().as_bytes())
            .unwrap(),
    );
    let fragment: String = instruction.iter().collect();
    let intrinsic = match crate::codegen::intrinsics::Intrinsic::from_str(&fragment) {
        Ok(i) => i,
        Err(_) => {
            return Err(nom::Err::Error(nom::error::Error::new(
                input,
                nom::error::ErrorKind::Tag,
            )));
        }
    };
    let token = Token {
        value: fragment,
        location: loc,
        ty: TokenType::Intrinsic(intrinsic),
    };
    Ok((input, token))
}

pub fn parse_name<'a>(base_input: Span<'a>) -> IResult<Span<'a>, Token> {
    // match any non whitespace character
    let (input, name) = many1(satisfy(|c| !c.is_whitespace()))(base_input)?;
    let loc = (
        base_input.extra.to_string(),
        base_input.location_line() as usize,
        base_input
            .get_line_beginning()
            .find_substring(name.iter().collect::<String>().as_bytes())
            .unwrap(),
    );
    let token = Token {
        value: name.iter().collect(),
        location: loc,
        ty: TokenType::Name,
    };
    Ok((input, token))
}

pub fn parse_keyword<'a>(base_input: Span<'a>) -> IResult<Span<'a>, Token> {
    let (input, keyword) = alt((
        tag("while"),
        tag("else if"),
        tag("do"),
        tag("if"),
        tag("elif"),
        tag("else"),
        tag("macro"),
        tag("end"),
        tag("include"),
    ))(base_input)?;
    let loc = (
        base_input.extra.to_string(),
        base_input.location_line() as usize,
        base_input
            .get_line_beginning()
            .find_substring(keyword.fragment().as_bytes())
            .unwrap(),
    );
    Ok((
        input,
        Token {
            value: keyword.fragment().to_string(),
            location: loc,
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
        tag("div"),
        tag("mod"),
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
        tag(",32"),
        tag(".32"),
        tag(",16"),
        tag(".16"),
        tag("."),
        tag(","),
    ))(input)?;
    Ok((input, instruction))
}

pub fn parse_op<'a>(base_input: Span<'a>) -> IResult<Span<'a>, Token> {
    let (input, op) = alt((ops1, ops2))(base_input)?;
    let loc = (
        base_input.extra.to_string(),
        base_input.location_line() as usize,
        base_input
            .get_line_beginning()
            .find_substring(op.fragment().as_bytes())
            .unwrap(),
    );
    Ok((
        input,
        Token {
            value: op.fragment().to_string(),
            location: loc,
            ty: TokenType::Op,
        },
    ))
}

pub fn parse_comment<'a>(base_input: Span<'a>) -> IResult<Span<'a>, Token> {
    let (input, _) = nom::bytes::complete::tag("//")(base_input)?;
    let (input, spaces) = multispace0(input)?;
    let (input, comment) = nom::bytes::complete::take_while(|c: char| c != '\n')(input)?;
    let loc = (
        base_input.extra.to_string(),
        base_input.location_line() as usize,
        base_input
            .get_line_beginning()
            .find_substring(("//".to_owned() + spaces.fragment() + comment.fragment()).as_bytes())
            .unwrap(),
    );
    Ok((
        input,
        Token {
            value: "".to_string(),
            location: loc,
            ty: TokenType::Comment,
        },
    ))
}
