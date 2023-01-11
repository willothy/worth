use clap::Parser;

use cli::{Cli, Command};

mod cfg;
mod cli;
mod codegen;
mod error;
mod instruction;
mod log;
mod parser;
mod preprocessor;
mod program;
mod runner;
mod sim;
mod typecheck;

use anyhow::{Context, Result};

use self::program::load_program;

fn main() -> Result<()> {
    let args = Cli::parse();

    let program =
        load_program(&args.file).with_context(|| format!("Failed to load {:?}.", args.file))?;

    let tc_debugger = if let Some(Command::Simulate(opt)) = &args.command {
        opt.tc_debug
    } else {
        false
    };
    if !args.unsafe_ {
        typecheck::typecheck(&program, tc_debugger)?;
    }

    match args.command {
        Some(Command::Build(opt)) => {
            let compiled = codegen::compile(&program, opt)?;
            log::log(log::LogLevel::Info, format!("Built {:?}", compiled), false);
        }
        Some(Command::Run(opt)) => {
            let compiled = codegen::compile(&program, opt.clone().into())?
                .canonicalize()
                .with_context(|| format!("Could not find compiled file for {:?}", &program.name))?;
            runner::run(&compiled, opt)?;
        }
        Some(Command::Simulate(opt)) => sim::simulate(&program, opt)?,
        Some(Command::Cfg(opt)) => {
            cfg::dump(&program, opt)?;
        }
        None => {
            todo!("Implement repl")
        }
    };

    Ok(())
}
