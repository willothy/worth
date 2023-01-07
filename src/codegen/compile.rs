use super::intrinsics::gen_intrinsics;
use super::ops;
use crate::cli::{CompilerOptions, OutputType};
use crate::{asm, asm_line, comment, global, label, segment};
use crate::{instruction::*, syscall};

pub const MEM_CAPACITY: usize = 640_000;

pub fn compile(program: &Program, opt: CompilerOptions) -> Result<(), Box<dyn std::error::Error>> {
    let mut asm = vec![];
    comment!(asm, "-- generated by the worth compiler --");

    segment!(asm, "bss");
    label!(asm, "mem");
    asm!(asm, ("resb", "{}", MEM_CAPACITY));

    segment!(asm, "text");
    global!(asm, "_start");
    label!(asm, "_start");

    let Program {
        instructions: program,
        name: program_name,
        ..
    } = program;

    for inst in program {
        match &inst {
            Instruction::Push(val) => match val {
                Value::Int(i) => {
                    asm!(asm, ("push", "{}", i))
                }
                Value::Char(_) => todo!(),
                Value::Ptr(_) => todo!(),
            },
            Instruction::Intrinsic(intrinsic) => {
                comment!(
                    asm,
                    &format!("-- intrinsic: {} --", intrinsic.to_string().to_lowercase())
                );
                intrinsic.compile()(&mut asm);
                comment!(asm, "-- end intrinsic --");
            }
            Instruction::While { self_ip, .. } => {
                comment!(asm, "-- while --");
                label!(asm, "addr_{}", self_ip);
            }
            Instruction::Do { end_ip } => {
                asm!(
                    asm,
                    ("pop", "rax"),
                    /// While loop condition
                    ("test", "rax, rax"),
                    /// Jump to end of while loop
                    ("jz", "addr_{}", end_ip)
                );
                comment!(asm, "-- do --");
            }
            Instruction::If { else_ip } => {
                comment!(asm, "-- if --");
                asm!(
                    asm,
                    ("pop", "rax"),
                    ("test", "rax, rax"),
                    /// Jump to else statement
                    ("jz", "addr_{}", else_ip)
                );
            }
            Instruction::Else { else_ip, end_ip } => {
                comment!(asm, "-- else --");
                asm!(
                    asm,
                    /// Jump to end of if statement
                    ("jmp", "addr_{}", end_ip)
                );
                label!(asm, "addr_{}", else_ip);
            }
            Instruction::End { self_ip, while_ip } => {
                comment!(asm, "-- end --");
                if let Some(while_ip) = while_ip {
                    asm!(
                        asm,
                        /// Jump to while statement
                        ("jmp", "addr_{}", while_ip)
                    )
                }
                label!(asm, "addr_{}", self_ip);
            }
            Instruction::Add => ops::add(&mut asm),
            Instruction::Sub => ops::sub(&mut asm),
            Instruction::Mul => ops::mul(&mut asm),
            Instruction::Div => ops::div(&mut asm),
            Instruction::Mod => ops::rem(&mut asm),
            Instruction::BitwiseAnd => ops::band(&mut asm),
            Instruction::BitwiseOr => ops::bor(&mut asm),
            Instruction::BitwiseXor => ops::xor(&mut asm),
            Instruction::BitwiseNot => ops::not(&mut asm),
            Instruction::Shl => ops::shl(&mut asm),
            Instruction::Shr => ops::shr(&mut asm),
            Instruction::Eq => ops::eq(&mut asm),
            Instruction::Neq => ops::neq(&mut asm),
            Instruction::Lt => ops::lt(&mut asm),
            Instruction::Gt => ops::gt(&mut asm),
            Instruction::Lte => ops::lte(&mut asm),
            Instruction::Gte => ops::gte(&mut asm),
            Instruction::Load => ops::load(&mut asm),
            Instruction::Store => ops::store(&mut asm),
            Instruction::Syscall0 => ops::syscall0(&mut asm),
            Instruction::Syscall1 => ops::syscall1(&mut asm),
            Instruction::Syscall2 => ops::syscall2(&mut asm),
            Instruction::Syscall3 => ops::syscall3(&mut asm),
            Instruction::Syscall4 => ops::syscall4(&mut asm),
            Instruction::Syscall5 => ops::syscall5(&mut asm),
            Instruction::Syscall6 => ops::syscall6(&mut asm),
            Instruction::Macro => unreachable!("Macro should be expanded before codegen"),
            Instruction::Name(name) => {
                unreachable!("Name {} should be expanded before codegen", name)
            }
        }
    }

    syscall!(asm, 60, 0);

    gen_intrinsics(&mut asm);

    // Write asm to out.asm
    let out_path = opt.output.unwrap_or_else(|| program_name.into());
    let output_type = match out_path.extension() {
        Some(ext) => match ext.to_str().unwrap() {
            "asm" => OutputType::Asm,
            "o" => OutputType::Obj,
            "exe" => OutputType::Exe,
            _ => {
                eprintln!(
                    "Warning: Unknown output type {}. Building elf64 executable.",
                    ext.to_str().unwrap()
                );
                OutputType::Exe
            }
        },
        None => OutputType::Exe,
    };
    let asm_out_path = out_path.with_extension("asm");
    let asm_out_path_str = asm_out_path
        .with_extension("asm")
        .to_string_lossy()
        .to_string();
    let obj_out_path_str = asm_out_path
        .with_extension("o")
        .to_string_lossy()
        .to_string();
    let exe_out_path_str = asm_out_path
        .with_extension("")
        .to_string_lossy()
        .to_string();

    let Ok(_) = std::fs::write(asm_out_path, asm.join("\n")) else {
        return Err(format!("Failed to write asm file").into())
    };

    if matches!(output_type, OutputType::Asm) {
        return Ok(());
    }

    // Call nasm
    let _nasm = std::process::Command::new("nasm")
        .args(&[&asm_out_path_str, "-f", "elf64", "-o", &obj_out_path_str])
        .spawn()?
        .wait()?;
    //std::fs::remove_file(asm_out_path_str)?;

    if matches!(output_type, OutputType::Obj) {
        return Ok(());
    }

    // Call ld
    let _ld = std::process::Command::new("ld")
        .args(&[&obj_out_path_str, "-o", &exe_out_path_str])
        .spawn()?
        .wait()?;
    let Ok(_) = std::fs::remove_file(&obj_out_path_str) else {
        println!("Warning: Could not remove object file {}", obj_out_path_str);
        return Ok(());
    };

    Ok(())
}
