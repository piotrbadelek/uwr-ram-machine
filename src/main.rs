#[derive(PartialEq, Eq)]
enum InstructionType {
    Add,
    Divide,
    Halt,
    JumpIfGreaterThanZero,
    Jump,
    JumpIfZero,
    Load,
    Multiply,
    Read,
    Store,
    Subtract,
    Write,
    Debug,
}

#[derive(PartialEq, Eq)]
enum ArgumentType {
    Direct,
    Value,
    IndirectAddressing,
}

struct Parameter {
    argument_type: ArgumentType,
    value: String,
}

struct Instruction {
    label: String,
    command: InstructionType,
    parameter: Parameter,
}

fn resolve_parameter(
    memory: &[i32; 256],
    instructions: &Vec<Instruction>,
    parameter: &Parameter,
    return_address: bool, // force the function to return the address instead of the value
) -> i32 {
    match parameter.argument_type {
        ArgumentType::Direct => {
            // return the value at the address if value is an integer,
            // otherwise it's a label and just return the address to jump to
            if parameter.value.chars().nth(0).unwrap().is_alphabetic() {
                let index = instructions.iter().position(|i| i.label == parameter.value);
                if return_address {
                    return index.unwrap() as i32;
                }
                return memory[index.unwrap()];
            }

            let index = parameter.value.parse::<usize>().unwrap();
            if return_address {
                return index as i32;
            }
            return memory[index];
        }
        ArgumentType::Value => parameter.value.parse::<i32>().unwrap(),
        ArgumentType::IndirectAddressing => {
            let index = resolve_parameter(
                memory,
                instructions,
                &Parameter {
                    argument_type: ArgumentType::Direct,
                    value: parameter.value.clone(),
                },
                false,
            );
            if return_address {
                return index as i32;
            }
            return memory[index as usize];
        }
    }
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let file_path = &args[1];
    let file_content = std::fs::read_to_string(file_path).unwrap();
    let mut instructions: Vec<Instruction> = Vec::with_capacity(file_content.lines().count());

    for line in file_content.split("\n") {
        if line.is_empty() || line.chars().next().unwrap() == '/' {
            continue;
        }
        let mut accumulated: String = "".to_owned();
        let mut instruction: Instruction = Instruction {
            label: String::new(),
            command: InstructionType::Add,
            parameter: Parameter {
                argument_type: ArgumentType::Direct,
                value: String::new(),
            },
        };
        for character in line.chars() {
            if character == '/' {
                break;
            }
            if character == ':' {
                instruction.label = accumulated.clone();
                accumulated = "".to_string();
                continue;
            }
            accumulated += character.to_string().as_str();
        }

        let mut instruction_part = 0;
        for token in accumulated.trim().split(" ") {
            if token.is_empty() {
                continue;
            }
            if instruction_part == 0 {
                match token.to_string().as_str() {
                    "add" => instruction.command = InstructionType::Add,
                    "div" => instruction.command = InstructionType::Divide,
                    "halt" => instruction.command = InstructionType::Halt,
                    "jgtz" => instruction.command = InstructionType::JumpIfGreaterThanZero,
                    "jump" => instruction.command = InstructionType::Jump,
                    "jzero" => instruction.command = InstructionType::JumpIfZero,
                    "load" => instruction.command = InstructionType::Load,
                    "mult" => instruction.command = InstructionType::Multiply,
                    "read" => instruction.command = InstructionType::Read,
                    "store" => instruction.command = InstructionType::Store,
                    "sub" => instruction.command = InstructionType::Subtract,
                    "write" => instruction.command = InstructionType::Write,
                    "debug" => instruction.command = InstructionType::Debug,
                    _ => {
                        println!("Invalid instruction: {}", token);
                        std::process::exit(1);
                    }
                }
            } else {
                match token.chars().nth(0).unwrap() {
                    '=' => instruction.parameter.argument_type = ArgumentType::Value,
                    '^' => instruction.parameter.argument_type = ArgumentType::IndirectAddressing,
                    _ => instruction.parameter.argument_type = ArgumentType::Direct,
                };
                if instruction.parameter.argument_type == ArgumentType::Direct {
                    instruction.parameter.value = token.to_string()
                } else {
                    let mut param_value = token.to_string();
                    param_value.remove(0);
                    instruction.parameter.value = param_value;
                }
            }
            instruction_part += 1;
        }

        instructions.push(instruction);
    }

    let mut memory: [i32; 256] = [0; 256];
    let mut highest_accessed: usize = 0;
    let mut program_counter: usize = 0;
    let mut input = String::new();
    let stdin = std::io::stdin();
    while program_counter < instructions.len() {
        let instruction = &instructions[program_counter];
        if instruction.command == InstructionType::Halt {
            break;
        }
        program_counter += 1;
        let resolved_parameter =
            resolve_parameter(&memory, &instructions, &instruction.parameter, false);
        let resolved_parameter_address =
            resolve_parameter(&memory, &instructions, &instruction.parameter, true);
        match instruction.command {
            InstructionType::Add => memory[0] += resolved_parameter,
            InstructionType::Divide => memory[0] /= resolved_parameter,
            InstructionType::Halt => std::process::exit(0),
            InstructionType::JumpIfGreaterThanZero => {
                if memory[0] > 0 {
                    program_counter = resolved_parameter_address as usize;
                }
            }
            InstructionType::Jump => program_counter = resolved_parameter_address as usize,
            InstructionType::JumpIfZero => {
                if memory[0] == 0 {
                    program_counter = resolved_parameter_address as usize;
                }
            }
            InstructionType::Load => memory[0] = resolved_parameter,
            InstructionType::Multiply => memory[0] *= resolved_parameter,
            InstructionType::Read => {
                println!("The program is requesting input: ");
                input.clear();
                stdin.read_line(&mut input).expect("Failed to read input.");
                memory[resolved_parameter_address as usize] =
                    input.trim().parse::<i32>().expect("Failed to parse input.");
            }
            InstructionType::Store => memory[resolved_parameter_address as usize] = memory[0],
            InstructionType::Subtract => memory[0] -= resolved_parameter,
            InstructionType::Write => println!("{}", resolved_parameter),
            InstructionType::Debug => {
                for i in 0..highest_accessed {
                    println!("Memory[{}] = {}", i, memory[i]);
                }
            }
        }
        
        highest_accessed = highest_accessed.max((resolved_parameter_address + 1) as usize);
    }
}
