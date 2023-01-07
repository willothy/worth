use std::{fmt::Display, path::PathBuf};

use clap::{Parser, ValueEnum};

#[derive(Debug, Parser)]
pub struct Cli {
    pub file: PathBuf,
    #[clap(subcommand)]
    pub command: Commands,
}

#[derive(Debug, Parser)]
pub enum Commands {
    #[clap(alias = "C")]
    Compile(CompilerOptions),
    #[clap(alias = "S")]
    Simulate(SimulatorOptions),
}

#[derive(Debug, Parser)]
pub struct CompilerOptions {
    #[clap(short, long)]
    pub output: Option<PathBuf>,
    #[clap(short = 'k', long)]
    pub keep_asm: bool,
    #[clap(short = 'K', long)]
    pub keep_obj: bool,
}

#[derive(Debug, Parser)]
pub struct SimulatorOptions {
    #[clap(short, long)]
    pub debug: bool,
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
