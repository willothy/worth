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

const STR_CAPACITY: usize = 640_000;
const ARGV_CAPACITY: usize = 640_000;
const BSS_CAPACITY: usize = 640_000;
const NULL_PTR_PADDING: usize = 1;
const STR_BUF_PTR: usize = NULL_PTR_PADDING;
const ARGV_BUF_PTR: usize = NULL_PTR_PADDING + STR_CAPACITY;
const MEM_BUF_PTR: usize = NULL_PTR_PADDING + STR_CAPACITY + ARGV_CAPACITY;
const MEM_LIMIT: usize = NULL_PTR_PADDING + STR_CAPACITY + ARGV_CAPACITY + BSS_CAPACITY;

pub struct SimulationState {
    pub stack: Vec<i64>,
    pub memory: Vec<u8>,
    pub fds: Vec<BinaryIO>,
    pub argc: usize,
    pub str_allocated: usize,
    pub ip: usize,
}

pub fn simulate(program: &Program, mut opt: SimulatorOptions) -> Result<()> {
    let mut debug = opt.debug;
    let Program {
        instructions: program,
        name: program_name,
        base_path,
        ..
    } = program;

    let mut state = SimulationState {
        stack: Vec::new(),
        memory: vec![0; MEM_LIMIT],
        fds: BinaryIO::stdio(),
        argc: 0,
        str_allocated: 0,
        ip: 0,
    };

    let mut argv = opt.sim_args;
    argv.insert(
        0,
        base_path.join(program_name).to_str().unwrap().to_string(),
    );

    // Allocate strings and push arguments (char** argv) onto the stack
    for arg in argv.iter() {
        let mut arg_bytes = arg.as_bytes().to_vec();
        arg_bytes.push(0); // null-terminate
        let len = arg_bytes.len();
        let arg_ptr = STR_BUF_PTR + state.str_allocated;
        state.memory[arg_ptr..arg_ptr + len].copy_from_slice(&arg_bytes);
        state.str_allocated += len;

        if arg_ptr > STR_CAPACITY {
            return Err(RuntimeError(StringCapacityExceeded)).with_context(|| {
                format!("String capacity exceeded: {} > {}", arg_ptr, STR_CAPACITY)
            });
        }

        let argv_ptr = ARGV_BUF_PTR + (state.argc * 8);
        // copy argv_ptr to bss[argv_ptr..argv_ptr + 8]
        state.memory[argv_ptr] = (arg_ptr >> 56) as u8;
        state.memory[argv_ptr + 1] = (arg_ptr >> 48) as u8;
        state.memory[argv_ptr + 2] = (arg_ptr >> 40) as u8;
        state.memory[argv_ptr + 3] = (arg_ptr >> 32) as u8;
        state.memory[argv_ptr + 4] = (arg_ptr >> 24) as u8;
        state.memory[argv_ptr + 5] = (arg_ptr >> 16) as u8;
        state.memory[argv_ptr + 6] = (arg_ptr >> 8) as u8;
        state.memory[argv_ptr + 7] = arg_ptr as u8;

        state.argc += 1;

        if state.argc * 8 > ARGV_CAPACITY {
            return Err(RuntimeError(BufferOverflow)).with_context(|| {
                format!(
                    "Argv buffer overflow: {} > {}",
                    state.argc * 8,
                    ARGV_CAPACITY
                )
            });
        }
    }

    if let Some(breakpoint) = opt.breakpoint {
        log::log(
            Info,
            format!("Breakpoint at instruction {}", breakpoint),
            debug,
        );
    }

    while state.ip < program.len() {
        if let Some(breakpoint) = opt.breakpoint {
            if breakpoint == state.ip {
                log::log(Info, format!("Breakpoint reached"), debug);
                opt.step = true;
            }
        }
        let inst = &program[state.ip];
        sim_instruction(inst, &mut state)?;

        if opt.step || opt.debug {
            println!("{}: {:?}", state.ip, inst);
            println!("Stack: {:?}", state.stack);
        }
        if opt.step {
            let mut cmd = String::new();
            std::io::stdin().read_line(&mut cmd).unwrap();
            match cmd.trim() {
                "c" => opt.step = false,
                "d" => debug = !debug,
                "q" => break,
                _ => {}
            }
        }
    }
    log::log(Debug, "Sim exited successfully".into(), debug);
    Ok(())
}

pub fn sim_instruction(inst: &Instruction, state: &mut SimulationState) -> Result<()> {
    let SimulationState {
        stack,
        memory: bss,
        fds,
        argc,
        str_allocated,
        ip,
    } = state;
    macro_rules! pop {
        () => {
            stack
                .pop()
                .ok_or(RuntimeError(StackUnderflow))
                .with_context(|| format!("Stack underflow at instruction {}", ip))?
        };
    }

    match &inst.kind {
        InstructionKind::Push(val) => match val {
            Value::Int(i) => stack.push(*i),
            Value::Char(c) => stack.push((*c) as i64),
            Value::Bool(b) => stack.push(*b as i64),
            Value::Str(s) => {
                let len = s.as_bytes().len();
                stack.push(len as i64);
                let str_buf_end = STR_BUF_PTR + *str_allocated;
                stack.push(str_buf_end as i64);
                bss[str_buf_end..str_buf_end + len].copy_from_slice(s.as_bytes());
                *str_allocated += len + 1;
                if str_buf_end > STR_CAPACITY {
                    return Err(RuntimeError(StringCapacityExceeded)).with_context(|| {
                        format!(
                            "String capacity exceeded: {} > {}",
                            str_buf_end, STR_CAPACITY
                        )
                    });
                }
            }
            Value::Ptr(_name) => todo!(),
        },
        InstructionKind::Syscall(SyscallKind::Syscall0) => {
            let syscall = pop!();
            match syscall {
                number => todo!("Implement syscall0 {}", number),
            }
        }
        InstructionKind::Syscall(SyscallKind::Syscall1) => {
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
        InstructionKind::Syscall(SyscallKind::Syscall2) => {
            let syscall = pop!();
            let arg1 = pop!();
            let arg2 = pop!();
            match syscall {
                number => todo!("Implement syscall2 {}", number),
            }
        }
        InstructionKind::Syscall(SyscallKind::Syscall3) => {
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
                    //let mut tmp_buf = String::new();
                    let buf = &mut bss[buf..buf + count];
                    let bytes_read = fds[fd]
                        .reader
                        .as_mut()
                        .with_context(|| {
                            format!("File descriptor {} is not opened for reading", fd)
                        })?
                        .read(buf)
                        .with_context(|| format!("Failed to read from file descriptor {}", fd))?;
                    stack.push(bytes_read as i64);
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
                        .with_context(|| format!("Failed to write to file descriptor {}", fd))?;
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
                    stack.push(count as i64);
                }
                number => todo!("Implement syscall3 {}", number),
            }
        }
        #[allow(unused_variables)]
        InstructionKind::Syscall(SyscallKind::Syscall4) => {
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
        InstructionKind::Syscall(SyscallKind::Syscall5) => {
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
        InstructionKind::Syscall(SyscallKind::Syscall6) => {
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
        InstructionKind::Keyword(Keyword::While { .. }) => {}
        InstructionKind::Keyword(Keyword::Do { end_ip }) => {
            let a = pop!();
            if a == 0 {
                *ip = *end_ip + 1;
                return Ok(());
            }
        }
        InstructionKind::Keyword(Keyword::If { .. }) => {}
        InstructionKind::Keyword(Keyword::Elif {
            end_ip: else_ip, ..
        }) => {
            *ip = *else_ip;
            return Ok(());
        }
        InstructionKind::Keyword(Keyword::Else { end_ip, .. }) => {
            *ip = *end_ip;
            return Ok(());
        }
        InstructionKind::Keyword(Keyword::End { while_ip, .. }) => {
            if let Some(while_ip) = while_ip {
                *ip = *while_ip;
                return Ok(());
            }
        }
        InstructionKind::Intrinsic(intrinsic) => match intrinsic {
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
            Intrinsic::Mem => stack.push(MEM_BUF_PTR as i64),
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
            Intrinsic::Argc => {
                stack.push(*argc as i64);
            }
            Intrinsic::Argv => {
                stack.push(ARGV_BUF_PTR as i64);
            }
            Intrinsic::CastPtr => {}
            Intrinsic::CastInt => {}
            #[allow(unreachable_patterns)]
            intrinsic => todo!("Implement intrinsic {}", intrinsic),
        },
        InstructionKind::Op(Op::Add) => {
            let a = pop!();
            let b = pop!();
            stack.push(a + b);
        }
        InstructionKind::Op(Op::Sub) => {
            let a = pop!();
            let b = pop!();
            stack.push(b - a);
        }
        InstructionKind::Op(Op::Mul) => {
            let a = pop!();
            let b = pop!();
            stack.push(a * b);
        }
        InstructionKind::Op(Op::Div) => {
            let a = pop!();
            let b = pop!();
            stack.push(b / a);
        }
        InstructionKind::Op(Op::Mod) => {
            let a = pop!();
            let b = pop!();
            stack.push(b % a);
        }
        InstructionKind::Op(Op::DivMod) => {
            let a = pop!();
            let b = pop!();
            stack.push(b / a);
            stack.push(b % a);
        }
        InstructionKind::Op(Op::BitwiseAnd) => {
            let a = pop!();
            let b = pop!();
            stack.push(a & b);
        }
        InstructionKind::Op(Op::BitwiseOr) => {
            let a = pop!();
            let b = pop!();
            stack.push(a | b);
        }
        InstructionKind::Op(Op::BitwiseXor) => {
            let a = pop!();
            let b = pop!();
            stack.push(a ^ b);
        }
        InstructionKind::Op(Op::BitwiseNot) => {
            let a = pop!();
            stack.push(!a);
        }
        InstructionKind::Op(Op::Shl) => {
            let a = pop!();
            let b = pop!();
            stack.push(b << a);
        }
        InstructionKind::Op(Op::Shr) => {
            let a = pop!();
            let b = pop!();
            stack.push(b >> a);
        }
        InstructionKind::Op(Op::Eq) => {
            let a = pop!();
            let b = pop!();
            stack.push((a == b) as i64);
        }
        InstructionKind::Op(Op::Neq) => {
            let a = pop!();
            let b = pop!();
            stack.push((a != b) as i64);
        }
        InstructionKind::Op(Op::Lt) => {
            let a = pop!();
            let b = pop!();
            stack.push((b < a) as i64);
        }
        InstructionKind::Op(Op::Gt) => {
            let a = pop!();
            let b = pop!();
            stack.push((b > a) as i64);
        }
        InstructionKind::Op(Op::Lte) => {
            let a = pop!();
            let b = pop!();
            stack.push((b <= a) as i64);
        }
        InstructionKind::Op(Op::Gte) => {
            let a = pop!();
            let b = pop!();
            stack.push((b >= a) as i64);
        }
        InstructionKind::Op(Op::Store) => {
            let val = pop!() % 0xFF;
            let addr = pop!();
            if addr > MEM_LIMIT as i64 {
                return Err(RuntimeError(InvalidMemoryAccess)).with_context(|| {
                    format!("Invalid memory write: {:x} > {:x}", addr, MEM_LIMIT)
                });
            }
            bss[addr as usize] = val as u8; // Take lower byte only
        }
        InstructionKind::Op(Op::Load) => {
            let addr = pop!();
            if addr > MEM_LIMIT as i64 {
                return Err(RuntimeError(InvalidMemoryAccess)).with_context(|| {
                    format!(
                        "Invalid memory read at {}: {:x} > {:x}",
                        ip, addr, MEM_LIMIT
                    )
                });
            }
            stack.push(bss[addr as usize] as i64);
        }
        InstructionKind::Op(Op::Store64) => {
            let val = pop!();
            let addr = pop!();
            if addr > MEM_LIMIT as i64 {
                return Err(RuntimeError(InvalidMemoryAccess)).with_context(|| {
                    format!("Invalid memory write: {:x} > {:x}", addr, MEM_LIMIT)
                });
            }
            // Store 8 bytes of value to the address
            bss[addr as usize] = (val >> 56) as u8;
            bss[addr as usize + 1] = (val >> 48) as u8;
            bss[addr as usize + 2] = (val >> 40) as u8;
            bss[addr as usize + 3] = (val >> 32) as u8;
            bss[addr as usize + 4] = (val >> 24) as u8;
            bss[addr as usize + 5] = (val >> 16) as u8;
            bss[addr as usize + 6] = (val >> 8) as u8;
            bss[addr as usize + 7] = val as u8;
        }
        InstructionKind::Op(Op::Load64) => {
            let addr = pop!();
            if addr > MEM_LIMIT as i64 {
                return Err(RuntimeError(InvalidMemoryAccess)).with_context(|| {
                    format!(
                        "Invalid memory read at {}: {:x} > {:x}",
                        ip, addr, MEM_LIMIT
                    )
                });
            }
            // Read 8 bytes of value from the address
            let val = (bss[addr as usize] as i64) << 56
                | (bss[addr as usize + 1] as i64) << 48
                | (bss[addr as usize + 2] as i64) << 40
                | (bss[addr as usize + 3] as i64) << 32
                | (bss[addr as usize + 4] as i64) << 24
                | (bss[addr as usize + 5] as i64) << 16
                | (bss[addr as usize + 6] as i64) << 8
                | bss[addr as usize + 7] as i64;
            stack.push(val);
        }
        InstructionKind::Keyword(Keyword::Macro) => {
            return Err(RuntimeError(MacroNotExpanded))
                .with_context(|| format!("Encountered macro definition at {}", ip))
        }
        InstructionKind::Name(name) => {
            return Err(RuntimeError(NameNotResolved))
                .with_context(|| format!("Encountered unresolved name at {}: {}", ip, name));
        }

        #[allow(unreachable_patterns)]
        instruction => todo!("Implement instruction {:?}", instruction),
    }
    *ip += 1;
    Ok(())
}
