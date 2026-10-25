use std::collections::HashMap;
use std::io::{Read, Write};
use crate::framebuffer::{draw_framebuffer, init_framebuffer};

mod framebuffer;

#[derive(PartialEq, Eq, Clone, Copy)]
enum Instruction {
    AddImmediate(i32),
    AddDirect(u32),
    AddIndirect(u32),
    SubtractImmediate(i32),
    SubtractDirect(u32),
    SubtractIndirect(u32),
    MultiplyImmediate(i32),
    MultiplyDirect(u32),
    MultiplyIndirect(u32),
    DivideImmediate(i32),
    DivideDirect(u32),
    DivideIndirect(u32),
    LoadImmediate(i32),
    LoadDirect(u32),
    LoadIndirect(u32),
    StoreDirect(u32),
    StoreIndirect(u32),
    ReadDirect(u32),
    ReadIndirect(u32),
    WriteImmediate(i32),
    WriteDirect(u32),
    WriteIndirect(u32),
    WriteCharImmediate(i32),
    WriteCharDirect(u32),
    WriteCharIndirect(u32),
    ReadCharDirect(u32),
    ReadCharIndirect(u32),
    Jump(usize),
    JumpIndirect(u32),
    JumpIfZero(usize),
    JumpIfGreaterThanZero(usize),
    Halt,
    Debug,
}

enum ParameterType {
    Immediate,
    Direct,
    Indirect,
    Label
}

pub const MEMORY_SIZE: usize = 17000000;
pub const PRELOAD_OFFSET: usize = 8388608;
const REDRAW_INTERVAL: i32 = 100_000;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let mut file_path = None;
    let mut preload_path = None;
    let mut enable_graphics = false;

    let mut arg_idx = 1;
    while arg_idx < args.len() {
        if args[arg_idx] == "--gfx" {
            enable_graphics = true;
            arg_idx += 1;
        } else if args[arg_idx] == "--preload" && arg_idx + 1 < args.len() {
            preload_path = Some(args[arg_idx + 1].clone());
            arg_idx += 2;
        } else if !args[arg_idx].starts_with("--") && file_path.is_none() {
            file_path = Some(args[arg_idx].clone());
            arg_idx += 1;
        } else {
            arg_idx += 1;
        }
    }

    let file_path = file_path.expect("Usage: program <file> [--gfx] [--preload <path>]");
    let file_content = std::fs::read_to_string(file_path).unwrap();
    let mut instructions: Vec<Instruction> = Vec::with_capacity(file_content.lines().count());
    let mut instructions_executed: u64 = 0;

    let mut window = if enable_graphics {
        Some(init_framebuffer())
    } else {
        None
    };
    let mut instructions_since_last_draw: i32 = 0;

    let file_content_split = file_content.split("\n");

    // pre-process labels
    let mut labels: HashMap<String, usize> = HashMap::new();
    let mut instruction_index = 0;
    for line in file_content_split.clone() {
        if line.is_empty() || line.chars().next().unwrap() == '/' {
            continue;
        }

        let mut accumulated: String = "".to_owned();
        for character in line.chars() {
            if character == '/' {
                accumulated = "".to_string();
                break;
            }
            if character == ':' {
                labels.insert(accumulated.trim().to_string(), instruction_index);
                accumulated = "".to_string();
                break;
            }
            accumulated += character.to_string().as_str();
        }
        instruction_index += 1;
    }

    for line in file_content_split {
        if line.is_empty() || line.chars().next().unwrap() == '/' {
            continue;
        }
        let mut accumulated: String = "".to_owned();
        for character in line.chars() {
            if character == '/' {
                break;
            }
            if character == ':' {
                accumulated = "".to_string();
                continue;
            }
            accumulated += character.to_string().as_str();
        }

        let instruction_parts = accumulated.trim().split_whitespace().collect::<Vec<&str>>();
        if instruction_parts.is_empty() {
            continue;
        }
        let operation = instruction_parts[0];
        let instruction = match operation {
            "halt" => Instruction::Halt,
            "debug" => Instruction::Debug,
            _ => {
                let parameter = instruction_parts[1];
                let parameter_type = match parameter.chars().nth(0).unwrap() {
                    '0'..='9' => ParameterType::Direct,
                    '^' => ParameterType::Indirect,
                    '=' => ParameterType::Immediate,
                    _ => ParameterType::Label,
                };

                match operation {
                    "add" => match parameter_type {
                        ParameterType::Direct => Instruction::AddDirect(parameter.parse::<u32>().unwrap()),
                        ParameterType::Indirect => Instruction::AddIndirect(parameter[1..].parse::<u32>().unwrap()),
                        ParameterType::Immediate => Instruction::AddImmediate(parameter[1..].parse::<i32>().unwrap()),
                        ParameterType::Label => panic!("Label not supported for add operation"),
                    },
                    "sub" => match parameter_type {
                        ParameterType::Direct => Instruction::SubtractDirect(parameter.parse::<u32>().unwrap()),
                        ParameterType::Indirect => Instruction::SubtractIndirect(parameter[1..].parse::<u32>().unwrap()),
                        ParameterType::Immediate => Instruction::SubtractImmediate(parameter[1..].parse::<i32>().unwrap()),
                        ParameterType::Label => panic!("Label not supported for sub operation"),
                    },
                    "mult" => match parameter_type {
                        ParameterType::Direct => Instruction::MultiplyDirect(parameter.parse::<u32>().unwrap()),
                        ParameterType::Indirect => Instruction::MultiplyIndirect(parameter[1..].parse::<u32>().unwrap()),
                        ParameterType::Immediate => Instruction::MultiplyImmediate(parameter[1..].parse::<i32>().unwrap()),
                        ParameterType::Label => panic!("Label not supported for mult operation"),
                    },
                    "div" => match parameter_type {
                        ParameterType::Direct => Instruction::DivideDirect(parameter.parse::<u32>().unwrap()),
                        ParameterType::Indirect => Instruction::DivideIndirect(parameter[1..].parse::<u32>().unwrap()),
                        ParameterType::Immediate => Instruction::DivideImmediate(parameter[1..].parse::<i32>().unwrap()),
                        ParameterType::Label => panic!("Label not supported for div operation"),
                    },
                    "load" => match parameter_type {
                        ParameterType::Direct => Instruction::LoadDirect(parameter.parse::<u32>().unwrap()),
                        ParameterType::Indirect => Instruction::LoadIndirect(parameter[1..].parse::<u32>().unwrap()),
                        ParameterType::Immediate => {
                            let val = if let Ok(n) = parameter[1..].parse::<i32>() {
                                n
                            } else if let Some(target) = labels.get(&parameter[1..]) {
                                *target as i32
                            } else {
                                panic!("Unknown immediate or label: {}", parameter);
                            };
                            Instruction::LoadImmediate(val)
                        }
                        ParameterType::Label => panic!("Label not supported for load operation"),
                    },
                    "store" => match parameter_type {
                        ParameterType::Direct => Instruction::StoreDirect(parameter.parse::<u32>().unwrap()),
                        ParameterType::Indirect => Instruction::StoreIndirect(parameter[1..].parse::<u32>().unwrap()),
                        _ => panic!("Argument not supported for store operation")
                    },
                    "read" => match parameter_type {
                        ParameterType::Direct => Instruction::ReadDirect(parameter.parse::<u32>().unwrap()),
                        ParameterType::Indirect => Instruction::ReadIndirect(parameter[1..].parse::<u32>().unwrap()),
                        _ => panic!("Argument not supported for read operation")
                    },
                    "write" => match parameter_type {
                        ParameterType::Direct => Instruction::WriteDirect(parameter.parse::<u32>().unwrap()),
                        ParameterType::Indirect => Instruction::WriteIndirect(parameter[1..].parse::<u32>().unwrap()),
                        ParameterType::Immediate => Instruction::WriteImmediate(parameter[1..].parse::<i32>().unwrap()),
                        ParameterType::Label => panic!("Label not supported for write operation"),
                    },
                    "writec" => match parameter_type {
                        ParameterType::Direct => Instruction::WriteCharDirect(parameter.parse::<u32>().unwrap()),
                        ParameterType::Indirect => Instruction::WriteCharIndirect(parameter[1..].parse::<u32>().unwrap()),
                        ParameterType::Immediate => Instruction::WriteCharImmediate(parameter[1..].parse::<i32>().unwrap()),
                        ParameterType::Label => panic!("Label not supported for writec operation"),
                    },
                    "readc" => match parameter_type {
                        ParameterType::Direct => Instruction::ReadCharDirect(parameter.parse::<u32>().unwrap()),
                        ParameterType::Indirect => Instruction::ReadCharIndirect(parameter[1..].parse::<u32>().unwrap()),
                        _ => panic!("Argument not supported for readc operation")
                    },
                    "jump" => match parameter_type {
                        ParameterType::Label => Instruction::Jump(*labels.get(parameter).expect("Unknown label")),
                        ParameterType::Direct => Instruction::Jump(parameter.parse::<usize>().unwrap()),
                        ParameterType::Indirect => Instruction::JumpIndirect(parameter[1..].parse::<u32>().unwrap()),
                        _ => panic!("Argument not supported for jump operation")
                    },
                    "jzero" => match parameter_type {
                        ParameterType::Label => Instruction::JumpIfZero(*labels.get(parameter).expect("Unknown label")),
                        ParameterType::Direct => Instruction::JumpIfZero(parameter.parse::<usize>().unwrap()),
                        _ => panic!("Argument not supported for jzero operation")
                    },
                    "jgtz" => match parameter_type {
                        ParameterType::Label => Instruction::JumpIfGreaterThanZero(*labels.get(parameter).expect("Unknown label")),
                        ParameterType::Direct => Instruction::JumpIfGreaterThanZero(parameter.parse::<usize>().unwrap()),
                        _ => panic!("Argument not supported for jgtz operation")
                    },
                    _ => {
                        println!("Invalid instruction: {}", operation);
                        std::process::exit(1);
                    }
                }
            }
        };

        instructions.push(instruction);
    }

    let mut memory = vec![0i32; MEMORY_SIZE].into_boxed_slice();
    if let Some(preload_file) = preload_path {
        let preload_bytes = std::fs::read(&preload_file).expect("Failed to read preload file");
        println!("Loaded preload file '{}' ({} bytes) at cell {}", preload_file, preload_bytes.len(), PRELOAD_OFFSET);
        memory[PRELOAD_OFFSET - 1] = preload_bytes.len() as i32;
        for (i, byte) in preload_bytes.iter().enumerate() {
            memory[PRELOAD_OFFSET + i] = *byte as i32;
        }
    }

    let mut highest_accessed: usize = 0;
    let mut program_counter: usize = 0;
    let mut input = String::new();
    let stdin = std::io::stdin();
    while program_counter < instructions.len() {
        let instruction = instructions[program_counter];
        if instruction == Instruction::Halt {
            break;
        }
        program_counter += 1;
        match instruction {
            Instruction::AddImmediate(val) => memory[0] = memory[0].wrapping_add(val),
            Instruction::AddDirect(addr) => {
                highest_accessed = highest_accessed.max(addr as usize + 1);
                memory[0] = memory[0].wrapping_add(memory[addr as usize]);
            }
            Instruction::AddIndirect(addr) => {
                let target = memory[addr as usize] as usize;
                highest_accessed = highest_accessed.max(addr as usize + 1).max(target + 1);
                memory[0] = memory[0].wrapping_add(memory[target]);
            }
            Instruction::SubtractImmediate(val) => memory[0] = memory[0].wrapping_sub(val),
            Instruction::SubtractDirect(addr) => {
                highest_accessed = highest_accessed.max(addr as usize + 1);
                memory[0] = memory[0].wrapping_sub(memory[addr as usize]);
            }
            Instruction::SubtractIndirect(addr) => {
                let target = memory[addr as usize] as usize;
                highest_accessed = highest_accessed.max(addr as usize + 1).max(target + 1);
                memory[0] = memory[0].wrapping_sub(memory[target]);
            }
            Instruction::MultiplyImmediate(val) => memory[0] = memory[0].wrapping_mul(val),
            Instruction::MultiplyDirect(addr) => {
                highest_accessed = highest_accessed.max(addr as usize + 1);
                memory[0] = memory[0].wrapping_mul(memory[addr as usize]);
            }
            Instruction::MultiplyIndirect(addr) => {
                let target = memory[addr as usize] as usize;
                highest_accessed = highest_accessed.max(addr as usize + 1).max(target + 1);
                memory[0] = memory[0].wrapping_mul(memory[target]);
            }
            Instruction::DivideImmediate(val) => memory[0] = if val != 0 { memory[0].wrapping_div(val) } else { 0 },
            Instruction::DivideDirect(addr) => {
                highest_accessed = highest_accessed.max(addr as usize + 1);
                let divisor = memory[addr as usize];
                memory[0] = if divisor != 0 { memory[0].wrapping_div(divisor) } else { 0 };
            }
            Instruction::DivideIndirect(addr) => {
                let target = memory[addr as usize] as usize;
                highest_accessed = highest_accessed.max(addr as usize + 1).max(target + 1);
                let divisor = memory[target];
                memory[0] = if divisor != 0 { memory[0].wrapping_div(divisor) } else { 0 };
            }
            Instruction::LoadImmediate(val) => memory[0] = val,
            Instruction::LoadDirect(addr) => {
                highest_accessed = highest_accessed.max(addr as usize + 1);
                memory[0] = memory[addr as usize];
            }
            Instruction::LoadIndirect(addr) => {
                let target = memory[addr as usize] as usize;
                highest_accessed = highest_accessed.max(addr as usize + 1).max(target + 1);
                memory[0] = memory[target];
            }
            Instruction::StoreDirect(addr) => {
                highest_accessed = highest_accessed.max(addr as usize + 1);
                memory[addr as usize] = memory[0];
            }
            Instruction::StoreIndirect(addr) => {
                let target = memory[addr as usize] as usize;
                highest_accessed = highest_accessed.max(addr as usize + 1).max(target + 1);
                memory[target] = memory[0];
            }
            Instruction::ReadDirect(addr) => {
                println!("The program is requesting input: ");
                input.clear();
                stdin.read_line(&mut input).expect("Failed to read input.");
                highest_accessed = highest_accessed.max(addr as usize + 1);
                memory[addr as usize] =
                    input.trim().parse::<i32>().expect("Failed to parse input.");
            }
            Instruction::ReadIndirect(addr) => {
                println!("The program is requesting input: ");
                input.clear();
                stdin.read_line(&mut input).expect("Failed to read input.");
                let target = memory[addr as usize] as usize;
                highest_accessed = highest_accessed.max(addr as usize + 1).max(target + 1);
                memory[target] =
                    input.trim().parse::<i32>().expect("Failed to parse input.");
            }
            Instruction::ReadCharDirect(addr) => {
                highest_accessed = highest_accessed.max(addr as usize + 1);
                let mut byte = [0u8];
                let n = std::io::stdin().read(&mut byte).unwrap_or(0);
                memory[addr as usize] = if n == 0 { 0 } else { byte[0] as i32 };
            }
            Instruction::ReadCharIndirect(addr) => {
                let target = memory[addr as usize] as usize;
                highest_accessed = highest_accessed.max(addr as usize + 1).max(target + 1);
                let mut byte = [0u8];
                let n = std::io::stdin().read(&mut byte).unwrap_or(0);
                memory[target] = if n == 0 { 0 } else { byte[0] as i32 };
            }
            Instruction::WriteImmediate(val) => println!("{}", val),
            Instruction::WriteDirect(addr) => {
                highest_accessed = highest_accessed.max(addr as usize + 1);
                println!("{}", memory[addr as usize]);
            }
            Instruction::WriteIndirect(addr) => {
                let target = memory[addr as usize] as usize;
                highest_accessed = highest_accessed.max(addr as usize + 1).max(target + 1);
                println!("{}", memory[target]);
            }
            Instruction::WriteCharImmediate(val) => {
                let byte = (val & 0xff) as u8;
                let _ = std::io::stdout().write_all(&[byte]);
                let _ = std::io::stdout().flush();
            }
            Instruction::WriteCharDirect(addr) => {
                highest_accessed = highest_accessed.max(addr as usize + 1);
                let byte = (memory[addr as usize] & 0xff) as u8;
                let _ = std::io::stdout().write_all(&[byte]);
                let _ = std::io::stdout().flush();
            }
            Instruction::WriteCharIndirect(addr) => {
                let target = memory[addr as usize] as usize;
                highest_accessed = highest_accessed.max(addr as usize + 1).max(target + 1);
                let byte = (memory[target] & 0xff) as u8;
                let _ = std::io::stdout().write_all(&[byte]);
                let _ = std::io::stdout().flush();
            }
            Instruction::Jump(target) => program_counter = target,
            Instruction::JumpIndirect(addr) => {
                let target = memory[addr as usize] as usize;
                highest_accessed = highest_accessed.max(addr as usize + 1);
                program_counter = target;
            }
            Instruction::JumpIfZero(target) => {
                if memory[0] == 0 {
                    program_counter = target;
                }
            }
            Instruction::JumpIfGreaterThanZero(target) => {
                if memory[0] > 0 {
                    program_counter = target;
                }
            }
            Instruction::Halt => break,
            Instruction::Debug => {
                for i in 0..highest_accessed {
                    println!("Memory[{}] = {}", i, memory[i]);
                }
            }
        }

        if let Some(window) = window.as_mut() {
            instructions_since_last_draw += 1;
            if instructions_since_last_draw >= REDRAW_INTERVAL {
                if !window.is_open() || !draw_framebuffer(window, &memory) {
                    break;
                }
                instructions_since_last_draw = 0;
            }
        }

        instructions_executed += 1;
    }

    println!("Instructions executed: {}", instructions_executed);
}
