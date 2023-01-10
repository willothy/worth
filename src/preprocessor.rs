use std::path::PathBuf;

use crate::codegen::intrinsics::Intrinsic;
use crate::err;
use crate::error::kw_str;
use crate::error::{Error::PreprocessorError, PreprocessorError::*};
use crate::instruction::{Instruction, InstructionKind, Keyword, Macro, Program, Value};
use anyhow::{Context, Result};

pub fn process(mut program: Program) -> Result<Program> {
    here(&mut program).context(format!(
        "Failed to process heres for {}.porth",
        program.name
    ))?;
    includes(&mut program, 0).context(format!(
        "Failed to process includes for {}.porth",
        program.name
    ))?;
    collect_macros(&mut program).context(format!(
        "Failed to process macros for {}.porth",
        program.name
    ))?;
    let mut depth = 0;
    while expand_macros(&mut program).context(format!(
        "Failed to process macros for {}.porth",
        program.name
    ))? == true
    {
        if depth >= 100 {
            err!(
                program,
                PreprocessorError(TooManyMacroExpansions),
                "Passed maximum macro recursion depth",
                0
            );
        }
        depth += 1;
    }
    ips(&mut program);
    jumps(&mut program).context(format!(
        "Failed to validate control flow for {}.porth",
        program.name
    ))?;
    Ok(program)
}

fn ips(program: &mut Program) {
    for (ip, instruction) in program.instructions.iter_mut().enumerate() {
        instruction.ip = ip;
    }
}

fn here(program: &mut Program) -> Result<()> {
    for instruction in &mut program.instructions {
        match instruction.kind {
            InstructionKind::Intrinsic(Intrinsic::Here) => {
                let loc = instruction.loc.clone();
                *instruction = Instruction {
                    kind: InstructionKind::Push(Value::Str(
                        loc.0.clone() + ":" + &loc.1.to_string() + ":" + &loc.2.to_string(),
                    )),
                    loc: loc,
                    ip: instruction.ip,
                };
            }
            _ => {}
        }
    }
    Ok(())
}

fn includes(program: &mut Program, depth: usize) -> Result<()> {
    // TODO: Search path for includes
    let mut include_paths = Vec::new();
    let mut inst_to_remove = Vec::new();

    let mut instructions = program.instructions.iter().enumerate();
    if depth > 100 {
        err!(
            program,
            PreprocessorError(RecursiveInclude),
            "Passed maximum include recursion depth",
            0
        );
    }
    // Collect includes
    loop {
        let Some((ip, instruction)) = instructions.next() else {
            break;
        };
        if let InstructionKind::Keyword(Keyword::Include) = instruction.kind {
            inst_to_remove.push(ip);

            let Some((ip, include)) = instructions.next() else {
                break;
            };
            match &include.kind {
                InstructionKind::Push(Value::Str(path)) => {
                    include_paths.push((PathBuf::from(path), ip));
                    inst_to_remove.push(ip);
                }
                other => err!(
                    program,
                    PreprocessorError(InvalidInclude(other.to_string())),
                    format!(
                        "Invalid include: Expected string literal include path, found {}",
                        other
                    ),
                    ip
                ),
            }
        }
    }

    // Process includes
    let base_path = program.base_path.clone();
    for (include, include_ip) in &mut include_paths {
        let include_path = base_path.join(&include);
        if !include_path.exists() {
            err!(
                program,
                PreprocessorError(IncludeNotFound(
                    include.clone().to_string_lossy().to_string(),
                )),
                format!("Invalid include {:?}", include),
                *include_ip
            );
        }
        let Ok(include_path) = include_path
            .canonicalize() else {
                err!(
                    program,
                    PreprocessorError(IncludeNotFound(
                        include.clone().to_string_lossy().to_string(),
                    )),
                    format!("Failed to canonicalize include path {:?}", include),
                    *include_ip
                );
            };
        *include = include_path;
    }

    // Remove include instructions
    let mut offset = 0;
    for ip in inst_to_remove {
        program.instructions.remove(ip - offset);
        offset += 1;
    }

    for (include, include_ip) in &include_paths {
        let include_path = base_path.join(&include);
        let Ok(include_file) = std::fs::read_to_string(include_path.clone()) else {
            err!(
                program,
                PreprocessorError(IncludeNotFound(
                    include.clone().to_string_lossy().to_string(),
                )),
                format!("Failed to read include file {:?}", include),
                *include_ip
            );
        };
        let name = include_path.clone().with_extension("");
        let Some(name) = name.file_name() else {
            err!(
                program,
                PreprocessorError(InvalidFilename(
                    include_path.clone().to_string_lossy().to_string(),
                )),
                format!("Invalid filename for include {:?}", include),
                *include_ip
            )
        };
        let name = name.to_string_lossy().to_string();
        let mut include_program = crate::parser::parse(include_file, &name, include_path.clone())?;
        here(&mut include_program)?;
        includes(&mut include_program, depth + 1)?;
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
        match &instruction.kind {
            InstructionKind::Keyword(Keyword::Macro) => {
                macro_stack.push(("macro", ip));
                in_macro = true;
                continue;
            }
            InstructionKind::Name(name) => {
                if in_macro && macro_name.is_empty() {
                    macro_name = name.clone();
                    continue;
                }
            }
            InstructionKind::Keyword(Keyword::End { .. }) => {
                let (kind, start_ip) = macro_stack.pop().unwrap();
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
                            err!(
                                program,
                                PreprocessorError(UnexpectedMacroEnd),
                                "Unexpected macro end",
                                ip
                            );
                        }
                    }
                    _ => {}
                }
            }
            InstructionKind::Keyword(Keyword::If { .. }) => {
                macro_stack.push(("if", ip));
            }
            InstructionKind::Keyword(Keyword::Elif { .. }) => {
                macro_stack.push(("elif", ip));
            }
            InstructionKind::Keyword(Keyword::Else { .. }) => {
                macro_stack.push(("else", ip));
            }
            InstructionKind::Keyword(Keyword::While { .. }) => {
                macro_stack.push(("while", ip));
            }
            InstructionKind::Keyword(Keyword::Do { .. }) => {
                let _ = macro_stack.pop().unwrap().0;
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

    // Expand macros
    let mut new_instructions = Vec::new();
    macro_stack.clear();
    let mut in_macro = false;
    for instruction in program.instructions.iter() {
        match &instruction.kind {
            InstructionKind::Keyword(Keyword::Macro) => {
                macro_stack.push("macro");
                in_macro = true;
                continue;
            }
            InstructionKind::Name(name) => {
                if !in_macro {
                    if let Some(macro_) = program.macros.get(name) {
                        new_instructions.extend(macro_.body.clone());
                        has_expanded = true;
                        continue;
                    }
                }
            }
            InstructionKind::Keyword(Keyword::While { .. }) => {
                macro_stack.push("while");
            }
            InstructionKind::Keyword(Keyword::Do { .. }) => {
                let _ = macro_stack.pop().unwrap();
                macro_stack.push("do");
            }
            InstructionKind::Keyword(Keyword::If { .. }) => {
                macro_stack.push("if");
            }
            InstructionKind::Keyword(Keyword::Elif { .. }) => {
                let _ = macro_stack.pop().unwrap();
                macro_stack.push("elif");
            }
            InstructionKind::Keyword(Keyword::Else { .. }) => {
                let _ = macro_stack.pop().unwrap();
                macro_stack.push("else");
            }
            InstructionKind::Keyword(Keyword::End { .. }) => {
                let kind = macro_stack.pop().unwrap();

                match kind {
                    "macro" => {
                        if in_macro {
                            in_macro = false;
                            continue;
                        }
                    }
                    _ => {}
                }
            }
            _ => {}
        }
        if !in_macro {
            new_instructions.push(instruction.clone());
        }
    }
    program.instructions = new_instructions;
    Ok(has_expanded)
}

fn jumps(program: &mut Program) -> Result<()> {
    let mut jump_stack: Vec<(
        &str,
        Option<&mut usize>,
        Option<&mut usize>,
        usize,
        Option<usize>,
    )> = Vec::new();

    let mut elifs = Vec::new();

    for (ip, instruction) in program.instructions.iter_mut().enumerate() {
        instruction.ip = ip;
        match &mut instruction.kind {
            InstructionKind::Keyword(Keyword::If) => {
                jump_stack.push(("if", None, None, ip, None));
            }
            InstructionKind::Keyword(Keyword::Elif {
                self_ip,
                end_ip: else_ip,
            }) => {
                let (t, if_do_end_ip, _, last_ip, last_last_ip) = jump_stack.pop().unwrap();
                *self_ip = ip;
                match t {
                    "ifdo" | "elifdo" => {}
                    _ => {
                        err!(
                            program,
                            PreprocessorError(UnexpectedKeyword(format!(
                                "elif following {}",
                                kw_str(t)
                            ))),
                            "Elif can only close if/do and elif/do blocks.",
                            ip,
                            last_last_ip
                        );
                    }
                }
                *if_do_end_ip.unwrap() = *self_ip;
                jump_stack.push(("elif", None, None, ip, Some(last_ip)));
                elifs.push(else_ip);
            }
            InstructionKind::Keyword(Keyword::Else { self_ip, end_ip }) => {
                let (t, if_end_ip, _, last_ip, last_last_ip) = jump_stack.pop().unwrap();
                *self_ip = ip;
                match t {
                    "ifdo" | "elifdo" => {}
                    _ => {
                        err!(
                            program,
                            PreprocessorError(UnexpectedKeyword(format!(
                                "else following {}",
                                kw_str(t)
                            ))),
                            "Else can only close if/do and elif/do blocks.",
                            ip,
                            last_last_ip
                        );
                    }
                }
                *if_end_ip.unwrap() = *self_ip;
                jump_stack.push(("else", Some(end_ip), None, ip, Some(last_ip)));
            }
            InstructionKind::Keyword(Keyword::End {
                self_ip,
                while_ip: return_ip,
            }) => {
                let (t, end_ip, while_ip, _, last_last_ip) = jump_stack.pop().unwrap();
                *self_ip = ip;
                match t {
                    "else" => {
                        *end_ip.unwrap() = *self_ip;
                        for elif in elifs.iter_mut() {
                            **elif = *self_ip;
                        }
                        elifs.clear();
                    }
                    "ifdo" => {
                        *end_ip.unwrap() = *self_ip;
                    }
                    "whiledo" => {
                        *return_ip = while_ip.cloned();
                        *end_ip.unwrap() = *self_ip;
                    }
                    "elifdo" => {
                        *end_ip.unwrap() = *self_ip;
                        for elif in elifs.iter_mut() {
                            **elif = *self_ip;
                        }
                        elifs.clear();
                    }
                    _ => {
                        err!(
                            program,
                            PreprocessorError(UnexpectedKeyword(format!("end following {t}"))),
                            "End can only close if/do, elif/do, else and while/do blocks.",
                            ip,
                            last_last_ip
                        );
                    }
                }
            }
            InstructionKind::Keyword(Keyword::While { self_ip, do_ip: _ }) => {
                *self_ip = ip;
                jump_stack.push(("while", Some(self_ip), None, ip, None));
            }
            InstructionKind::Keyword(Keyword::Do { end_ip }) => {
                let (t, while_ip, _, last_ip, _) = jump_stack.pop().unwrap();
                match t {
                    "if" => {
                        jump_stack.push(("ifdo", Some(end_ip), None, ip, Some(last_ip)));
                    }
                    "elif" => {
                        jump_stack.push(("elifdo", Some(end_ip), None, ip, Some(last_ip)));
                    }
                    "while" => {
                        jump_stack.push(("whiledo", Some(end_ip), while_ip, ip, Some(last_ip)));
                    }
                    t => {
                        err!(
                            program,
                            PreprocessorError(UnexpectedKeyword(format!(
                                "do following {}",
                                kw_str(t)
                            ))),
                            "Do can only follow if, elif and while.",
                            ip,
                            Some(last_ip)
                        );
                    }
                }
            }
            _ => {}
        }
    }
    Ok(())
}
