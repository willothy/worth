use std::collections::HashMap;
use std::path::PathBuf;

use crate::error::{Error::PreprocessorError, PreprocessorError::*};
use crate::instruction::{Instruction, Keyword, Macro, Program, Value};
use anyhow::{Context, Result};

pub fn process(mut program: Program) -> Result<Program> {
    includes(&mut program).context(format!(
        "Failed to process includes for {}.porth",
        program.name
    ))?;
    collect_macros(&mut program).context(format!(
        "Failed to process macros for {}.porth",
        program.name
    ))?;
    while expand_macros(&mut program).context(format!(
        "Failed to process macros for {}.porth",
        program.name
    ))? == true
    {}
    jumps(&mut program).context(format!(
        "Failed to process jumps for {}.porth",
        program.name
    ))?;
    Ok(program)
}

fn process_include(program: &mut Program) -> Result<()> {
    includes(program)?;
    Ok(())
}

fn includes(program: &mut Program) -> Result<()> {
    // TODO: Safety method for recursive includes
    // TODO: Search path for includes
    let mut include_paths = Vec::new();
    let mut inst_to_remove = Vec::new();

    let mut instructions = program.instructions.iter().enumerate();

    // Collect includes
    loop {
        let Some((ip, instruction)) = instructions.next() else {
            break;
        };

        match instruction {
            Instruction::Keyword(Keyword::Include) => {
                inst_to_remove.push(ip);

                let Some((ip, include)) = instructions.next() else {
                    break;
                };
                match include {
                    Instruction::Push(Value::Str(path)) => {
                        include_paths.push(PathBuf::from(path));
                        inst_to_remove.push(ip);
                    }
                    other => {
                        return Err(PreprocessorError(InvalidInclude(other.to_string())).into())
                    }
                }
            }
            _ => {}
        }
    }

    // Remove include instructions
    let mut offset = 0;
    for ip in inst_to_remove {
        program.instructions.remove(ip - offset);
        offset += 1;
    }

    // Process includes
    let base_path = program.base_path.clone();
    for include in include_paths {
        let include_path = base_path.join(&include);
        if !include_path.exists() {
            return Err(PreprocessorError(IncludeNotFound(
                include.clone().to_string_lossy().to_string(),
            )))
            .with_context(|| format!("Invalid include {:?}", include));
        }
        let include_path = include_path
            .canonicalize()
            .with_context(|| format!("Failed to canonicalize include path {:?}", include))?;
        let include_file = std::fs::read_to_string(include_path.clone())
            .with_context(|| format!("Failed to read include file {:?}", include_path))?;
        let name = include_path
            .clone()
            .with_extension("")
            .file_name()
            .ok_or(PreprocessorError(
                crate::error::PreprocessorError::InvalidFilename(
                    include_path.clone().to_string_lossy().to_string(),
                ),
            ))?
            .to_string_lossy()
            .to_string();
        let mut include_program = crate::parser::parse(include_file, &name, include_path)?;
        process_include(&mut include_program)?;

        program
            .instructions
            .append(&mut include_program.instructions);
    }
    Ok(())
}

fn collect_macros(program: &mut Program) -> Result<()> {
    let mut macro_body = Vec::new();
    let mut macro_name = String::new();
    let mut macro_stack = Vec::new();
    let mut in_macro = false;

    // Collect macros
    for (ip, instruction) in program.instructions.iter().enumerate() {
        match instruction {
            Instruction::Keyword(Keyword::Macro) => {
                macro_stack.push(("macro", ip));
                in_macro = true;
                continue;
            }
            Instruction::Name(name) => {
                if in_macro && macro_name.is_empty() {
                    macro_name = name.clone();
                    continue;
                }
            }
            Instruction::Keyword(Keyword::End { .. }) => {
                let (kind, start_ip) = macro_stack.pop().unwrap();
                assert!(
                    kind == "macro"
                        || kind == "if"
                        || kind == "else"
                        || kind == "while"
                        || kind == "do"
                );
                match kind {
                    "macro" => {
                        if in_macro {
                            in_macro = false;

                            program.macros.insert(
                                macro_name.clone(),
                                Macro {
                                    name: macro_name.clone(),
                                    body: macro_body.clone(),
                                    loc: (start_ip, ip),
                                    uses: vec![],
                                },
                            );
                            macro_name.clear();
                            macro_body.clear();
                            continue;
                        } else {
                            panic!("Macro end without start");
                        }
                    }
                    "if" => {}
                    "else" => {}
                    "while" => {}
                    "do" => {}
                    _ => {}
                }
            }
            Instruction::Keyword(Keyword::If { .. }) => {
                macro_stack.push(("if", ip));
            }
            Instruction::Keyword(Keyword::Else { .. }) => {
                macro_stack.push(("else", ip));
            }
            Instruction::Keyword(Keyword::While { .. }) => {
                macro_stack.push(("while", ip));
            }
            Instruction::Keyword(Keyword::Do { .. }) => {
                assert!(macro_stack.pop().unwrap().0 == "while");
                macro_stack.push(("do", ip));
            }
            _ => {}
        }
        if in_macro {
            macro_body.push(instruction.clone());
        }
    }
    Ok(())
}

fn expand_macros(program: &mut Program) -> Result<bool> {
    let mut macro_stack = Vec::new();
    let mut has_expanded = false;
    // Expand macros in macros
    /* let mut expanded_macros = HashMap::new();
    for (_, macro_) in &program.macros {
        let mut new_body = Vec::new();
        macro_stack.clear();
        for inst in &macro_.body {
            match inst {
                Instruction::Name(name) => {
                    if let Some(macro_) = program.macros.get(name) {
                        new_body.extend(macro_.body.clone());
                        continue;
                    }
                }
                _ => {}
            }
            new_body.push(inst.clone());
        }
        expanded_macros.insert(
            macro_.name.clone(),
            Macro {
                name: macro_.name.clone(),
                body: new_body,
                loc: macro_.loc,
                uses: vec![],
            },
        );
    }
    program.macros = expanded_macros; */

    // Expand macros
    let mut new_instructions = Vec::new();
    macro_stack.clear();
    let mut in_macro = false;
    for inst in program.instructions.iter() {
        match inst {
            Instruction::Keyword(Keyword::Macro) => {
                macro_stack.push(("macro", 0));
                in_macro = true;
                continue;
            }
            Instruction::Name(name) => {
                if !in_macro {
                    if let Some(macro_) = program.macros.get(name) {
                        new_instructions.extend(macro_.body.clone());
                        has_expanded = true;
                        continue;
                    }
                }
            }
            Instruction::Keyword(Keyword::While { .. }) => {
                macro_stack.push(("while", 0));
            }
            Instruction::Keyword(Keyword::Do { .. }) => {
                assert!(macro_stack.pop().unwrap().0 == "while");
                macro_stack.push(("do", 0));
            }
            Instruction::Keyword(Keyword::If { .. }) => {
                macro_stack.push(("if", 0));
            }
            Instruction::Keyword(Keyword::Else { .. }) => {
                let pred = macro_stack.pop().unwrap().0;
                if pred != "if" {
                    return Err(PreprocessorError(UnexpectedKeyword))
                        .with_context(|| format!("Else without if: found {}", pred));
                }
                macro_stack.push(("else", 0));
            }
            Instruction::Keyword(Keyword::End { .. }) => {
                let (kind, _) = macro_stack.pop().unwrap();

                if ["macro", "if", "else", "while", "do"]
                    .iter()
                    .find(|x| *x == &kind)
                    .is_none()
                {
                    return Err(PreprocessorError(UnexpectedKeyword)).with_context(|| {
                        format!("Unexpected keyword {} in macro expansion", kind)
                    });
                }
                match kind {
                    "macro" => {
                        if in_macro {
                            in_macro = false;
                            continue;
                        }
                    }
                    "if" => {}
                    "else" => {}
                    "while" => {}
                    "do" => {}
                    _ => {}
                }
            }
            _ => {}
        }
        if !in_macro {
            new_instructions.push(inst.clone());
        }
    }
    program.instructions = new_instructions;
    Ok(has_expanded)
}

fn jumps(program: &mut Program) -> Result<()> {
    let mut jump_stack = Vec::new();
    for (ip, instruction) in program.instructions.iter_mut().enumerate() {
        match instruction {
            Instruction::Keyword(Keyword::If { else_ip }) => {
                jump_stack.push(("if", 0, else_ip));
            }
            Instruction::Keyword(Keyword::Else { else_ip, end_ip }) => {
                let (kind, _, precursor_else_ip) = jump_stack.pop().unwrap();
                if kind != "if" {
                    return Err(PreprocessorError(UnexpectedKeyword))
                        .with_context(|| format!("Else without if: found {} instead", kind));
                }
                *precursor_else_ip = ip;
                *else_ip = ip;
                jump_stack.push(("else", 0, end_ip));
            }
            Instruction::Keyword(Keyword::End {
                self_ip,
                while_ip: end_ip,
            }) => {
                let (kind, while_ip, precursor_end_ip) = jump_stack.pop().unwrap();
                if kind != "if" && kind != "else" && kind != "do" {
                    return Err(PreprocessorError(UnexpectedKeyword)).with_context(|| {
                        format!("End without if/else/do: found {} instead", kind)
                    });
                }
                *precursor_end_ip = ip;
                *self_ip = ip;
                if kind == "do" {
                    *end_ip = Some(while_ip);
                }
            }
            Instruction::Keyword(Keyword::While { self_ip, do_ip }) => {
                *self_ip = ip;
                jump_stack.push(("while", ip, do_ip));
            }
            Instruction::Keyword(Keyword::Do { end_ip }) => {
                let (kind, while_ip, precursor_do_ip) = jump_stack.pop().unwrap();
                if kind != "while" {
                    return Err(PreprocessorError(UnexpectedKeyword))
                        .with_context(|| format!("Do without while: found {} instead", kind));
                }
                *precursor_do_ip = ip;
                jump_stack.push(("do", while_ip, end_ip));
            }
            _ => {}
        }
    }
    Ok(())
}
