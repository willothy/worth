use std::fmt::Display;

use anyhow::{Context, Result};

use crate::error::{Error::TypecheckError, TypecheckError::*};
use crate::instruction::{Instruction, Op, Program, SyscallKind, Value};

pub enum ValType {
    Int,
    Char,
    Ptr,
    Str,
}

impl Display for ValType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ValType::Int => write!(f, "int"),
            ValType::Char => write!(f, "char"),
            ValType::Ptr => write!(f, "ptr"),
            ValType::Str => write!(f, "str"),
        }
    }
}

pub fn typecheck(program: &Program) -> Result<()> {
    let Program {
        instructions: program,
        ..
    } = program;

    let mut stack = Vec::new();

    let mut ip = 0;
    while ip < program.len() {
        macro_rules! pop {
            () => {
                stack
                    .pop()
                    .ok_or(TypecheckError(StackUnderflow))
                    .with_context(|| format!("Stack underflow at instruction {}", ip))?
            };
            ($op:expr, $expect:tt) => {
                let v = pop!();
                if !matches!(v, $expect) {
                    return Err(TypecheckError(InvalidTypeForOp($op))).with_context(|| {
                        format!("Invalid type for {}: Expected {}, got {}.", $op, $expect, v)
                    });
                }
            };
        }
        macro_rules! push {
            ($v:expr) => {
                stack.push($v)
            };
        }

        let inst = &program[ip];

        use ValType::*;
        match inst {
            Instruction::Push(val) => match val {
                Value::Int(_) => push!(Int),
                Value::Char(_) => push!(Char),
                Value::Str(_) => {
                    push!(Int);
                    push!(Ptr);
                }
                Value::Ptr(_) => push!(Ptr),
            },
            Instruction::Op(op) => match op {
                Op::Add => {
                    pop!("add", Int);
                    pop!("add", Int);
                }
                Op::Sub => todo!(),
                Op::Mul => todo!(),
                Op::Div => todo!(),
                Op::DivMod => todo!(),
                Op::BitwiseAnd => todo!(),
                Op::BitwiseOr => todo!(),
                Op::BitwiseXor => todo!(),
                Op::BitwiseNot => todo!(),
                Op::Shl => todo!(),
                Op::Shr => todo!(),
                Op::Eq => todo!(),
                Op::Neq => todo!(),
                Op::Lt => todo!(),
                Op::Gt => todo!(),
                Op::Lte => todo!(),
                Op::Gte => todo!(),
                Op::Store => todo!(),
                Op::Load => todo!(),
                Op::Load64 => todo!(),
                Op::Store64 => todo!(),
                Op::Mod => todo!(),
            },
            Instruction::Syscall(SyscallKind::Syscall0) => {
                pop!();
                push!(Int);
            }
            Instruction::Syscall(SyscallKind::Syscall1) => {}
            Instruction::Syscall(SyscallKind::Syscall2) => {}
            Instruction::Syscall(SyscallKind::Syscall3) => {}
            Instruction::Syscall(SyscallKind::Syscall4) => {}
            Instruction::Syscall(SyscallKind::Syscall5) => {}
            Instruction::Syscall(SyscallKind::Syscall6) => {}
            unim => todo!("Implement typechecking for instruction {}", unim),
        }
        ip += 1;
    }
    Ok(())
}
