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
}

impl Display for ValType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ValType::Int => write!(f, "int"),
            ValType::Char => write!(f, "char"),
            ValType::Ptr => write!(f, "ptr"),
            ValType::Str => write!(f, "str"),
            ValType::Bool => write!(f, "bool"),
        }
    }
}

fn err_loc(program: &Program, ip: usize) -> String {
    let spread_len = 6;
    let start = if spread_len > ip { 0 } else { ip - spread_len };
    let end = (ip + spread_len).min(program.instructions.len());
    let spread = start..end;
    let output = program.instructions[spread.clone()]
        .iter()
        .enumerate()
        .map(|(idx, i)| {
            if idx == spread.len() / 2 {
                format!("\x1b[31m>>> {}\x1b[0m", i.to_string())
            } else {
                i.to_string()
            }
        })
        .collect::<Vec<_>>();
    output.join(" ")
}

pub fn typecheck(program: &Program, debugger: bool) -> Result<()> {
    use ValType::*;
    let Program { instructions, .. } = program;

    let mut stack = vec![]; // Start with int for argc and a ptr for argv
    let mut snapshots = Vec::new();

    let mut ip = 0;
    while ip < instructions.len() {
        let inst = &instructions[ip];
        macro_rules! pop {
            () => {
                stack
                    .pop()
                    .ok_or(TypecheckError(StackUnderflow))
                    .with_context(|| {
                        format!(
                            "Stack underflow at instruction {}: {}\n\n{}",
                            ip,
                            inst,
                            err_loc(&program, ip)
                        )
                    })?
            };
        }
        macro_rules! expect {
            ($expect:ident) => {{
                let v = pop!();
                if !matches!(v, $expect) {
                    return Err(TypecheckError(InvalidTypeForOp(inst.to_string()))).with_context(
                        || {
                            format!(
                                "Invalid type for {}: Expected {}, got {}.\n\n{}",
                                inst,
                                casey::lower!(stringify!($expect)),
                                v,
                                err_loc(&program, ip)
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
                                    "Invalid type for {}: Expected {}, got {}.\n\n{}",
                                    inst,
                                    casey::lower!(stringify!($($expect)or+)),
                                    v,
                                    err_loc(&program, ip)
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
                            "Not enough arguments for {}: Expected {} items, got {}.\n\n{}",
                            inst,
                            $num,
                            stack.len(),
                            err_loc(&program, ip)
                        )
                    });
                }
                for _ in 0..$num {
                    stack.pop();
                }
            }};
        }

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
                    let (a, b) = tc!(expect: (Int, Ptr, Char, Bool), (Int, Ptr, Char, Bool));
                    match (a, b) {
                        (Int, Int) => stack.push(Int),
                        (Int, Ptr) => stack.push(Ptr),
                        (Ptr, Int) => stack.push(Ptr),
                        (Char, Int) => stack.push(Char),
                        (Int, Char) => stack.push(Int),
                        (Int, Bool) => stack.push(Int),
                        (Bool, Int) => stack.push(Int),
                        (illegal_a, illegal_b) => {
                            return Err(TypecheckError(InvalidTypeForOp(inst.to_string())))
                                .with_context(|| {
                                    format!(
                                        "Invalid type for {}: Expected int or ptr, got {} and {}.\n\n{}",
                                        inst, illegal_a, illegal_b, err_loc(&program, ip)
                                    )
                                });
                        }
                    }
                }
                Op::Sub => {
                    let (a, b) = tc!(expect: (Int, Ptr, Char, Bool), (Int, Ptr, Char, Bool));
                    match (a, b) {
                        (Int, Int) => stack.push(Int),
                        (Ptr, Int) => stack.push(Ptr),
                        (Char, Int) => stack.push(Char),
                        (Int, Char) => stack.push(Int),
                        (Int, Bool) => stack.push(Int),
                        (Bool, Int) => stack.push(Int),
                        (illegal_a, illegal_b) => {
                            return Err(TypecheckError(InvalidTypeForOp(inst.to_string())))
                                .with_context(|| {
                                    format!(
                                        "Invalid type for {}: Expected int or ptr, got {} and {}.\n\n{}",
                                        inst, illegal_a, illegal_b, err_loc(&program, ip)
                                    )
                                });
                        }
                    }
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
                Op::BitwiseAnd => {
                    let (a, b) = tc!(expect: (Int, Char, Bool), (Int, Char, Bool));
                    match (a, b) {
                        (Int, Bool) => stack.push(Int),
                        (Bool, Int) => stack.push(Int),
                        (Bool, Bool) => stack.push(Bool),
                        (Int, Char) => stack.push(Int),
                        (Char, Int) => stack.push(Int),
                        (Char, Char) => stack.push(Char),
                        (Char, Bool) => stack.push(Char),
                        (Bool, Char) => stack.push(Char),
                        (Int, Int) => stack.push(Int),
                        (illegal_a, illegal_b) => {
                            return Err(TypecheckError(InvalidTypeForOp(inst.to_string())))
                                .with_context(|| {
                                    format!(
                                        "Invalid type for {}: Expected int or bool, got {} and {}.\n\n{}",
                                        inst, illegal_a, illegal_b, err_loc(&program, ip)
                                    )
                                });
                        }
                    }
                }
                Op::BitwiseOr => {
                    let (a, b) = tc!(expect: (Int, Char, Bool), (Int, Char, Bool));
                    match (a, b) {
                        (Int, Bool) => stack.push(Int),
                        (Bool, Int) => stack.push(Int),
                        (Bool, Bool) => stack.push(Bool),
                        (Int, Char) => stack.push(Int),
                        (Char, Int) => stack.push(Int),
                        (Char, Char) => stack.push(Char),
                        (Char, Bool) => stack.push(Char),
                        (Bool, Char) => stack.push(Char),
                        (Int, Int) => stack.push(Int),
                        (illegal_a, illegal_b) => {
                            return Err(TypecheckError(InvalidTypeForOp(inst.to_string())))
                                .with_context(|| {
                                    format!(
                                        "Invalid type for {}: Expected int or bool, got {} and {}.\n\n{}",
                                        inst, illegal_a, illegal_b, err_loc(&program, ip)
                                    )
                                });
                        }
                    }
                }
                Op::BitwiseXor => {
                    let (a, b) = tc!(expect: (Int, Char, Bool), (Int, Char, Bool));
                    match (a, b) {
                        (Int, Bool) => stack.push(Int),
                        (Bool, Int) => stack.push(Int),
                        (Bool, Bool) => stack.push(Bool),
                        (Int, Char) => stack.push(Int),
                        (Char, Int) => stack.push(Int),
                        (Char, Char) => stack.push(Char),
                        (Char, Bool) => stack.push(Char),
                        (Bool, Char) => stack.push(Char),
                        (Int, Int) => stack.push(Int),
                        (illegal_a, illegal_b) => {
                            return Err(TypecheckError(InvalidTypeForOp(inst.to_string())))
                                .with_context(|| {
                                    format!(
                                        "Invalid type for {}: Expected int or bool, got {} and {}.\n\n{}",
                                        inst, illegal_a, illegal_b, err_loc(&program, ip)
                                    )
                                });
                        }
                    }
                }
                Op::BitwiseNot => {
                    let t = tc!(expect: (Int, Bool));
                    match t {
                        Int => stack.push(Int),
                        Char => stack.push(Char),
                        Ptr => panic!(),
                        Str => panic!(),
                        Bool => stack.push(Bool),
                    }
                }
                Op::Shl => {
                    tc!(expect: Int, Int => push: Int);
                }
                Op::Shr => {
                    tc!(expect: Int, Int => push: Int);
                }
                Op::Eq => {
                    let (a, b) = tc!(
                        expect: (Int, Ptr, Char, Bool, Str),
                        (Int, Ptr, Char, Bool, Str)
                    );
                    match (a, b) {
                        (Int, Int) => stack.push(Bool),
                        (Int, Char) => stack.push(Bool),
                        (Int, Ptr) => stack.push(Bool),
                        (Int, Bool) => stack.push(Bool),
                        (Char, Char) => stack.push(Bool),
                        (Char, Int) => stack.push(Bool),
                        (Char, Bool) => stack.push(Bool),
                        (Ptr, Ptr) => stack.push(Bool),
                        (Ptr, Int) => stack.push(Bool),
                        (Str, Str) => stack.push(Bool),
                        (Bool, Bool) => stack.push(Bool),
                        (Bool, Int) => stack.push(Bool),
                        (Bool, Char) => stack.push(Bool),
                        (illegal_a, illegal_b) => {
                            return Err(TypecheckError(InvalidTypeForOp(inst.to_string())))
                                .with_context(|| {
                                    format!(
                                        "Invalid type for {}: Expected int or ptr, got {} and {}.\n\n{}",
                                        inst, illegal_a, illegal_b, err_loc(&program, ip)
                                    )
                                });
                        }
                    }
                }
                Op::Neq => {
                    let (a, b) = tc!(
                        expect: (Int, Ptr, Char, Bool, Str),
                        (Int, Ptr, Char, Bool, Str)
                    );
                    match (a, b) {
                        (Int, Int) => stack.push(Bool),
                        (Int, Char) => stack.push(Bool),
                        (Int, Ptr) => stack.push(Bool),
                        (Int, Bool) => stack.push(Bool),
                        (Char, Char) => stack.push(Bool),
                        (Char, Int) => stack.push(Bool),
                        (Char, Bool) => stack.push(Bool),
                        (Ptr, Ptr) => stack.push(Bool),
                        (Ptr, Int) => stack.push(Bool),
                        (Str, Str) => stack.push(Bool),
                        (Bool, Bool) => stack.push(Bool),
                        (Bool, Int) => stack.push(Bool),
                        (Bool, Char) => stack.push(Bool),
                        (illegal_a, illegal_b) => {
                            return Err(TypecheckError(InvalidTypeForOp(inst.to_string())))
                                .with_context(|| {
                                    format!(
                                        "Invalid type for {}: Expected int or ptr, got {} and {}.\n\n{}",
                                        inst, illegal_a, illegal_b, err_loc(&program, ip)
                                    )
                                });
                        }
                    }
                }
                Op::Lt => {
                    let (a, b) = tc!(expect: (Int, Ptr, Char), (Int, Ptr, Char));
                    match (a, b) {
                        (Int, Int) => stack.push(Bool),
                        (Ptr, Ptr) => stack.push(Bool),
                        (Char, Char) => stack.push(Bool),
                        (Char, Int) => stack.push(Bool),
                        (Int, Char) => stack.push(Bool),
                        (illegal_a, illegal_b) => {
                            return Err(TypecheckError(InvalidTypeForOp(inst.to_string())))
                                .with_context(|| {
                                    format!(
                                        "Invalid type for {}: Expected int or ptr, got {} and {}.\n\n{}",
                                        inst, illegal_a, illegal_b, err_loc(&program, ip)
                                    )
                                });
                        }
                    }
                }
                Op::Gt => {
                    let (a, b) = tc!(expect: (Int, Ptr, Char), (Int, Ptr, Char));
                    match (a, b) {
                        (Int, Int) => stack.push(Bool),
                        (Ptr, Ptr) => stack.push(Bool),
                        (Char, Char) => stack.push(Bool),
                        (Char, Int) => stack.push(Bool),
                        (Int, Char) => stack.push(Bool),
                        (illegal_a, illegal_b) => {
                            return Err(TypecheckError(InvalidTypeForOp(inst.to_string())))
                                .with_context(|| {
                                    format!(
                                        "Invalid type for {}: Expected int or ptr, got {} and {}.\n\n{}",
                                        inst, illegal_a, illegal_b, err_loc(&program, ip)
                                    )
                                });
                        }
                    }
                }
                Op::Lte => {
                    let (a, b) = tc!(expect: (Int, Ptr, Char), (Int, Ptr, Char));
                    match (a, b) {
                        (Int, Int) => stack.push(Bool),
                        (Ptr, Ptr) => stack.push(Bool),
                        (Char, Char) => stack.push(Bool),
                        (Char, Int) => stack.push(Bool),
                        (Int, Char) => stack.push(Bool),
                        (illegal_a, illegal_b) => {
                            return Err(TypecheckError(InvalidTypeForOp(inst.to_string())))
                                .with_context(|| {
                                    format!(
                                        "Invalid type for {}: Expected int or ptr, got {} and {}.\n\n{}",
                                        inst, illegal_a, illegal_b, err_loc(&program, ip)
                                    )
                                });
                        }
                    }
                }
                Op::Gte => {
                    let (a, b) = tc!(expect: (Int, Ptr, Char), (Int, Ptr, Char));
                    match (a, b) {
                        (Int, Int) => stack.push(Bool),
                        (Ptr, Ptr) => stack.push(Bool),
                        (Char, Char) => stack.push(Bool),
                        (Char, Int) => stack.push(Bool),
                        (Int, Char) => stack.push(Bool),
                        (illegal_a, illegal_b) => {
                            return Err(TypecheckError(InvalidTypeForOp(inst.to_string())))
                                .with_context(|| {
                                    format!(
                                        "Invalid type for {}: Expected int or ptr, got {} and {}.\n\n{}",
                                        inst, illegal_a, illegal_b, err_loc(&program, ip)
                                    )
                                });
                        }
                    }
                }
                Op::Store => {
                    tc!(expect: (Int, Char, Bool), Ptr);
                }
                Op::Store64 => {
                    tc!(expect: (Int, Char, Bool), Ptr);
                }
                Op::Load => {
                    tc!(expect: Ptr);
                    stack.push(Int);
                }
                Op::Load64 => {
                    tc!(expect: Ptr);
                    stack.push(Int);
                }
                Op::Mod => {
                    let (a, b) = tc!(expect: (Int, Char, Ptr), (Int, Char));
                    match (a, b) {
                        (Int, Int) => stack.push(Int),
                        (Char, Char) => stack.push(Char),
                        (Char, Int) => stack.push(Char),
                        (Int, Char) => stack.push(Int),
                        (Ptr, Int) => stack.push(Ptr),
                        (Ptr, Char) => stack.push(Ptr),
                        (illegal_a, illegal_n) => {
                            return Err(TypecheckError(InvalidTypeForOp(inst.to_string())))
                                .with_context(|| {
                                    format!(
                                        "Invalid type for {}: Expected (int | char | ptr) and (int | char), got {} and {}.\n\n{}",
                                        inst, illegal_a, illegal_n, err_loc(&program, ip)
                                    )
                                });
                        }
                    }
                }
            },
            Instruction::Intrinsic(i) => match i {
                Intrinsic::Argc => tc!(push: Int),
                Intrinsic::Argv => tc!(push: Ptr),
                Intrinsic::Print => require!(1),
                Intrinsic::Panic => require!(0),
                Intrinsic::Dup => {
                    let a = pop!();
                    stack.push(a);
                    stack.push(a);
                }
                Intrinsic::Dup2 => {
                    let a = pop!();
                    let b = pop!();
                    stack.push(b);
                    stack.push(a);
                    stack.push(b);
                    stack.push(a);
                }
                Intrinsic::Swap => {
                    let a = pop!();
                    let b = pop!();
                    stack.push(a);
                    stack.push(b);
                }
                Intrinsic::Mem => {
                    tc!(push: Ptr);
                }
                Intrinsic::Drop => require!(1),
                Intrinsic::Drop2 => require!(2),
                Intrinsic::Over => {
                    let a = pop!();
                    let b = pop!();
                    stack.push(b);
                    stack.push(a);
                    stack.push(b);
                }
                Intrinsic::CastPtr => {
                    tc!(expect: Int => push: Ptr);
                }
                Intrinsic::CastInt => {
                    tc!(expect: (Char, Ptr, Bool) => push: Int);
                }
            },
            Instruction::Keyword(kw) => match kw {
                Keyword::While { .. } => {
                    snapshots.push((
                        stack.clone(),
                        Keyword::While {
                            self_ip: 0,
                            do_ip: 0,
                        },
                    ));
                }
                Keyword::Do { .. } => {
                    tc!(expect: Bool);
                    let (stack_snapshot, op_type) = snapshots
                        .pop()
                        .ok_or(TypecheckError(InvalidLoop))
                        .with_context(|| format!("Invalid do: No stack snapshot available"))?;
                    if let Keyword::While { .. } = op_type {
                        if stack != stack_snapshot {
                            return Err(TypecheckError(InvalidLoop)).with_context(|| {
                                format!(
                                    "Expected types {:?}, got {:?}. A while loop cannot modify the stack.\n\n{}",
                                    stack_snapshot, stack, err_loc(&program, ip)
                                )
                            });
                        }
                        snapshots.push((stack.clone(), Keyword::Do { end_ip: 0 }));
                    } else {
                        return Err(TypecheckError(InvalidLoop)).with_context(|| {
                            format!(
                                "Invalid do: Expected while, got {:?}\n\n{}",
                                op_type,
                                err_loc(&program, ip)
                            )
                        });
                    }
                }
                Keyword::If { else_ip } => {
                    tc!(expect: (Bool, Int, Ptr, Char));
                    snapshots.push((stack.clone(), Keyword::If { else_ip: *else_ip }));
                }
                Keyword::Else { .. } => {
                    let (stack_snapshot, op_type) = snapshots
                        .pop()
                        .ok_or(TypecheckError(InvalidElse))
                        .with_context(|| {
                            format!(
                                "Invalid else: No stack snapshot available: \n\n{}",
                                err_loc(&program, ip)
                            )
                        })?;
                    if let Keyword::If { .. } = op_type {
                        snapshots.push((
                            std::mem::replace(&mut stack, stack_snapshot),
                            Keyword::Else {
                                else_ip: 0,
                                end_ip: 0,
                            },
                        ));
                    } else {
                        return Err(TypecheckError(InvalidElse)).with_context(|| {
                            format!(
                                "Invalid else: Expected if, got {:?}\n\n{}",
                                op_type,
                                err_loc(&program, ip)
                            )
                        });
                    }
                }
                Keyword::End { .. } => {
                    let (expected_stack, op_type) = snapshots
                        .pop()
                        .ok_or(TypecheckError(InvalidEnd))
                        .with_context(|| format!("Invalid end: No stack snapshot available"))?;
                    if let Keyword::Do { .. } = op_type {
                        if stack != expected_stack {
                            return Err(TypecheckError(InvalidEnd)).with_context(|| {
                                format!(
                                    "Expected types {:?}, got {:?}. A while loop cannot modify the stack.\n\n{}",
                                    expected_stack, stack, err_loc(&program, ip)
                                )
                            });
                        }
                    } else if let Keyword::If { .. } = op_type {
                        if stack != expected_stack {
                            return Err(TypecheckError(InvalidEnd)).with_context(|| {
                                format!(
                                    "Expected types {:?}, got {:?}. An elseless if statement cannot modify the stack.\n\n{}",
                                    expected_stack, stack, err_loc(&program, ip)
                                )
                            });
                        }
                    } else if let Keyword::Else { .. } = op_type {
                        if stack != expected_stack {
                            return Err(TypecheckError(InvalidEnd)).with_context(|| {
                                format!(
                                    "Expected types {:?}, got {:?}. Both branches of an if statement must push the same types to the stack\n\n{}",
                                    expected_stack, stack, err_loc(&program, ip)
                                )
                            });
                        }
                    } else {
                        unreachable!()
                    }
                }
                Keyword::Macro => {
                    return Err(TypecheckError(MacroInCode)).with_context(|| {
                        format!(
                            "Unexpected macro in code at instruction {}\n\n{}",
                            ip,
                            err_loc(&program, ip)
                        )
                    })
                }
                Keyword::Include => {
                    return Err(TypecheckError(IncludeInCode)).with_context(|| {
                        format!(
                            "Unexpected include in code at instruction {}\n\n{}",
                            ip,
                            err_loc(&program, ip)
                        )
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
        if debugger {
            println!("{}: {:?}", ip, inst);
            println!("Stack: {:?}", stack);
            println!("Snapshots: {}\n", snapshots.len());
            std::io::stdin().read_line(&mut String::new()).unwrap();
        }

        ip += 1;
    }

    if stack.len() > 1 {
        return Err(TypecheckError(InvalidStack)).with_context(|| {
            format!(
                "Invalid stack at end of program: Expected argc and/or return code, stack size was {}.",
                stack.len()
            )
        });
    } else if stack.len() == 1 && !matches!(&stack[0], ValType::Int) {
        return Err(TypecheckError(InvalidStack)).with_context(|| {
            format!(
                "Invalid stack at end of program: Expected argc and/or return code as int, got {}.",
                &stack[0]
            )
        });
    }
    Ok(())
}
