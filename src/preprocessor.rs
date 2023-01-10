use std::cell::RefCell;
use std::path::PathBuf;
use std::rc::Rc;

use crate::codegen::intrinsics::Intrinsic;
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
            return Err(PreprocessorError(TooManyMacroExpansions).into());
        }
        depth += 1;
    }
    jumps(&mut program).context(format!(
        "Failed to process jumps for {}.porth",
        program.name
    ))?;
    Ok(program)
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
                };
            }
            _ => {}
        }
    }
    Ok(())
}

fn includes(program: &mut Program, depth: usize) -> Result<()> {
    // TODO: Safety method for recursive includes
    // TODO: Search path for includes
    let mut include_paths = Vec::new();
    let mut inst_to_remove = Vec::new();

    let mut instructions = program.instructions.iter().enumerate();
    if depth > 100 {
        return Err(PreprocessorError(RecursiveInclude))
            .with_context(|| format!("Invalid include"));
    }
    // Collect includes
    loop {
        let Some((ip, instruction)) = instructions.next() else {
            break;
        };

        match instruction.kind {
            InstructionKind::Keyword(Keyword::Include) => {
                inst_to_remove.push(ip);

                let Some((ip, include)) = instructions.next() else {
                    break;
                };
                match &include.kind {
                    InstructionKind::Push(Value::Str(path)) => {
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
    for include in &mut include_paths {
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
        *include = include_path;
    }

    for include in &include_paths {
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
                assert!(
                    kind == "macro"
                        || kind == "if"
                        || kind == "else"
                        || kind == "elif"
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
                let t = macro_stack.pop().unwrap().0;
                assert!(t == "while" || t == "if" || t == "elif", "Invalid do {}", t);
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
                macro_stack.push(("macro", 0));
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
                macro_stack.push(("while", 0));
            }
            InstructionKind::Keyword(Keyword::Do { .. }) => {
                let t = macro_stack.pop().unwrap().0;
                assert!(t == "while" || t == "if" || t == "elif");
                macro_stack.push(("do", 0));
            }
            InstructionKind::Keyword(Keyword::If { .. }) => {
                macro_stack.push(("if", 0));
            }
            InstructionKind::Keyword(Keyword::Elif { .. }) => {
                let pred = macro_stack.pop().unwrap().0;
                if pred != "do" {
                    return Err(PreprocessorError(UnexpectedKeyword))
                        .with_context(|| format!("Else without if/do: found {}", pred));
                }
                macro_stack.push(("elif", 0));
            }
            InstructionKind::Keyword(Keyword::Else { .. }) => {
                let pred = macro_stack.pop().unwrap().0;
                if pred != "do" {
                    return Err(PreprocessorError(UnexpectedKeyword))
                        .with_context(|| format!("Else without if/do: found {}", pred));
                }
                macro_stack.push(("else", 0));
            }
            InstructionKind::Keyword(Keyword::End { .. }) => {
                let (kind, _) = macro_stack.pop().unwrap();

                if ["macro", "do", "else", "elif"]
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
    let mut jump_stack: Vec<(&str, Option<&mut usize>, Option<&mut usize>)> = Vec::new();

    let mut elifs = Vec::new();

    for (ip, instruction) in program.instructions.iter_mut().enumerate() {
        match &mut instruction.kind {
            InstructionKind::Keyword(Keyword::If) => {
                jump_stack.push(("if", None, None));
            }
            InstructionKind::Keyword(Keyword::Elif {
                self_ip,
                end_ip: else_ip,
            }) => {
                let (t, if_do_end_ip, _) = jump_stack.pop().unwrap();
                *self_ip = ip;
                assert!(t == "ifdo" || t == "elifdo");
                *if_do_end_ip.unwrap() = *self_ip;
                jump_stack.push(("elif", None, None));
                elifs.push(else_ip);
            }
            InstructionKind::Keyword(Keyword::Else { self_ip, end_ip }) => {
                let (t, if_end_ip, _) = jump_stack.pop().unwrap();
                *self_ip = ip;
                assert!(t == "ifdo" || t == "elifdo");
                *if_end_ip.unwrap() = *self_ip;
                jump_stack.push(("else", Some(end_ip), None));
            }
            InstructionKind::Keyword(Keyword::End {
                self_ip,
                while_ip: return_ip,
            }) => {
                let (t, end_ip, while_ip) = jump_stack.pop().unwrap();
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
                    _ => panic!(),
                }
            }
            InstructionKind::Keyword(Keyword::While { self_ip, do_ip: _ }) => {
                *self_ip = ip;
                jump_stack.push(("while", Some(self_ip), None));
            }
            InstructionKind::Keyword(Keyword::Do { end_ip }) => {
                let (t, while_or_elif_ip, elif_end) = jump_stack.pop().unwrap();
                match t {
                    "if" => {
                        jump_stack.push(("ifdo", Some(end_ip), None));
                    }
                    "elif" => {
                        //while_or_elif_ip.unwrap() = end_ip.get();
                        jump_stack.push(("elifdo", Some(end_ip), elif_end));
                    }
                    "while" => {
                        jump_stack.push(("whiledo", Some(end_ip), while_or_elif_ip));
                    }
                    _ => panic!(),
                }
            }
            _ => {}
        }
    }
    Ok(())
}
