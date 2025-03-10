use core::panic;
use std::{
    env,
    fmt::Debug,
    fs,
    io::Read,
    num::Wrapping,
    ops::{Neg, Not},
    path::PathBuf,
    usize,
};

use instructions::{Instruction, A, C};
use parser::{parse, MAX_INSTRUCTIONS};

mod instructions;
mod parser;
mod symbol_table;

const ASM_FILE_EXTENSION: &'static str = "asm";

fn main() {
    let instructions = read_arg_file();

    let mut state = CPUState::new();

    let instructions = parse(instructions, &mut state.address_table);
    dbg!(&instructions);

    while state.pc < MAX_INSTRUCTIONS as u16 {
        dbg!(&instructions[state.pc as usize]);
        state.interpret(&instructions[state.pc as usize]);
        dbg!(&state.ram[0..20]);
        // std::io::stdin().bytes().next();
    }
}

fn read_arg_file() -> [String; MAX_INSTRUCTIONS] {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        panic!("Invalid usage, please use: cpuemulator <input path>")
    }
    let argument_path = env::args().nth(1).expect("No path provided");
    let argument_path = fs::canonicalize(&argument_path).expect("Invalid path provided");
    let input_path: PathBuf = if argument_path.is_dir() {
        panic!("Directories are not supported");
    } else {
        argument_path
    };

    if let Some(extension) = input_path.extension() {
        if extension.to_str().unwrap_or("").to_lowercase() != ASM_FILE_EXTENSION {
            panic!("Expected asm file, got {}", extension.to_str().unwrap());
        }
    }

    let input_path = PathBuf::from(input_path);

    let contents: String =
        fs::read_to_string(input_path).expect("Should have been able to read file");
    let instructions: Vec<String> = contents.split("\n").map(|s| s.trim().to_string()).collect();
    if instructions.len() > MAX_INSTRUCTIONS {
        panic!(
            "Too many instructions, expected a maximum of {}, got {}",
            MAX_INSTRUCTIONS,
            instructions.len()
        );
    }
    let mut ret: [String; MAX_INSTRUCTIONS] = [const { String::new() }; MAX_INSTRUCTIONS];
    for (i, instruction) in instructions.iter().enumerate() {
        ret[i] = instruction.to_string();
    }
    ret
}

#[derive(Debug)]
struct CPUState {
    a: Wrapping<i16>,
    d: Wrapping<i16>,
    pc: u16,
    ram: [Wrapping<i16>; MAX_INSTRUCTIONS],
    address_table: symbol_table::SymbolTable,
}

impl CPUState {
    pub fn new() -> Self {
        Self {
            a: Wrapping(0),
            d: Wrapping(0),
            pc: 0,
            ram: std::array::from_fn(|_| Wrapping(0)),
            address_table: symbol_table::SymbolTable::new(),
        }
    }

    pub fn interpret(self: &mut Self, instruction: &Instruction) {
        match instruction {
            Instruction::A(a) => self.a_instruction(&a),
            Instruction::C(c) => self.c_instruction(&c),
            Instruction::Label() => self.pc += 1,
            Instruction::None => self.pc += 1,
        }
    }

    fn a_instruction(self: &mut Self, a: &A) {
        let destination = self.address_table.table.get(&a.dest);
        match destination {
            Some(loc) => self.a = Wrapping((*loc) as i16),
            None => panic!("Invalid instruction: {:?}", a),
        }
        self.pc += 1;
    }

    fn c_instruction(self: &mut Self, c: &C) {
        let answer: Wrapping<i16> = match c.comp.as_str() {
            "0" => Wrapping(0),
            "1" => Wrapping(1),
            "-1" => Wrapping(-1),
            "D" => self.d,
            "A" => self.a,
            "!D" => self.d.not(),
            "!A" => self.a.not(),
            "-D" => -self.d,
            "-A" => -(self.a),
            "D+1" => self.d + Wrapping(1),
            "A+1" => self.a + Wrapping(1),
            "D-1" => self.d + Wrapping(1),
            "A-1" => self.a + Wrapping(1),
            "D+A" => self.d + self.a,
            "D-A" => self.d - self.a,
            "A-D" => self.a - self.d,
            "D&A" => self.d & self.a,
            "D|A" => self.d | self.a,

            "M" => self.ram[self.a.0 as usize],
            "!M" => self.ram[self.a.0 as usize].not(),
            "-M" => self.ram[self.a.0 as usize].neg(),
            "M+1" => self.ram[self.a.0 as usize] + Wrapping(1),
            "M-1" => self.ram[self.a.0 as usize] - Wrapping(1),
            "D+M" => self.d + self.ram[self.a.0 as usize],
            "D-M" => self.d - self.ram[self.a.0 as usize],
            "M-D" => self.ram[self.a.0 as usize] - self.d,
            "D&M" => self.ram[self.a.0 as usize] & self.d,
            "D|M" => self.ram[self.a.0 as usize] | self.d,

            "A<<" => self.a << 1,
            "D<<" => self.d << 1,
            "M<<" => self.ram[self.a.0 as usize] << 1,
            "A>>" => self.a >> 1,
            "D>>" => self.d >> 1,
            "M>>" => self.ram[self.a.0 as usize] >> 1,

            _ => panic!("Invalid instruction {}", c.comp),
        };

        match c.dest.as_str() {
            "" => {}
            "A" => self.a = answer,
            "M" => self.ram[self.a.0 as usize] = answer,
            "D" => self.d = answer,
            "MD" => {
                self.ram[self.a.0 as usize] = answer;
                self.d = answer;
            }
            "AM" => {
                self.ram[self.a.0 as usize] = answer;
                self.a = answer;
            }
            "AD" => {
                self.a = answer;
                self.d = answer;
            }
            "AMD" => {
                self.ram[self.a.0 as usize] = answer;
                self.a = answer;
                self.d = answer;
            }
            _ => panic!("Unrecognised destination: {}", c.dest),
        }

        self.pc = match c.jump.as_str() {
            "" => self.pc + 1,
            "JGT" => {
                if answer > Wrapping(0) { // TODO check that A > 0?
                    self.a.0 as u16
                } else {
                    self.pc + 1
                }
            }
            "JEQ" => {
                if answer == Wrapping(0) {
                    self.a.0 as u16
                } else {
                    self.pc + 1
                }
            }
            "JGE" => {
                if answer >= Wrapping(0) {
                    self.a.0 as u16
                } else {
                    self.pc + 1
                }
            }
            "JLT" => {
                if answer < Wrapping(0) {
                    self.a.0 as u16
                } else {
                    self.pc + 1
                }
            }
            "JNE" => {
                if answer != Wrapping(0) {
                    self.a.0 as u16
                } else {
                    self.pc + 1
                }
            }
            "JLE" => {
                if answer <= Wrapping(0) {
                    self.a.0 as u16
                } else {
                    self.pc + 1
                }
            }
            "JMP" => self.a.0 as u16,

            _ => panic!("Invalid jump command: {}", c.jump),
        };
    }
}
