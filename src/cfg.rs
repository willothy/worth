use std::fmt::Write as _;
use std::io::Write as _;
use std::process::Command;

use crate::instruction::{InstructionKind, Keyword};
use crate::log::*;
use anyhow::{Context, Result};

fn sanitize(str: String) -> String {
    str.replace(" ", "_")
        .replace("\n", "\\n")
        .replace("\r", "\\r")
        .replace("\t", "\\t")
}

pub fn dump(program: &crate::instruction::Program, opt: crate::cli::CfgOptions) -> Result<()> {
    let dot_path = opt.output.unwrap_or_else(|| {
        let mut path = program.base_path.join(&program.name);
        path.set_extension("dot");
        path
    });

    if dot_path.exists() {
        dialoguer::console::set_colors_enabled(true);
        let overwrite = dialoguer::Select::new()
            .item("Yes")
            .item("No")
            .default(1)
            .with_prompt(format!("File {:?} already exists, overwrite?", dot_path))
            .report(true)
            .clear(true)
            .interact()?;
        if overwrite == 0 {
            if dot_path.is_file() {
                std::fs::remove_file(&dot_path)
                    .context(format!("Failed to remove file {:?}", &dot_path))?;
            } else {
                std::fs::remove_dir(&dot_path)
                    .context(format!("Failed to remove directory {:?}", &dot_path))?;
            }
        } else {
            return Err(anyhow::anyhow!(
                "Aborted by user: file {:?} already exists.",
                &dot_path
            ))
            .context(format!("Could not write graphviz."));
        }
    }

    let file_name = dot_path.file_name().unwrap().to_string_lossy().to_string();

    let mut file = std::fs::File::create(&dot_path)
        .context(format!("Failed to create file {:?}", &dot_path))?;

    log(
        LogLevel::Info,
        format!("Generating dotfile for {}.porth", &program.name),
        false,
    );

    let mut graph = String::new();
    writeln!(graph, "digraph {{")?;

    for ip in 0..program.instructions.len() {
        let op = &program.instructions[ip];
        use Keyword::*;
        match &op.kind {
            InstructionKind::Keyword(If) => {
                writeln!(graph, "\tNode{} [shape=record label=if];", ip)?;
                writeln!(graph, "\tNode{} -> Node{}", ip, ip + 1)?;
            }
            _ => {
                writeln!(
                    graph,
                    "\tNode{ip} [label=\"{}\"];",
                    snailquote::escape(&op.kind.to_string())
                )?;
                writeln!(graph, "\tNode{} -> Node{};", ip, ip + 1)?;
            }
        }
    }
    writeln!(graph, "\tNode{} [label=halt]", program.instructions.len())?;
    writeln!(graph, "}}")?;

    file.write(graph.as_bytes())
        .context(format!("Failed to write to file {:?}", &dot_path))?;
    log(LogLevel::Info, format!("Generated {}", &file_name), false);

    log(
        LogLevel::Info,
        format!("Generating graphviz svg for {}", &file_name),
        false,
    );
    let dot = Command::new("dot")
        .arg("-Tsvg")
        .arg("-O")
        .arg(&dot_path)
        .output()
        .context(format!("Failed to render graphviz for {}", &file_name))?;
    if dot.status.success() {
        log(
            LogLevel::Info,
            format!("Generated {}.svg", &file_name),
            false,
        );
    } else {
        log(
            LogLevel::Warn,
            format!(
                "Failed to render graphviz for {:?}: {}",
                &dot_path,
                String::from_utf8_lossy(&dot.stderr)
            ),
            false,
        );
    }

    Ok(())
}
