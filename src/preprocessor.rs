use std::collections::HashMap;

use crate::instruction::{Instruction, Keyword, Macro, Program};

pub fn process(mut program: Program) -> Result<Program, String> {
    expand_macros(&mut program);
    resolve_jumps(&mut program);
    Ok(program)
}

fn expand_macros(program: &mut Program) {
    let mut macros = HashMap::new();
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
                            macros.insert(
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

    // Expand macros
    let mut new_instructions = Vec::new();
    macro_stack.clear();
    in_macro = false;
    for inst in program.instructions.iter() {
        match inst {
            Instruction::Keyword(Keyword::Macro) => {
                macro_stack.push(("macro", 0));
                in_macro = true;
                continue;
            }
            Instruction::Name(name) => {
                if !in_macro {
                    if let Some(macro_) = macros.get(name) {
                        new_instructions.extend(macro_.body.clone());
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
                assert!(pred == "if", "Else without if");
                macro_stack.push(("else", 0));
            }
            Instruction::Keyword(Keyword::End { .. }) => {
                let (kind, _) = macro_stack.pop().unwrap();
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
}

fn resolve_jumps(program: &mut Program) {
    let mut jump_stack = Vec::new();
    for (ip, instruction) in program.instructions.iter_mut().enumerate() {
        match instruction {
            Instruction::Keyword(Keyword::If { else_ip }) => {
                jump_stack.push(("if", 0, else_ip));
            }
            Instruction::Keyword(Keyword::Else { else_ip, end_ip }) => {
                let (kind, _, precursor_else_ip) = jump_stack.pop().unwrap();
                assert_eq!(kind, "if");
                *precursor_else_ip = ip;
                *else_ip = ip;
                jump_stack.push(("else", 0, end_ip));
            }
            Instruction::Keyword(Keyword::End {
                self_ip,
                while_ip: end_ip,
            }) => {
                let (kind, while_ip, precursor_end_ip) = jump_stack.pop().unwrap();
                assert!(kind == "if" || kind == "else" || kind == "do");
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
                assert_eq!(kind, "while");
                *precursor_do_ip = ip;
                jump_stack.push(("do", while_ip, end_ip));
            }
            _ => {}
        }
    }
}
