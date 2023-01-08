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
    #[clap(alias = "B", alias = "b")]
    Build(CompilerOptions),
    #[clap(alias = "R", alias = "r")]
    Run(RunOptions),
    #[clap(alias = "S", alias = "s")]
    Simulate(SimulatorOptions),
}

#[derive(Debug, Parser, Clone)]
pub struct CompilerOptions {
    #[clap(short, long)]
    pub output: Option<PathBuf>,
    #[clap(short = 'k', long)]
    pub keep_asm: bool,
    #[clap(short = 'K', long)]
    pub keep_obj: bool,
    #[clap(short = 'd', long)]
    pub debug: bool,
}

#[derive(Debug, Parser, Clone)]
pub struct RunOptions {
    #[clap(
        short,
        help = "Output file name / type [ types: .asm, .o, .exe ]\nIf file extension is not specified, .exe is assumed."
    )]
    pub output: Option<PathBuf>,
    #[clap(short = 'k', help = "Keep the assembly file after compilation.")]
    pub keep_asm: bool,
    #[clap(short = 'K', help = "Keep the object file after compilation.")]
    pub keep_obj: bool,
    #[clap(short = 'd', help = "Enable debug mode.")]
    pub debug: bool,
    #[clap(
        long_help = "Arguments to pass to the program, use -- to separate them from the compiler arguments.\nExample: ./worthc test.porth run -d -- arg1 arg2."
    )]
    pub run_args: Vec<String>,
}

impl From<RunOptions> for CompilerOptions {
    fn from(opt: RunOptions) -> Self {
        Self {
            output: opt.output,
            keep_asm: opt.keep_asm,
            keep_obj: opt.keep_obj,
            debug: opt.debug,
        }
    }
}

#[derive(Debug, Parser)]
pub struct SimulatorOptions {
    #[clap(short = 'd', long)]
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
