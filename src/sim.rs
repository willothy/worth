use std::io::{self, BufRead, BufReader, Write};

use crate::{cli::SimulatorOptions, codegen::intrinsics::Intrinsic, instruction::*};

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

pub fn simulate(program: &Program, opt: SimulatorOptions) -> Result<(), String> {
    let Program {
        instructions: program,
        ..
    } = program;
    //println!("Simulating program: {:#?}", program);
    let mut stack = Vec::new();
    let mut bss: Vec<u8> = vec![0; crate::codegen::MEM_CAPACITY];
    let mut fds: Vec<BinaryIO> = BinaryIO::stdio();
    let debug = opt.debug;

    let mut ip = 0;
    while ip < program.len() {
        let inst = &program[ip];
        if debug {
            println!("ip: {:?} inst: {:?}", ip, inst);
            println!("stack: {:?}", stack);
            println!("bss[0..10]: {:?}", &bss[0..10]);
            println!("-------------------------------------------");
        }

        match &inst {
            Instruction::Syscall(SyscallKind::Syscall0) => {
                let syscall = stack.pop().unwrap();
                match syscall {
                    number => todo!("Implement syscall0 {}", number),
                }
            }
            Instruction::Syscall(SyscallKind::Syscall1) => {
                let syscall = stack.pop().unwrap();
                let arg1 = stack.pop().unwrap();
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
                let syscall = stack.pop().unwrap();
                let arg1 = stack.pop().unwrap();
                let arg2 = stack.pop().unwrap();
                match syscall {
                    number => todo!("Implement syscall2 {}", number),
                }
            }
            Instruction::Syscall(SyscallKind::Syscall3) => {
                let syscall = stack.pop().unwrap();
                let arg1 = stack.pop().unwrap();
                let arg2 = stack.pop().unwrap();
                let arg3 = stack.pop().unwrap();
                match syscall {
                    0 => {
                        // Read
                        let fd = arg1 as usize;
                        let buf = arg2 as usize;
                        let count = arg3 as usize;
                        let buf = &mut bss[buf..buf + count];
                        fds[fd].reader.as_mut().unwrap().read_exact(buf).unwrap();
                    }
                    1 => {
                        // Write
                        let fd = arg1 as usize;
                        let buf = arg2 as usize;
                        let count = arg3 as usize;
                        let buf = &bss[buf..buf + count];
                        fds[fd].writer.as_mut().unwrap().write_all(buf).unwrap();
                        fds[fd].writer.as_mut().unwrap().flush().unwrap();
                    }
                    number => todo!("Implement syscall3 {}", number),
                }
            }
            #[allow(unused_variables)]
            Instruction::Syscall(SyscallKind::Syscall4) => {
                let syscall = stack.pop().unwrap();
                let arg1 = stack.pop().unwrap();
                let arg2 = stack.pop().unwrap();
                let arg3 = stack.pop().unwrap();
                let arg4 = stack.pop().unwrap();
                match syscall {
                    number => todo!("Implement syscall4 {}", number),
                }
            }
            #[allow(unused_variables)]
            Instruction::Syscall(SyscallKind::Syscall5) => {
                let syscall = stack.pop().unwrap();
                let arg1 = stack.pop().unwrap();
                let arg2 = stack.pop().unwrap();
                let arg3 = stack.pop().unwrap();
                let arg4 = stack.pop().unwrap();
                let arg5 = stack.pop().unwrap();
                match syscall {
                    number => todo!("Implement syscall5 {}", number),
                }
            }
            #[allow(unused_variables)]
            Instruction::Syscall(SyscallKind::Syscall6) => {
                let syscall = stack.pop().unwrap();
                let arg1 = stack.pop().unwrap();
                let arg2 = stack.pop().unwrap();
                let arg3 = stack.pop().unwrap();
                let arg4 = stack.pop().unwrap();
                let arg5 = stack.pop().unwrap();
                let arg6 = stack.pop().unwrap();
                match syscall {
                    number => todo!("Implement syscall6 {}", number),
                }
            }
            Instruction::Keyword(Keyword::While { .. }) => {}
            Instruction::Keyword(Keyword::Do { end_ip }) => {
                let a = stack.pop().unwrap();
                if a == 0 {
                    ip = *end_ip + 1;
                    continue;
                }
            }
            Instruction::Keyword(Keyword::If { else_ip }) => {
                let a = stack.pop().unwrap();
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
                Value::Ptr(_name) => todo!(),
            },
            Instruction::Intrinsic(intrinsic) => match intrinsic {
                Intrinsic::Panic => std::process::exit(1),
                Intrinsic::Dump => {
                    let a = stack.pop().unwrap();
                    println!("{}", a);
                }
                Intrinsic::Dup => {
                    let a = stack.pop().unwrap();
                    stack.push(a);
                    stack.push(a);
                }
                Intrinsic::Mem => stack.push(0),
                Intrinsic::Swap => {
                    let a = stack.pop().unwrap();
                    let b = stack.pop().unwrap();
                    stack.push(a);
                    stack.push(b);
                }
                Intrinsic::Drop => {
                    stack.pop();
                }
                Intrinsic::Over => {
                    let a = stack.pop().unwrap();
                    let b = stack.pop().unwrap();
                    stack.push(b);
                    stack.push(a);
                    stack.push(b);
                }
                Intrinsic::Drop2 => {
                    stack.pop();
                    stack.pop();
                }
                Intrinsic::Dup2 => {
                    let a = stack.pop().unwrap();
                    let b = stack.pop().unwrap();
                    stack.push(b);
                    stack.push(a);
                    stack.push(b);
                    stack.push(a);
                }
                #[allow(unreachable_patterns)]
                intrinsic => todo!("Implement intrinsic {}", intrinsic),
            },
            Instruction::Op(Op::Add) => {
                let a = stack.pop().unwrap();
                let b = stack.pop().unwrap();
                stack.push(a + b);
            }
            Instruction::Op(Op::Sub) => {
                let a = stack.pop().unwrap();
                let b = stack.pop().unwrap();
                stack.push(b - a);
            }
            Instruction::Op(Op::Mul) => {
                let a = stack.pop().unwrap();
                let b = stack.pop().unwrap();
                stack.push(a * b);
            }
            Instruction::Op(Op::Div) => {
                let a = stack.pop().unwrap();
                let b = stack.pop().unwrap();
                stack.push(b / a);
            }
            Instruction::Op(Op::Mod) => {
                let a = stack.pop().unwrap();
                let b = stack.pop().unwrap();
                stack.push(a % b);
            }
            Instruction::Op(Op::BitwiseAnd) => {
                let a = stack.pop().unwrap();
                let b = stack.pop().unwrap();
                stack.push(a & b);
            }
            Instruction::Op(Op::BitwiseOr) => {
                let a = stack.pop().unwrap();
                let b = stack.pop().unwrap();
                stack.push(a | b);
            }
            Instruction::Op(Op::BitwiseXor) => {
                let a = stack.pop().unwrap();
                let b = stack.pop().unwrap();
                stack.push(a ^ b);
            }
            Instruction::Op(Op::BitwiseNot) => {
                let a = stack.pop().unwrap();
                stack.push(!a);
            }
            Instruction::Op(Op::Shl) => {
                let a = stack.pop().unwrap();
                let b = stack.pop().unwrap();
                stack.push(b << a);
            }
            Instruction::Op(Op::Shr) => {
                let a = stack.pop().unwrap();
                let b = stack.pop().unwrap();
                stack.push(b >> a);
            }
            Instruction::Op(Op::Eq) => {
                let a = stack.pop().unwrap();
                let b = stack.pop().unwrap();
                stack.push((a == b) as i64);
            }
            Instruction::Op(Op::Neq) => {
                let a = stack.pop().unwrap();
                let b = stack.pop().unwrap();
                stack.push((a != b) as i64);
            }
            Instruction::Op(Op::Lt) => {
                let a = stack.pop().unwrap();
                let b = stack.pop().unwrap();
                stack.push((b < a) as i64);
            }
            Instruction::Op(Op::Gt) => {
                let a = stack.pop().unwrap();
                let b = stack.pop().unwrap();
                stack.push((b > a) as i64);
            }
            Instruction::Op(Op::Lte) => {
                let a = stack.pop().unwrap();
                let b = stack.pop().unwrap();
                stack.push((b <= a) as i64);
            }
            Instruction::Op(Op::Gte) => {
                let a = stack.pop().unwrap();
                let b = stack.pop().unwrap();
                stack.push((b >= a) as i64);
            }
            Instruction::Op(Op::Store) => {
                let val = stack.pop().unwrap() % 0xFF;
                let addr = stack.pop().unwrap();
                bss[addr as usize] = val as u8; // Take lower byte only
            }
            Instruction::Op(Op::Load) => {
                let a = stack.pop().unwrap();
                stack.push(bss[a as usize] as i64);
            }
            Instruction::Keyword(Keyword::Macro) => {
                unreachable!("Macro should be expanded before simulation")
            }
            Instruction::Name(name) => {
                unreachable!("Name {} should be expanded before simulation", name)
            }

            #[allow(unreachable_patterns)]
            instruction => todo!("Implement instruction {:?}", instruction),
        }
        ip += 1;
    }
    Ok(())
}
