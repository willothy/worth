use crate::{cli::SimulatorOptions, codegen::intrinsics::Intrinsic, instruction::*};

pub fn simulate(program: &Program, opt: SimulatorOptions) -> Result<(), String> {
    let Program {
        instructions: program,
        ..
    } = program;
    let mut stack = Vec::new();
    let mut bss = vec![0; crate::codegen::MEM_CAPACITY];
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
            Instruction::While { .. } => {}
            Instruction::Do { end_ip } => {
                let a = stack.pop().unwrap();
                if a == 0 {
                    ip = *end_ip + 1;
                    continue;
                }
            }
            Instruction::If { else_ip } => {
                let a = stack.pop().unwrap();
                if a == 0 {
                    ip = *else_ip + 1;
                    continue;
                }
            }
            Instruction::Else { end_ip, .. } => {
                ip = *end_ip;
                continue;
            }
            Instruction::End { while_ip, .. } => {
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
                Intrinsic::Panic => return Err(format!("Panicked at {:?} ({})", inst, ip)),
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
                #[allow(unreachable_patterns)]
                intrinsic => todo!("Implement intrinsic {}", intrinsic),
            },
            Instruction::Add => {
                let a = stack.pop().unwrap();
                let b = stack.pop().unwrap();
                stack.push(a + b);
            }
            Instruction::Sub => {
                let a = stack.pop().unwrap();
                let b = stack.pop().unwrap();
                stack.push(b - a);
            }
            Instruction::Mul => {
                let a = stack.pop().unwrap();
                let b = stack.pop().unwrap();
                stack.push(a * b);
            }
            Instruction::Div => {
                let a = stack.pop().unwrap();
                let b = stack.pop().unwrap();
                stack.push(b / a);
            }
            Instruction::Mod => {
                let a = stack.pop().unwrap();
                let b = stack.pop().unwrap();
                stack.push(a % b);
            }
            Instruction::And => {
                let a = stack.pop().unwrap();
                let b = stack.pop().unwrap();
                stack.push(a & b);
            }
            Instruction::Or => {
                let a = stack.pop().unwrap();
                let b = stack.pop().unwrap();
                stack.push(a | b);
            }
            Instruction::Xor => {
                let a = stack.pop().unwrap();
                let b = stack.pop().unwrap();
                stack.push(a ^ b);
            }
            Instruction::Not => {
                let a = stack.pop().unwrap();
                stack.push(!a);
            }
            Instruction::Shl => {
                let a = stack.pop().unwrap();
                let b = stack.pop().unwrap();
                stack.push(a << b);
            }
            Instruction::Shr => {
                let a = stack.pop().unwrap();
                let b = stack.pop().unwrap();
                stack.push(a >> b);
            }
            Instruction::Eq => {
                let a = stack.pop().unwrap();
                let b = stack.pop().unwrap();
                stack.push((a == b) as i64);
            }
            Instruction::Neq => {
                let a = stack.pop().unwrap();
                let b = stack.pop().unwrap();
                stack.push((a != b) as i64);
            }
            Instruction::Lt => {
                let a = stack.pop().unwrap();
                let b = stack.pop().unwrap();
                stack.push((a < b) as i64);
            }
            Instruction::Gt => {
                let a = stack.pop().unwrap();
                let b = stack.pop().unwrap();
                stack.push((a > b) as i64);
            }
            Instruction::Lte => {
                let a = stack.pop().unwrap();
                let b = stack.pop().unwrap();
                stack.push((a <= b) as i64);
            }
            Instruction::Gte => {
                let a = stack.pop().unwrap();
                let b = stack.pop().unwrap();
                stack.push((a >= b) as i64);
            }
            Instruction::Macro => unreachable!("Macro should be expanded before simulation"),
            Instruction::Name(name) => {
                unreachable!("Name {} should be expanded before simulation", name)
            }
            Instruction::Store => {
                let val = stack.pop().unwrap();
                let addr = stack.pop().unwrap();
                bss[addr as usize] = val;
            }
            Instruction::Load => {
                let a = stack.pop().unwrap();
                stack.push(bss[a as usize]);
            }
            #[allow(unreachable_patterns)]
            instruction => todo!("Implement instruction {:?}", instruction),
        }
        ip += 1;
    }
    Ok(())
}
