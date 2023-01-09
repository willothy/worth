use std::path::PathBuf;

use super::intrinsics::gen_intrinsics;
use super::ops;
use crate::{
    asm, asm_line,
    cli::{CompilerOptions, OutputType},
    codegen::builder::Builder,
    comment,
    error::{
        BoolError, CompileError::*, Error::CompileError, Error::IOError, IOError::NoFileExtension,
    },
    global,
    instruction::*,
    label,
    log::{self, LogLevel},
    segment, syscall,
};

use anyhow::{Context, Result};

pub const BSS_CAPACITY: usize = 640_000;

pub fn compile(program: &Program, opt: CompilerOptions) -> Result<PathBuf> {
    let mut asm = Builder::new();
    comment!(asm, "-- generated by the worth compiler --");

    segment!(asm, "bss");

    label!(asm, "mem");
    asm!(asm, ("resb", "{}", BSS_CAPACITY));

    label!(asm, "args_ptr");
    asm!(asm, ("resq", "1"));

    segment!(asm, "text");
    global!(asm, "_start");
    label!(asm, "_start");

    asm!(
        asm,
        /// Save the stack pointer for argc and argv intrinsics
        ("mov", "[args_ptr], rsp")
    );

    let Program {
        instructions: program,
        name: program_name,
        ..
    } = program;

    for inst in program {
        match &inst.kind {
            InstructionKind::Push(val) => match val {
                Value::Int(i) => {
                    asm!(asm, ("push", "{}", i))
                }
                Value::Char(c) => {
                    asm!(asm, ("push", "{}", c))
                }
                Value::Ptr(_) => todo!(),
                Value::Str(s) => {
                    let s_id = asm.new_const_str(s);
                    asm!(
                        asm,
                        ("mov", "rax, {}", s.as_bytes().len()),
                        ("push", "rax"),
                        ("mov", "rax, const_str_{}", s_id),
                        ("push", "rax")
                    );
                }
            },
            InstructionKind::Intrinsic(intrinsic) => {
                comment!(
                    asm,
                    &format!("-- intrinsic: {} --", intrinsic.to_string().to_lowercase())
                );
                intrinsic.compile()(&mut asm);
                comment!(asm, "-- end intrinsic --");
            }
            InstructionKind::Keyword(Keyword::While { self_ip, .. }) => {
                comment!(asm, "-- while --");
                label!(asm, "addr_{}", self_ip);
            }
            InstructionKind::Keyword(Keyword::Do { end_ip }) => {
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
            InstructionKind::Keyword(Keyword::If { else_ip }) => {
                comment!(asm, "-- if --");
                asm!(
                    asm,
                    ("pop", "rax"),
                    ("test", "rax, rax"),
                    /// Jump to else statement
                    ("jz", "addr_{}", else_ip)
                );
            }
            InstructionKind::Keyword(Keyword::Else { else_ip, end_ip }) => {
                comment!(asm, "-- else --");
                asm!(
                    asm,
                    /// Jump to end of if statement
                    ("jmp", "addr_{}", end_ip)
                );
                label!(asm, "addr_{}", else_ip);
            }
            InstructionKind::Keyword(Keyword::End { self_ip, while_ip }) => {
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
            InstructionKind::Op(Op::Add) => ops::add(&mut asm),
            InstructionKind::Op(Op::Sub) => ops::sub(&mut asm),
            InstructionKind::Op(Op::Mul) => ops::mul(&mut asm),
            InstructionKind::Op(Op::Div) => ops::div(&mut asm),
            InstructionKind::Op(Op::Mod) => ops::mod_(&mut asm),
            InstructionKind::Op(Op::DivMod) => ops::divmod(&mut asm),
            InstructionKind::Op(Op::BitwiseAnd) => ops::band(&mut asm),
            InstructionKind::Op(Op::BitwiseOr) => ops::bor(&mut asm),
            InstructionKind::Op(Op::BitwiseXor) => ops::xor(&mut asm),
            InstructionKind::Op(Op::BitwiseNot) => ops::not(&mut asm),
            InstructionKind::Op(Op::Shl) => ops::shl(&mut asm),
            InstructionKind::Op(Op::Shr) => ops::shr(&mut asm),
            InstructionKind::Op(Op::Eq) => ops::eq(&mut asm),
            InstructionKind::Op(Op::Neq) => ops::neq(&mut asm),
            InstructionKind::Op(Op::Lt) => ops::lt(&mut asm),
            InstructionKind::Op(Op::Gt) => ops::gt(&mut asm),
            InstructionKind::Op(Op::Lte) => ops::lte(&mut asm),
            InstructionKind::Op(Op::Gte) => ops::gte(&mut asm),
            InstructionKind::Op(Op::Load) => ops::load(&mut asm),
            InstructionKind::Op(Op::Store) => ops::store(&mut asm),
            InstructionKind::Op(Op::Load64) => ops::load64(&mut asm),
            InstructionKind::Op(Op::Store64) => ops::store64(&mut asm),
            InstructionKind::Syscall(SyscallKind::Syscall0) => ops::syscall0(&mut asm),
            InstructionKind::Syscall(SyscallKind::Syscall1) => ops::syscall1(&mut asm),
            InstructionKind::Syscall(SyscallKind::Syscall2) => ops::syscall2(&mut asm),
            InstructionKind::Syscall(SyscallKind::Syscall3) => ops::syscall3(&mut asm),
            InstructionKind::Syscall(SyscallKind::Syscall4) => ops::syscall4(&mut asm),
            InstructionKind::Syscall(SyscallKind::Syscall5) => ops::syscall5(&mut asm),
            InstructionKind::Syscall(SyscallKind::Syscall6) => ops::syscall6(&mut asm),
            InstructionKind::Keyword(Keyword::Include) => {
                return Err(CompileError(UnexpectedToken("include".into())))
                    .with_context(|| "Include should be expanded before codegen")
            }
            InstructionKind::Keyword(Keyword::Macro) => {
                return Err(CompileError(UnexpectedToken("macro".into())))
                    .with_context(|| "Macro should be expanded before codegen")
            }
            InstructionKind::Name(name) => {
                return Err(CompileError(UnexpectedToken("macro".into())))
                    .with_context(|| format!("Name {} should be resolved before codegen", name))
            }
        }
    }

    syscall!(asm, 60, 0);

    gen_intrinsics(&mut asm);

    // Write asm to out.asm
    let out_path = opt.output.unwrap_or_else(|| program_name.into());
    let output_type = match out_path.extension() {
        Some(ext) => match ext
            .to_str()
            .ok_or(IOError(NoFileExtension))
            .with_context(|| format!("Invalid filename: {}", out_path.to_string_lossy()))?
        {
            "asm" => OutputType::Asm,
            "o" => OutputType::Obj,
            "exe" => OutputType::Exe,
            _ => {
                log::log(
                    LogLevel::Warn,
                    format!(
                        "Unknown output type {}. Building elf64 executable.",
                        ext.to_str()
                            .ok_or(IOError(NoFileExtension))
                            .with_context(|| {
                                format!("Invalid filename: {}", out_path.to_string_lossy())
                            })?
                    ),
                    opt.debug,
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

    let count_lines = asm.count_lines();
    let asm = asm.finalize();
    std::fs::write(&asm_out_path, asm)
        .with_context(|| format!("Could not write asm to {}", asm_out_path.to_string_lossy()))?;
    log::log(
        LogLevel::Info,
        format!("Wrote {} lines to {}", count_lines, asm_out_path_str),
        opt.debug,
    );

    if matches!(output_type, OutputType::Asm) {
        return Ok(asm_out_path);
    }

    // Call nasm
    let mut nasm_cmd = std::process::Command::new("nasm");
    nasm_cmd.args(&[&asm_out_path_str, "-f", "elf64", "-o", &obj_out_path_str]);
    log::log(
        LogLevel::Cmd,
        format!("{:?}", nasm_cmd).replace("\"", ""),
        opt.debug,
    );

    let nasm = nasm_cmd
        .spawn()
        .map_err(|e| CompileError(NasmInvokeError(e)))
        .with_context(|| format!("Failed to spawn nasm process"))?
        .wait_with_output()
        .map_err(|e| CompileError(NasmInvokeError(e)))
        .with_context(|| format!("Failed to wait for nasm process to complete"))?;

    nasm.status
        .success()
        .to_err()
        .map_err(|_| CompileError(NasmCompileError))
        .with_context(|| {
            format!(
                "Nasm failed to compile {}:\n{}\n",
                asm_out_path_str,
                String::from_utf8_lossy(&nasm.stderr)
            )
        })?;

    if !opt.keep_asm {
        if let Err(e) = std::fs::remove_file(&asm_out_path_str) {
            log::log(
                LogLevel::Warn,
                format!("Could not remove asm file {}: {}", asm_out_path_str, e),
                opt.debug,
            );
        };
    }

    if matches!(output_type, OutputType::Obj) {
        return Ok(obj_out_path_str.into());
    }

    // Call ld
    let mut ld_cmd = std::process::Command::new("ld");
    ld_cmd.args(&[&obj_out_path_str, "-o", &exe_out_path_str]);
    log::log(
        LogLevel::Cmd,
        format!("{:?}", ld_cmd).replace("\"", ""),
        opt.debug,
    );
    let ld = ld_cmd
        .spawn()
        .map_err(|e| CompileError(LdInvokeError(e)))
        .with_context(|| format!("Failed to spawn ld process"))?
        .wait_with_output()
        .map_err(|e| CompileError(LdInvokeError(e)))
        .with_context(|| format!("Failed to wait for ld process to complete"))?;

    ld.status
        .success()
        .to_err()
        .map_err(|_| CompileError(LdLinkError))
        .with_context(|| {
            format!(
                "Ld failed to link {}:\n{}\n",
                obj_out_path_str,
                String::from_utf8_lossy(&ld.stderr)
            )
        })?;

    if !opt.keep_obj {
        if let Err(e) = std::fs::remove_file(&obj_out_path_str) {
            log::log(
                LogLevel::Warn,
                format!("Could not remove object file {}: {}", obj_out_path_str, e),
                opt.debug,
            );
        };
    }

    Ok(exe_out_path_str.into())
}
