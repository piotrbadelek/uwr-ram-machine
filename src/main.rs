use std::collections::HashMap;
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
    Jump(usize),
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

const MEMORY_SIZE: usize = 262144;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let enable_graphics = args.iter().any(|arg| arg == "--gfx");
    let file_path = args
        .iter()
        .skip(1)
        .find(|arg| *arg != "--gfx")
        .expect("Usage: program <file> [--gfx]");
    let file_content = std::fs::read_to_string(file_path).unwrap();
    let mut instructions: Vec<Instruction> = Vec::with_capacity(file_content.lines().count());
    let mut instructions_executed = 0;

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
                        ParameterType::Immediate => Instruction::LoadImmediate(parameter[1..].parse::<i32>().unwrap()),
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
                    "jump" => match parameter_type {
                        ParameterType::Label => Instruction::Jump(*labels.get(parameter).expect("Unknown label")),
                        ParameterType::Direct => Instruction::Jump(parameter.parse::<usize>().unwrap()),
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

    let mut memory: [i32; MEMORY_SIZE] = [0; MEMORY_SIZE];
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
            Instruction::AddImmediate(val) => memory[0] += val,
            Instruction::AddDirect(addr) => {
                highest_accessed = highest_accessed.max(addr as usize + 1);
                memory[0] += memory[addr as usize];
            }
            Instruction::AddIndirect(addr) => {
                let target = memory[addr as usize] as usize;
                highest_accessed = highest_accessed.max(addr as usize + 1).max(target + 1);
                memory[0] += memory[target];
            }
            Instruction::SubtractImmediate(val) => memory[0] -= val,
            Instruction::SubtractDirect(addr) => {
                highest_accessed = highest_accessed.max(addr as usize + 1);
                memory[0] -= memory[addr as usize];
            }
            Instruction::SubtractIndirect(addr) => {
                let target = memory[addr as usize] as usize;
                highest_accessed = highest_accessed.max(addr as usize + 1).max(target + 1);
                memory[0] -= memory[target];
            }
            Instruction::MultiplyImmediate(val) => memory[0] *= val,
            Instruction::MultiplyDirect(addr) => {
                highest_accessed = highest_accessed.max(addr as usize + 1);
                memory[0] *= memory[addr as usize];
            }
            Instruction::MultiplyIndirect(addr) => {
                let target = memory[addr as usize] as usize;
                highest_accessed = highest_accessed.max(addr as usize + 1).max(target + 1);
                memory[0] *= memory[target];
            }
            Instruction::DivideImmediate(val) => memory[0] /= val,
            Instruction::DivideDirect(addr) => {
                highest_accessed = highest_accessed.max(addr as usize + 1);
                memory[0] /= memory[addr as usize];
            }
            Instruction::DivideIndirect(addr) => {
                let target = memory[addr as usize] as usize;
                highest_accessed = highest_accessed.max(addr as usize + 1).max(target + 1);
                memory[0] /= memory[target];
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
            Instruction::Jump(target) => program_counter = target,
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
            if instructions_since_last_draw >= 10000 {
                draw_framebuffer(window, &memory);
                instructions_since_last_draw = 0;
            }
        }

        instructions_executed += 1;
    }

    println!("Instructions executed: {}", instructions_executed);
}
