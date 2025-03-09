use std::{
    fmt::Debug,
    num::Wrapping,
    ops::{Add, Neg, Not, Sub},
    usize,
};

use instructions::{Instruction, A, C};
use parser::parse;

mod instructions;
mod parser;
mod symbol_table;

fn main() {
    let instructions = vec![
        String::from("@15"),
        String::from("@R10"),
        String::from("M=A"),
        String::from("M=M<<"),
    ];
    let mut state = CPUState::new();

    let instructions = parse(&instructions, &mut state.address_table);
    dbg!(&instructions);

    for instr in instructions {
        state.interpret(&instr);
        dbg!(state.ram[10]);
    }
}

#[derive(Debug)]
struct CPUState {
    a: Wrapping<i16>,
    d: Wrapping<i16>,
    pc: u16,
    ram: [Wrapping<i16>; 32768],
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
            Instruction::Label() => {},
        }
    }

    fn a_instruction(self: &mut Self, a: &A) {
        let destination = self.address_table.table.get(&a.dest);
        match destination {
            Some(loc) => self.a = Wrapping((*loc) as i16),
            None => panic!("Invalid instruction: {:?}", a),
        }
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
            "" => self.pc,
            "JGT" => {
                if answer > Wrapping(0) {
                    answer.0 as u16
                } else {
                    self.pc
                }
            }
            "JEQ" => {
                if answer == Wrapping(0) {
                    answer.0 as u16
                } else {
                    self.pc
                }
            }
            "JGE" => {
                if answer >= Wrapping(0) {
                    answer.0 as u16
                } else {
                    self.pc
                }
            }
            "JLT" => {
                if answer < Wrapping(0) {
                    answer.0 as u16
                } else {
                    self.pc
                }
            }
            "JNE" => {
                if answer != Wrapping(0) {
                    answer.0 as u16
                } else {
                    self.pc
                }
            }
            "JLE" => {
                if answer <= Wrapping(0) {
                    answer.0 as u16
                } else {
                    self.pc
                }
            }
            "JMP" => answer.0 as u16,

            _ => panic!("Invalid jump command: {}", c.jump),
        };
    }
}

