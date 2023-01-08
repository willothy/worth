use std::io::{self, BufRead, BufReader, Write};

use crate::error::{Error::RuntimeError, RuntimeError::*};
use crate::log::{self, LogLevel::*};
use crate::{cli::SimulatorOptions, codegen::intrinsics::Intrinsic, instruction::*};
use anyhow::{Context, Result};

pub struct BinaryIO {
    pub reader: Option<Box<dyn BufRead>>,
    pub writer: Option<Box<dyn Write>>,
}

impl BinaryIO {
    pub fn new(reader: Option<Box<dyn BufRead>>, writer: Option<Box<dyn Write>>) -> Self {
        Self { reader, writer }
    }

    pub fn stdio() -> Vec<Self> {
        vec![
            Self::new(Some(Box::new(BufReader::new(io::stdin()))), None),
            Self::new(None, Some(Box::new(io::stdout()))),
            Self::new(None, Some(Box::new(io::stderr()))),
        ]
    }
}

const STR_CAPACITY: usize = 4096;
const BSS_CAPACITY: usize = 640_000;

pub fn simulate(program: &Program, opt: SimulatorOptions) -> Result<()> {
    let Program {
        instructions: program,
        ..
    } = program;
    let mut stack = Vec::new();
    let mut bss: Vec<u8> = vec![0; STR_CAPACITY + BSS_CAPACITY];
    let mut str_allocated = 0;

    let mut fds: Vec<BinaryIO> = BinaryIO::stdio();
    let debug = opt.debug;

    let mut ip = 0;
    while ip < program.len() {
        macro_rules! pop {
            () => {
                stack
                    .pop()
                    .ok_or(RuntimeError(StackUnderflow))
                    .with_context(|| format!("Stack underflow at instruction {}", ip))?
            };
        }
        let inst = &program[ip];

        log::log(
            Debug,
            format!(
                "ip: {ip:?} inst: {inst:?}\n{}{stack:?}\n{}{:?}\n{}",
                "stack: ",
                "bss[0..10]: ",
                &bss[0..10],
                "-------------------------------------------"
            ),
            debug,
        );

        match &inst {
            Instruction::Syscall(SyscallKind::Syscall0) => {
                let syscall = pop!();
                match syscall {
                    number => todo!("Implement syscall0 {}", number),
                }
            }
            Instruction::Syscall(SyscallKind::Syscall1) => {
                let syscall = pop!();
                let arg1 = pop!();
                match syscall {
                    60 => {
                        // Exit
                        std::process::exit(arg1 as i32);
                    }
                    3 => {
                        // Close
                        fds.remove(arg1 as usize);
                    }
                    number => todo!("Implement syscall1 {}", number),
                }
            }
            #[allow(unused_variables)]
            Instruction::Syscall(SyscallKind::Syscall2) => {
                let syscall = pop!();
                let arg1 = pop!();
                let arg2 = pop!();
                match syscall {
                    number => todo!("Implement syscall2 {}", number),
                }
            }
            Instruction::Syscall(SyscallKind::Syscall3) => {
                let syscall = pop!();
                let arg1 = pop!();
                let arg2 = pop!();
                let arg3 = pop!();
                match syscall {
                    0 => {
                        // Read
                        let fd = arg1 as usize;
                        let buf = arg2 as usize;
                        let count = arg3 as usize;
                        let buf = &mut bss[buf..buf + count];
                        fds[fd]
                            .reader
                            .as_mut()
                            .with_context(|| {
                                format!("File descriptor {} is not opened for reading", fd)
                            })?
                            .read_exact(buf)
                            .with_context(|| {
                                format!("Failed to read from file descriptor {}", fd)
                            })?;
                    }
                    1 => {
                        // Write
                        let fd = arg1 as usize;
                        let buf = arg2 as usize;
                        let count = arg3 as usize;
                        let buf = &bss[buf..buf + count];
                        fds[fd]
                            .writer
                            .as_mut()
                            .ok_or(RuntimeError(IOError))
                            .with_context(|| {
                                format!("File descriptor {} is not opened for writing", fd)
                            })?
                            .write_all(buf)
                            .with_context(|| {
                                format!("Failed to write to file descriptor {}", fd)
                            })?;
                        fds[fd]
                            .writer
                            .as_mut()
                            .with_context(|| {
                                format!("File descriptor {} is not opened for writing", fd)
                            })?
                            .flush()
                            .with_context(|| {
                                format!("Failed to flush writer for file descriptor {}", fd)
                            })?;
                    }
                    number => todo!("Implement syscall3 {}", number),
                }
            }
            #[allow(unused_variables)]
            Instruction::Syscall(SyscallKind::Syscall4) => {
                let syscall = pop!();
                let arg1 = pop!();
                let arg2 = pop!();
                let arg3 = pop!();
                let arg4 = pop!();
                match syscall {
                    number => todo!("Implement syscall4 {}", number),
                }
            }
            #[allow(unused_variables)]
            Instruction::Syscall(SyscallKind::Syscall5) => {
                let syscall = pop!();
                let arg1 = pop!();
                let arg2 = pop!();
                let arg3 = pop!();
                let arg4 = pop!();
                let arg5 = pop!();
                match syscall {
                    number => todo!("Implement syscall5 {}", number),
                }
            }
            #[allow(unused_variables)]
            Instruction::Syscall(SyscallKind::Syscall6) => {
                let syscall = pop!();
                let arg1 = pop!();
                let arg2 = pop!();
                let arg3 = pop!();
                let arg4 = pop!();
                let arg5 = pop!();
                let arg6 = pop!();
                match syscall {
                    number => todo!("Implement syscall6 {}", number),
                }
            }
            Instruction::Keyword(Keyword::While { .. }) => {}
            Instruction::Keyword(Keyword::Do { end_ip }) => {
                let a = pop!();
                if a == 0 {
                    ip = *end_ip + 1;
                    continue;
                }
            }
            Instruction::Keyword(Keyword::If { else_ip }) => {
                let a = pop!();
                if a == 0 {
                    ip = *else_ip + 1;
                    continue;
                }
            }
            Instruction::Keyword(Keyword::Else { end_ip, .. }) => {
                ip = *end_ip;
                continue;
            }
            Instruction::Keyword(Keyword::End { while_ip, .. }) => {
                if let Some(while_ip) = while_ip {
                    ip = *while_ip;
                    continue;
                }
            }
            Instruction::Push(val) => match val {
                Value::Int(i) => stack.push(*i),
                Value::Char(c) => stack.push((*c) as i64),
                Value::Str(s) => {
                    let len = s.as_bytes().len();
                    stack.push(len as i64);
                    stack.push(str_allocated as i64);
                    bss[str_allocated..str_allocated + len].copy_from_slice(s.as_bytes());
                    str_allocated += len;
                    if str_allocated > STR_CAPACITY {
                        return Err(RuntimeError(StringCapacityExceeded)).with_context(|| {
                            format!(
                                "String capacity exceeded: {} > {}",
                                str_allocated, STR_CAPACITY
                            )
                        });
                    }
                }
                Value::Ptr(_name) => todo!(),
            },
            Instruction::Intrinsic(intrinsic) => match intrinsic {
                Intrinsic::Panic => std::process::exit(1),
                Intrinsic::Print => {
                    let a = pop!();
                    println!("{}", a);
                }
                Intrinsic::Dup => {
                    let a = pop!();
                    stack.push(a);
                    stack.push(a);
                }
                Intrinsic::Mem => stack.push(STR_CAPACITY as i64),
                Intrinsic::Swap => {
                    let a = pop!();
                    let b = pop!();
                    stack.push(a);
                    stack.push(b);
                }
                Intrinsic::Drop => {
                    stack.pop();
                }
                Intrinsic::Over => {
                    let a = pop!();
                    let b = pop!();
                    stack.push(b);
                    stack.push(a);
                    stack.push(b);
                }
                Intrinsic::Drop2 => {
                    stack.pop();
                    stack.pop();
                }
                Intrinsic::Dup2 => {
                    let a = pop!();
                    let b = pop!();
                    stack.push(b);
                    stack.push(a);
                    stack.push(b);
                    stack.push(a);
                }
                #[allow(unreachable_patterns)]
                intrinsic => todo!("Implement intrinsic {}", intrinsic),
            },
            Instruction::Op(Op::Add) => {
                let a = pop!();
                let b = pop!();
                stack.push(a + b);
            }
            Instruction::Op(Op::Sub) => {
                let a = pop!();
                let b = pop!();
                stack.push(b - a);
            }
            Instruction::Op(Op::Mul) => {
                let a = pop!();
                let b = pop!();
                stack.push(a * b);
            }
            Instruction::Op(Op::Div) => {
                let a = pop!();
                let b = pop!();
                stack.push(b / a);
            }
            Instruction::Op(Op::DivMod) => {
                let a = pop!();
                let b = pop!();
                stack.push(b / a);
                stack.push(b % a);
            }
            Instruction::Op(Op::BitwiseAnd) => {
                let a = pop!();
                let b = pop!();
                stack.push(a & b);
            }
            Instruction::Op(Op::BitwiseOr) => {
                let a = pop!();
                let b = pop!();
                stack.push(a | b);
            }
            Instruction::Op(Op::BitwiseXor) => {
                let a = pop!();
                let b = pop!();
                stack.push(a ^ b);
            }
            Instruction::Op(Op::BitwiseNot) => {
                let a = pop!();
                stack.push(!a);
            }
            Instruction::Op(Op::Shl) => {
                let a = pop!();
                let b = pop!();
                stack.push(b << a);
            }
            Instruction::Op(Op::Shr) => {
                let a = pop!();
                let b = pop!();
                stack.push(b >> a);
            }
            Instruction::Op(Op::Eq) => {
                let a = pop!();
                let b = pop!();
                stack.push((a == b) as i64);
            }
            Instruction::Op(Op::Neq) => {
                let a = pop!();
                let b = pop!();
                stack.push((a != b) as i64);
            }
            Instruction::Op(Op::Lt) => {
                let a = pop!();
                let b = pop!();
                stack.push((b < a) as i64);
            }
            Instruction::Op(Op::Gt) => {
                let a = pop!();
                let b = pop!();
                stack.push((b > a) as i64);
            }
            Instruction::Op(Op::Lte) => {
                let a = pop!();
                let b = pop!();
                stack.push((b <= a) as i64);
            }
            Instruction::Op(Op::Gte) => {
                let a = pop!();
                let b = pop!();
                stack.push((b >= a) as i64);
            }
            Instruction::Op(Op::Store) => {
                let val = pop!() % 0xFF;
                let addr = pop!();
                if addr > (STR_CAPACITY + BSS_CAPACITY) as i64 {
                    return Err(RuntimeError(InvalidMemoryAccess)).with_context(|| {
                        format!(
                            "Invalid memory write: {:x} > {:x}",
                            addr,
                            STR_CAPACITY + BSS_CAPACITY
                        )
                    });
                }
                bss[addr as usize] = val as u8; // Take lower byte only
            }
            Instruction::Op(Op::Load) => {
                let addr = pop!();
                if addr > (STR_CAPACITY + BSS_CAPACITY) as i64 {
                    return Err(RuntimeError(InvalidMemoryAccess)).with_context(|| {
                        format!(
                            "Invalid memory read: {:x} > {:x}",
                            addr,
                            STR_CAPACITY + BSS_CAPACITY
                        )
                    });
                }
                stack.push(bss[addr as usize] as i64);
            }
            Instruction::Keyword(Keyword::Macro) => {
                return Err(RuntimeError(MacroNotExpanded))
                    .with_context(|| format!("Encountered macro definition at {}", ip))
            }
            Instruction::Name(name) => {
                return Err(RuntimeError(NameNotResolved))
                    .with_context(|| format!("Encountered unresolved name at {}: {}", ip, name));
            }

            #[allow(unreachable_patterns)]
            instruction => todo!("Implement instruction {:?}", instruction),
        }
        ip += 1;
    }
    log::log(Debug, "Sim exited successfully".into(), debug);
    Ok(())
}
