#![feature(trace_macros)]
//trace_macros!(true);

use std::path::PathBuf;

use clap::Parser;

use cli::{Cli, Commands};
use instruction::Program;

mod cli;
mod codegen;
mod instruction;
mod parser;
mod preprocessor;
mod sim;

fn load_program(path: &PathBuf) -> Result<Program, Box<dyn std::error::Error>> {
    let name = path.clone().with_extension("");
    let name = name.file_name().unwrap().to_str().unwrap();
    let Ok(source) = std::fs::read_to_string(path) else {
        return Err(format!("Could not read file {}", path.display()).into());
    };

    let program = parser::parse(source, name)?;
    let program = preprocessor::process(program)?;
    Ok(program)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Cli::parse();

    let program = load_program(&args.file.canonicalize().expect("Could not find file!"))?;

    match args.command {
        Commands::Compile(opt) => codegen::compile(&program, opt)?,
        Commands::Simulate(opt) => sim::simulate(&program, opt)?,
    }

    Ok(())
}
