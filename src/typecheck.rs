use std::fmt::Display;

use anyhow::{Context, Result};

use crate::codegen::intrinsics::Intrinsic;
use crate::error::{Error::TypecheckError, TypecheckError::*};
use crate::instruction::{Instruction, Keyword, Op, Program, SyscallKind, Value};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ValType {
    Int,
    Char,
    Ptr,
    Str,
    Bool,
    LoopStop,
}

impl Display for ValType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ValType::Int => write!(f, "int"),
            ValType::Char => write!(f, "char"),
            ValType::Ptr => write!(f, "ptr"),
            ValType::Str => write!(f, "str"),
            ValType::Bool => write!(f, "bool"),
            ValType::LoopStop => write!(f, "loopstop"),
        }
    }
}

pub fn typecheck(program: &Program) -> Result<()> {
    let Program {
        instructions: program,
        ..
    } = program;

    let mut stack = Vec::new();
    let mut snapshots = Vec::new();

    let mut ip = 0;
    while ip < program.len() {
        let inst = &program[ip];
        macro_rules! pop {
            () => {
                stack
                    .pop()
                    .ok_or(TypecheckError(StackUnderflow))
                    .with_context(|| format!("Stack underflow at instruction {}: {}", ip, inst))?
            };
        }
        macro_rules! expect {
            ($expect:ident) => {{
                let v = pop!();
                if !matches!(v, $expect) {
                    return Err(TypecheckError(InvalidTypeForOp(inst.to_string()))).with_context(
                        || {
                            format!(
                                "Invalid type for {}: Expected {}, got {}.",
                                inst,
                                casey::lower!(stringify!($expect)),
                                v
                            )
                        },
                    );
                } else {
                    $expect
                }
            }};
            (($($expect:ident),+)) => {{
                let v = pop!();
                #[allow(unreachable_patterns)]
                match v {
                    $($expect => $expect,)+
                    _ => {
                        return Err(TypecheckError(InvalidTypeForOp(inst.to_string()))).with_context(
                            || {
                                format!(
                                    "Invalid type for {}: Expected {}, got {}.",
                                    inst,
                                    casey::lower!(stringify!($($expect)or+)),
                                    v
                                )
                            },
                        );
                    }
                }
            }};
        }
        macro_rules! tc {
            (
                expect: $($expect:tt),+ =>
                push: $($result:ident),+
            ) => {
                {
                    let found = ($(expect!($expect)),+);
                    $(stack.push($result);)+
                    found
                }
            };
            (
                push: $($result:ident),+
            ) => {
                {
                    $(stack.push($result);)+
                }
            };
            (
                expect: $($expect:tt),+
            ) => {
                {
                    let found = ($(expect!($expect)),+);
                    found
                }
            };
        }

        macro_rules! require {
            ($num:expr) => {{
                #[allow(unused_comparisons)]
                if stack.len() < $num {
                    return Err(TypecheckError(StackUnderflow)).with_context(|| {
                        format!(
                            "Not enough arguments for {}: Expected {} items, got {}.",
                            inst,
                            $num,
                            stack.len()
                        )
                    });
                }
                for _ in 0..$num {
                    stack.pop();
                }
            }};
        }

        use ValType::*;
        match inst {
            Instruction::Push(val) => match val {
                Value::Int(_) => {
                    tc!(push: Int);
                }
                Value::Char(_) => {
                    tc!(push: Char);
                }
                Value::Str(_) => {
                    tc!(push: Int, Ptr);
                }
                Value::Ptr(_) => {
                    tc!(push: Ptr);
                }
            },
            Instruction::Op(op) => match op {
                Op::Add => {
                    let (a, b) = tc!(expect: (Int, Ptr), Int);
                    match (a, b) {
                        (Int, Int) => stack.push(Int),
                        (Int, Ptr) => stack.push(Ptr),
                        (Ptr, Int) => stack.push(Ptr),
                        (illegal_a, illegal_b) => {
                            return Err(TypecheckError(InvalidTypeForOp(inst.to_string())))
                                .with_context(|| {
                                    format!(
                                        "Invalid type for {}: Expected int or ptr, got {} and {}.",
                                        inst, illegal_a, illegal_b
                                    )
                                });
                        }
                    }
                }
                Op::Sub => {
                    tc!(expect: (Int, Ptr), (Int, Ptr) => push: Int);
                }
                Op::Mul => {
                    tc!(expect: Int, Int => push: Int);
                }
                Op::Div => {
                    tc!(expect: Int, Int => push: Int);
                }
                Op::DivMod => {
                    tc!(expect: Int, Int =>  push: Int, Int);
                }
                Op::BitwiseAnd => todo!(),
                Op::BitwiseOr => todo!(),
                Op::BitwiseXor => todo!(),
                Op::BitwiseNot => todo!(),
                Op::Shl => todo!(),
                Op::Shr => todo!(),
                Op::Eq => todo!(),
                Op::Neq => todo!(),
                Op::Lt => {
                    let (a, b) = tc!(expect: (Int, Ptr), (Int, Ptr));
                    if a == b && matches!(a, Int | Ptr) {
                        stack.push(Bool);
                    } else {
                        return Err(TypecheckError(InvalidTypeForOp(inst.to_string())))
                            .with_context(|| format!("Invalid arg types for op {}: Expected matching types, found {} and {}", inst, a, b));
                    }
                }
                Op::Gt => {
                    let (a, b) = tc!(expect: (Int, Ptr), (Int, Ptr));
                    if a == b && matches!(a, Int | Ptr) {
                        stack.push(Bool);
                    } else {
                        return Err(TypecheckError(InvalidTypeForOp(inst.to_string())))
                            .with_context(|| format!("Invalid arg types for op {}: Expected matching types, found {} and {}", inst, a, b));
                    }
                }
                Op::Lte => todo!(),
                Op::Gte => todo!(),
                Op::Store => todo!(),
                Op::Load => todo!(),
                Op::Load64 => todo!(),
                Op::Store64 => todo!(),
                Op::Mod => todo!(),
            },
            Instruction::Intrinsic(i) => match i {
                Intrinsic::Print => require!(1),
                Intrinsic::Panic => require!(0),
                Intrinsic::Dup => {
                    let a = pop!();
                    stack.push(a);
                    stack.push(a);
                }
                Intrinsic::Dup2 => todo!(),
                Intrinsic::Swap => {
                    let a = pop!();
                    let b = pop!();
                    stack.push(a);
                    stack.push(b);
                }
                Intrinsic::Mem => todo!(),
                Intrinsic::Drop => require!(1),
                Intrinsic::Drop2 => require!(2),
                Intrinsic::Over => {
                    let a = pop!();
                    let b = pop!();
                    stack.push(b);
                    stack.push(a);
                    stack.push(b);
                }
            },
            Instruction::Keyword(kw) => match kw {
                Keyword::While { .. } => {}
                Keyword::Do { end_ip } => {
                    tc!(expect: Bool);
                    snapshots.push((stack.clone(), Keyword::Do { end_ip: *end_ip }));
                }
                Keyword::If { else_ip } => {
                    tc!(expect: Bool);
                    snapshots.push((stack.clone(), Keyword::If { else_ip: *else_ip }));
                }
                Keyword::Else { .. } => todo!(),
                Keyword::End { .. } => {
                    let (expected_stack, op_type) = snapshots
                        .pop()
                        .ok_or(TypecheckError(InvalidEnd))
                        .with_context(|| format!("Invalid end: No stack snapshot available"))?;
                    if let Keyword::Do { .. } = op_type {
                        if stack != expected_stack {
                            return Err(TypecheckError(InvalidEnd)).with_context(|| {
                                format!(
                                    "Invalid end: Expected types {:?}, got {:?}",
                                    expected_stack, stack
                                )
                            });
                        }
                    } else if let Keyword::If { .. } = op_type {
                        if stack != expected_stack {
                            return Err(TypecheckError(InvalidEnd)).with_context(|| {
                                format!(
                                    "Invalid end: Expected types {:?}, got {:?}",
                                    expected_stack, stack
                                )
                            });
                        }
                    } else if let Keyword::Else { .. } = op_type {
                        unimplemented!()
                    } else {
                        unreachable!()
                    }
                }
                Keyword::Macro => {
                    return Err(TypecheckError(MacroInCode))
                        .with_context(|| format!("Unexpected macro in code at instruction {}", ip))
                }
                Keyword::Include => {
                    return Err(TypecheckError(IncludeInCode)).with_context(|| {
                        format!("Unexpected include in code at instruction {}", ip)
                    })
                }
            },
            // TODO: Figure out how to typecheck syscall args and return types
            Instruction::Syscall(s) => {
                require!(match s {
                    SyscallKind::Syscall0 => 1,
                    SyscallKind::Syscall1 => 2,
                    SyscallKind::Syscall2 => 3,
                    SyscallKind::Syscall3 => 4,
                    SyscallKind::Syscall4 => 5,
                    SyscallKind::Syscall5 => 6,
                    SyscallKind::Syscall6 => 7,
                });
                tc!(push: Int)
            }
            unim => todo!("Implement typechecking for instruction {}", unim),
        };
        ip += 1;
    }

    if stack.len() > 1 {
        return Err(TypecheckError(InvalidStack)).with_context(|| {
            format!(
                "Invalid stack at end of program: Expected empty stack or return code, stack size was {}.",
                stack.len()
            )
        });
    } else if stack.len() == 1 && !matches!(&stack[0], ValType::Int) {
        return Err(TypecheckError(InvalidStack)).with_context(|| {
            format!(
                "Invalid stack at end of program: Expected empty stack or return code as int, got {}.",
                &stack[0]
            )
        });
    }
    Ok(())
}
