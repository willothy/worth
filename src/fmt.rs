use std::path::PathBuf;

use anyhow::Context;
use clap::Parser;

#[allow(unused)]
mod cfg;
#[allow(unused)]
mod cli;
#[allow(unused)]
mod codegen;
#[allow(unused)]
mod error;
#[allow(unused)]
mod instruction;
#[allow(unused)]
mod log;
#[allow(unused)]
mod parser;
#[allow(unused)]
mod preprocessor;
#[allow(unused)]
mod program;
#[allow(unused)]
mod runner;
#[allow(unused)]
mod sim;
#[allow(unused)]
mod typecheck;

use error::{Error::IOError, IOError::*, RenderFmt};

#[derive(Parser, Debug)]
pub struct Args {
    files: Vec<PathBuf>,
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    for file in &args.files {
        let path = file
            .canonicalize()
            .with_context(|| format!("Failed to canonicalize path {:?}", file))?;
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
        let formatted = error::fmt_program(&program.instructions[..]).render(0, false, false);
        std::fs::write(file, formatted)?;
    }
    Ok(())
}
