use core::panic;
use std::{
    env,
    error::Error,
    fmt::Debug,
    fs,
    io::Read,
    num::Wrapping,
    ops::{Neg, Not},
    path::PathBuf,
    usize,
};

use instructions::{Comp, Destination, Instruction, Jump, A, C};
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
        std::io::stdin().bytes().next();
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
            Instruction::Label() | Instruction::None => self.pc += 1,
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
        let answer: Wrapping<i16> = match c.comp {
            Comp::Zero => Wrapping(0),
            Comp::One => Wrapping(1),
            Comp::MinusOne => Wrapping(-1),
            Comp::D => self.d,
            Comp::A => self.a,
            Comp::NotD => self.d.not(),
            Comp::NotA => self.a.not(),
            Comp::MinusD => -self.d,
            Comp::MinusA => -(self.a),
            Comp::DPlusOne => self.d + Wrapping(1),
            Comp::APlusOne => self.a + Wrapping(1),
            Comp::DMinusOne => self.d + Wrapping(1),
            Comp::AMinusOne => self.a + Wrapping(1),
            Comp::DPlusA => self.d + self.a,
            Comp::DMinusA => self.d - self.a,
            Comp::AMinusD => self.a - self.d,
            Comp::DAndA => self.d & self.a,
            Comp::DOrA => self.d | self.a,

            Comp::M => self.ram[self.a.0 as usize],
            Comp::NotM => self.ram[self.a.0 as usize].not(),
            Comp::MinusM => self.ram[self.a.0 as usize].neg(),
            Comp::MPlusOne => self.ram[self.a.0 as usize] + Wrapping(1),
            Comp::MMinusOne => self.ram[self.a.0 as usize] - Wrapping(1),
            Comp::DPlusM => self.d + self.ram[self.a.0 as usize],
            Comp::DMinusM => self.d - self.ram[self.a.0 as usize],
            Comp::MMinusD => self.ram[self.a.0 as usize] - self.d,
            Comp::DAndM => self.ram[self.a.0 as usize] & self.d,
            Comp::DOrM => self.ram[self.a.0 as usize] | self.d,

            Comp::LeftShiftA => self.a << 1,
            Comp::LeftShiftD => self.d << 1,
            Comp::LeftShiftM => self.ram[self.a.0 as usize] << 1,
            Comp::RightShiftA => self.a >> 1,
            Comp::RightShiftD => self.d >> 1,
            Comp::RightShiftM => self.ram[self.a.0 as usize] >> 1,
        };

        match c.dest {
            Destination::None => {}
            Destination::A => self.a = answer,
            Destination::M => self.ram[self.a.0 as usize] = answer,
            Destination::D => self.d = answer,
            Destination::MD => {
                self.ram[self.a.0 as usize] = answer;
                self.d = answer;
            }
            Destination::AM => {
                self.ram[self.a.0 as usize] = answer;
                self.a = answer;
            }
            Destination::AD => {
                self.a = answer;
                self.d = answer;
            }
            Destination::AMD => {
                self.ram[self.a.0 as usize] = answer;
                self.a = answer;
                self.d = answer;
            }
        }

        self.pc = match c.jump {
            Jump::None => self.pc + 1,
            Jump::JGT => {
                if answer > Wrapping(0) {
                    // TODO check that A > 0?
                    self.a.0 as u16
                } else {
                    self.pc + 1
                }
            }
            Jump::JEQ => {
                if answer == Wrapping(0) {
                    self.a.0 as u16
                } else {
                    self.pc + 1
                }
            }
            Jump::JGE => {
                if answer >= Wrapping(0) {
                    self.a.0 as u16
                } else {
                    self.pc + 1
                }
            }
            Jump::JLT => {
                if answer < Wrapping(0) {
                    self.a.0 as u16
                } else {
                    self.pc + 1
                }
            }
            Jump::JNE => {
                if answer != Wrapping(0) {
                    self.a.0 as u16
                } else {
                    self.pc + 1
                }
            }
            Jump::JLE => {
                if answer <= Wrapping(0) {
                    self.a.0 as u16
                } else {
                    self.pc + 1
                }
            }
            Jump::JMP => self.a.0 as u16,
        };
    }
}
