use std::path::PathBuf;

use anyhow::{Context, Result};

use crate::cli::RunOptions;
use crate::error::{Error::RunnerError, RunnerError::*};
use crate::{log, log::LogLevel};

pub fn run(compiled: &PathBuf, opt: RunOptions) -> Result<()> {
    log::log(
        log::LogLevel::Info,
        format!("Running {:?}", compiled).replace("\"", ""),
        false,
    );
    let mut run_cmd = std::process::Command::new(compiled);
    run_cmd.args(&opt.run_args);
    log::log(
        LogLevel::Cmd,
        format!("{:?}\n", run_cmd).replace("\"", ""),
        false,
    );
    let run = run_cmd
        .spawn()
        .map_err(|e| RunnerError(InvokeError(e)))
        .with_context(|| format!("Failed to spawn run process for {:?}", compiled))?
        .wait_with_output()
        .map_err(|e| RunnerError(InvokeError(e)))
        .with_context(|| format!("Failed to wait for {:?} process to complete", compiled))?;

    if run.status.code().unwrap_or(0) != 0 {
        return Err(RunnerError(NonZeroStatus(run.status.code().unwrap_or(0) as usize)).into());
    }

    // Delete executable
    if let Err(e) = std::fs::remove_file(compiled) {
        log::log(
            LogLevel::Warn,
            format!("Failed to delete executable: {}", e),
            false,
        );
    }
    Ok(())
}
