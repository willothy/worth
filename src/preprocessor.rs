use std::collections::HashMap;

use crate::instruction::{Instruction, Macro, Program};

pub fn process(mut program: Program) -> Result<Program, String> {
    expand_macros(&mut program);
    resolve_jumps(&mut program);
    Ok(program)
}

fn expand_macros(program: &mut Program) {
    let mut macro_stack = Vec::new();
    let mut macros = HashMap::new();
    let mut macro_body = Vec::new();
    let mut macro_name = String::new();
    let mut in_macro = false;

    // Collect macros
    for (ip, instruction) in program.instructions.iter_mut().enumerate() {
        match instruction {
            Instruction::Macro => {
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
            Instruction::End { .. } => {
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
            Instruction::If { .. } => {
                macro_stack.push(("if", ip));
            }
            Instruction::Else { .. } => {
                macro_stack.push(("else", ip));
            }
            Instruction::While { .. } => {
                macro_stack.push(("while", ip));
            }
            Instruction::Do { .. } => {
                assert!(macro_stack.pop().unwrap().0 == "while");
                macro_stack.push(("do", ip));
            }
            _ => {}
        }
        if in_macro {
            macro_body.push(instruction.clone());
        }
    }

    // Remove macro definitions
    for (name, macro_def) in macros {
        println!("Defining macro {} at {:?}", name, macro_def.loc);
        let (start_ip, end_ip) = macro_def.loc;
        program.instructions.drain(start_ip..=end_ip);
        program.macros.insert(name, macro_def);
    }

    // Resolve uses
    for (ip, inst) in program.instructions.iter().enumerate() {
        match inst {
            Instruction::Name(name) => {
                if let Some(macro_def) = program.macros.get_mut(name) {
                    macro_def.uses.push(ip);
                }
            }
            _ => {}
        }
    }

    // Expand macro uses
    for (_, macro_def) in &program.macros {
        for use_ip in &macro_def.uses {
            let use_ip = *use_ip;
            program
                .instructions
                .splice(use_ip..=use_ip, macro_def.body.clone());
        }
    }
}

fn resolve_jumps(program: &mut Program) {
    let mut jump_stack = Vec::new();
    for (ip, instruction) in program.instructions.iter_mut().enumerate() {
        match instruction {
            Instruction::If { else_ip } => {
                jump_stack.push(("if", 0, else_ip));
            }
            Instruction::Else { else_ip, end_ip } => {
                let (kind, _, precursor_else_ip) = jump_stack.pop().unwrap();
                assert_eq!(kind, "if");
                *precursor_else_ip = ip;
                *else_ip = ip;
                jump_stack.push(("else", 0, end_ip));
            }
            Instruction::End {
                self_ip,
                while_ip: end_ip,
            } => {
                let (kind, while_ip, precursor_end_ip) = jump_stack.pop().unwrap();
                assert!(kind == "if" || kind == "else" || kind == "do");
                *precursor_end_ip = ip;
                *self_ip = ip;
                if kind == "do" {
                    *end_ip = Some(while_ip);
                }
            }
            Instruction::While { self_ip, do_ip } => {
                *self_ip = ip;
                jump_stack.push(("while", ip, do_ip));
            }
            Instruction::Do { end_ip } => {
                let (kind, while_ip, precursor_do_ip) = jump_stack.pop().unwrap();
                assert_eq!(kind, "while");
                *precursor_do_ip = ip;
                jump_stack.push(("do", while_ip, end_ip));
            }
            _ => {}
        }
    }
}
