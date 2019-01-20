#![feature(test)]
#![feature(duration_as_u128)]

mod parse;
use parse::{parse, Instruction};

mod optimize;
use optimize::optimize;

mod annotate;
use annotate::annotate;

use std::fs::File;
use std::io::prelude::*;
use std::time::{Instant, Duration};

extern crate clap;
use clap::{Arg, App, SubCommand};

extern crate itertools;

fn main() {
    let matches = App::new("BranfuckVM")
        .version("0.1")
        .author("Matej Stuchlik")
        .about("Interpreter for the Brainfuck language")
        .arg(Arg::with_name("FILE")
             .help("Brainfuck source file to execute")
             .required(true))
        .arg(Arg::with_name("optimize")
             .long("optimize")
             .help("Set optimizations on or off")
             .takes_value(false))
        .subcommand(SubCommand::with_name("annotate")
                    .about("Print bytecode generated from a given source"))
        .subcommand(SubCommand::with_name("run")
                    .about("Execute given source file and print the output"))
        .subcommand(SubCommand::with_name("profile")
                    .about("Execute given source file and display annotated source with timing"))
        .get_matches();

    let filename = matches.value_of("FILE").expect("Must specify file to execute");
    let mut f = File::open(filename).expect("File not found");
    let mut program = String::new();
    f.read_to_string(&mut program).expect("Something went wrong reading the file");
    let program: Vec<_> = program.chars().collect();

    let mut vm = VirtualMachine::new();
    vm.optimize = matches.is_present("optimize");
    if let Some(_matches) = matches.subcommand_matches("run") {
		let output = vm.run(&program).output;
		println!("{}", output.iter().collect::<String>());
    } else if let Some(_matches) = matches.subcommand_matches("annotate") {
        println!("{}", annotate(&vm.bytecode(&program), None));
    } else if let Some(_matches) = matches.subcommand_matches("profile") {
        let profile = vm.run(&program).profile;
        println!("{}", annotate(&vm.bytecode(&program), Some(profile)));
    }
}

struct VirtualMachine {
    memory: [u8; 30_000],
    pointer: usize,
    optimize: bool,
    profile: bool,
}

struct ExecutionOutput {
    output: Vec<char>,
    profile: Vec<u32>,
}

impl VirtualMachine {
    fn new() -> Self {
        VirtualMachine {
            memory: [0; 30000],
            pointer: 0,
            optimize: false,
            profile: false,
        }
    }

    fn run(&mut self, program: &[char]) -> ExecutionOutput {
        let now = Instant::now();
        let mut program = parse(program);
        eprintln!("Parse completed in {}μs", now.elapsed().as_micros());
        if self.optimize == true {
            let now = Instant::now();
            program = optimize(&program);
            eprintln!("Optimize completed in {}μs", now.elapsed().as_micros());
        }
        self.execute(&program)
    }

    fn bytecode(&mut self, program: &[char]) -> Vec<Instruction> {
        let mut program = parse(program);
        if self.optimize == true {
            program = optimize(&program);
        }
        program
    }

    fn execute(&mut self, program: &[Instruction]) -> ExecutionOutput {
        let mut output = vec![];
        let mut instruction_ptr = 0;
        let mut op_counter = vec![0; program.len()];

        while instruction_ptr < program.len() {
            let instruction = &program[instruction_ptr];
            op_counter[instruction_ptr] += 1;
            match *instruction {
                Instruction::Move(n) => {
                    if n > 0 {
                        let n = n as usize;
                        if self.pointer + n as usize >= 30_000 {
                            panic!(format!(
                                "Data pointer overflow! (pointer = {}, offset = {})",
                                self.pointer, n
                            ))
                        };
                        self.pointer += n;
                    } else {
                        let n = n.abs() as usize;
                        if n > self.pointer {
                            panic!(format!(
                                "Data pointer underflow! (pointer = {}, offset = {})",
                                self.pointer, n
                            ))
                        };
                        self.pointer -= n;
                    }
                }
                Instruction::Add(n) => {
                    let value = self.memory[self.pointer];
                    if n > 0 {
                        self.memory[self.pointer] = value.wrapping_add(n as u8);
                    } else if n < 0 {
                        self.memory[self.pointer] = value.wrapping_sub(n.abs() as u8);
                    }
                }
                Instruction::JumpIfZero(n) => {
                    if self.memory[self.pointer] == 0 {
                        instruction_ptr = n;
                        continue;
                    }
                }
                Instruction::JumpIfNotZero(n) => {
                    if self.memory[self.pointer] != 0 {
                        instruction_ptr = n;
                        continue;
                    }
                }
                Instruction::Print => output.push(self.memory[self.pointer] as char),
                Instruction::SetZero => self.memory[self.pointer] = 0,
                _ => panic!(format!(
                    "Attempted to execute unknown instruction {:?}",
                    instruction
                )),
            };
            instruction_ptr += 1;
        }
        ExecutionOutput{ output: output, profile: op_counter }
    }
}

#[cfg(test)]
mod tests {
    use parse::Instruction::*;
    use VirtualMachine;

    #[test]
    fn test_complex_program() {
        let program = "++++++++[>++++[>++>+++>+++>+<<<<-]>+>+>->>+[<]<-]>>.>---.+++++++..+++.>>
            .<-.<.+++.------.--------.>>+.>++.";
        let mut vm = VirtualMachine::new();
        let result = vm.run(&program.chars().collect::<Vec<char>>());
        assert_eq!(result.output, "Hello World!\n".chars().collect::<Vec<_>>());
    }

    #[test]
    fn it_prints() {
        let print = vec![Print];
        let mut vm = VirtualMachine::new();
        assert_eq!(vm.execute(&print).output, vec![0 as char]);
    }

    #[test]
    fn it_moves() {
        let move_forward = vec![Move(1)];
        let move_backward = vec![Move(-1)];
        let mut vm = VirtualMachine::new();
        assert_eq!(vm.pointer, 0);
        vm.execute(&move_forward);
        assert_eq!(vm.pointer, 1);
        vm.execute(&move_backward);
        assert_eq!(vm.pointer, 0);
    }

    #[test]
    #[should_panic]
    fn it_panics_on_move_overflow() {
        let program = vec![Move(30_000)];
        let mut vm = VirtualMachine::new();
        vm.execute(&program);
    }

    #[test]
    #[should_panic]
    fn it_panics_on_move_underflow() {
        let program = vec![Move(-1)];
        let mut vm = VirtualMachine::new();
        vm.execute(&program);
    }

    #[test]
    fn it_adds() {
        let add_positive = vec![Add(1)];
        let add_zero = vec![Add(0)];
        let add_negative = vec![Add(-1)];
        let mut vm = VirtualMachine::new();
        assert_eq!(vm.memory[0], 0);
        vm.execute(&add_positive);
        assert_eq!(vm.memory[0], 1);
        vm.execute(&add_zero);
        assert_eq!(vm.memory[0], 1);
        vm.execute(&add_negative);
        assert_eq!(vm.memory[0], 0);
    }

    #[test]
    fn it_sets_zero() {
        let add_ten = vec![Add(10)];
        let set_zero = vec![SetZero];
        let mut vm = VirtualMachine::new();
        vm.execute(&add_ten);
        assert_eq!(vm.memory[0], 10);
        vm.execute(&set_zero);
        assert_eq!(vm.memory[0], 0);
    }
}
