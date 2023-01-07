use std::path::PathBuf;

use clap::Parser;

use cli::{Cli, Commands};
use instruction::Program;

mod cli;
mod codegen;
mod error;
mod instruction;
mod parser;
mod preprocessor;
mod sim;

use anyhow::{anyhow, Context, Result};
use error::Error::IOError;
use error::IOError::*;

fn load_program(path: &PathBuf) -> Result<Program> {
    let name = path.clone().with_extension("");
    let name = name
        .file_name()
        .ok_or(IOError(FileLoadError))
        .with_context(|| format!("Path {:?} does not have a filename", path))?
        .to_str()
        .ok_or(IOError(FileLoadError))
        .with_context(|| format!("Path {:?} does not have a valid filename", path))?;

    let source = std::fs::read_to_string(path).map_err(|e| IOError(Inherited(e)))?;

    let program = parser::parse(source, name)?;
    let program = preprocessor::process(program)?;
    Ok(program)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Cli::parse();

    let program = load_program(&args.file.canonicalize().expect("Could not find file!"))?;

    let res = match args.command {
        Commands::Compile(opt) => codegen::compile(&program, opt)?,
        Commands::Simulate(opt) => sim::simulate(&program, opt)?,
    };

    println!("{:?}", res);

    Ok(())
}
