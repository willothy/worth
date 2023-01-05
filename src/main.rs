use std::{fmt::Display, path::PathBuf};

use clap::{Parser, ValueEnum};
use nom::{
    bytes::complete::take_while, character::complete::multispace0, combinator::eof,
    multi::many_till, IResult,
};
use nom_locate::LocatedSpan;

use self::instruction::{Instruction, Program, Token};

type Span<'a> = LocatedSpan<&'a str>;

mod codegen;
mod instruction;
mod sim;

#[allow(unused)]
#[derive(Debug)]
pub struct Macro {
    name: String,
    body: Vec<instruction::Instruction>,
}

#[derive(Debug, Parser)]
struct Cli {
    file: PathBuf,
    #[clap(subcommand)]
    command: Commands,
}

#[derive(Debug, Parser)]
enum Commands {
    #[clap(alias = "C")]
    Compile(CompilerOptions),
    #[clap(alias = "S")]
    Simulate,
}

#[derive(Debug, Parser)]
pub struct CompilerOptions {
    #[clap(short, long)]
    output: Option<PathBuf>,
}

#[derive(Debug, Parser, Clone, ValueEnum)]
pub enum OutputType {
    Asm,
    Obj,
    Exe,
}

impl Display for OutputType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OutputType::Asm => write!(f, "asm"),
            OutputType::Obj => write!(f, "obj"),
            OutputType::Exe => write!(f, "exe"),
        }
    }
}

fn parse_program<'a>(input: Span<'a>) -> IResult<Span<'a>, Vec<Token>> {
    let (input, (instructions, _)) = many_till(parse_instruction, eof)(input)?;
    Ok((input, instructions))
}

fn parse_instruction<'a>(input: Span<'a>) -> IResult<Span<'a>, Token> {
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

fn resolve_jumps(program: &mut Program) {
    let mut jump_stack = Vec::new();
    for (ip, instruction) in program.instructions.iter_mut().enumerate() {
        match instruction {
            Instruction::If { else_ip } => {
                jump_stack.push(("if", else_ip));
            }
            Instruction::Else { else_ip, end_ip } => {
                let (kind, precursor_else_ip) = jump_stack.pop().unwrap();
                assert_eq!(kind, "if");
                *precursor_else_ip = ip;
                *else_ip = ip;
                jump_stack.push(("else", end_ip));
            }
            Instruction::End { end_ip } => {
                let (kind, precursor_end_ip) = jump_stack.pop().unwrap();
                assert!(kind == "if" || kind == "else");
                *precursor_end_ip = ip;
                *end_ip = ip;
            }
            _ => {}
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Cli::parse();

    let prog_name = args.file.clone().with_extension("");
    let prog_name = prog_name.file_name().unwrap().to_str().unwrap();
    let source = std::fs::read_to_string(args.file)?;
    let source = Span::new(&source);

    let tokens = match parse_program(source) {
        Ok((_, tokens)) => tokens,
        Err(e) => {
            println!("Failed to parse program: {:?}", e);
            return Ok(());
        }
    };

    let mut program = Program {
        name: prog_name.to_string(),
        instructions: tokens
            .iter()
            .map(|t| Instruction::from(t.value.as_str()))
            .collect(),
    };
    resolve_jumps(&mut program);

    match args.command {
        Commands::Compile(opt) => codegen::compile(&program, opt)?,
        Commands::Simulate => sim::simulate(&program)?,
    }

    Ok(())
}
