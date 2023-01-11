use crate::error::IOError::*;
use crate::instruction::Program;
use crate::preprocessor;
use crate::{error::Error::IOError, parser};
use anyhow::{Context, Result};
use std::path::PathBuf;

pub fn load_program(path: &PathBuf) -> Result<Program> {
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
