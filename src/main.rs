use std::path::PathBuf;

use clap::Parser;

use cli::{Cli, Commands};
use instruction::Program;

mod cli;
mod codegen;
mod error;
mod instruction;
mod log;
mod parser;
mod preprocessor;
mod runner;
mod sim;

use anyhow::{Context, Result};
use error::Error::IOError;
use error::IOError::*;

fn load_program(path: &PathBuf) -> Result<Program> {
    let path = path
        .canonicalize()
        .with_context(|| format!("Failed to canonicalize path {:?}", path))?;
    let name = path.clone().with_extension("");
    let name = name
        .file_name()
        .ok_or(IOError(InvalidFilename))
        .with_context(|| format!("Path {:?} does not have a filename", path))?
        .to_str()
        .ok_or(IOError(InvalidFilename))
        .with_context(|| format!("Path {:?} does not have a valid filename", path))?;

    let source = std::fs::read_to_string(&path).map_err(|e| IOError(Inherited(e)))?;

    let program = parser::parse(source, name, path.clone())?;
    let program = preprocessor::process(program)?;
    Ok(program)
}

fn main() -> Result<()> {
    let args = Cli::parse();

    let program =
        load_program(&args.file).with_context(|| format!("Failed to load {:?}.", args.file))?;

    match args.command {
        Commands::Build(opt) => {
            let compiled = codegen::compile(&program, opt)?;
            log::log(log::LogLevel::Info, format!("Built {:?}", compiled), false);
        }
        Commands::Run(opt) => {
            let compiled = codegen::compile(&program, opt.clone().into())?
                .canonicalize()
                .with_context(|| format!("Could not find compiled file for {:?}", &program.name))?;
            runner::run(&compiled, opt)?;
        }
        Commands::Simulate(opt) => sim::simulate(&program, opt)?,
    };

    Ok(())
}
