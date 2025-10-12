use crate::debug::Breakpoint;
use crate::instructions::{Comp, Destination, Instruction, Jump, A, C};
use crate::parser::MAX_RAM;
use crate::symbol_table;
use std::collections::HashSet;
use std::{
    num::Wrapping,
    ops::{Neg, Not},
    usize,
};

/// Represents the HACK CPU state, including the 3 registers, and the RAM. It additionally stores
/// the [symbol_table::SymbolTable] (also known as an address table, useful for the labels in the program code) and
/// the [Breakpoint]s (used for debugging programs).
#[derive(Debug)]
pub struct CPUState {
    pub a: Wrapping<i16>,
    pub d: Wrapping<i16>,
    pub pc: u16,
    pub ram: [Wrapping<i16>; MAX_RAM],
    pub address_table: symbol_table::SymbolTable,
    pub breakpoints: HashSet<Breakpoint>,
}

impl CPUState {
    /// Creates a CPU, with default starting states.
    /// # Example
    /// ```
    /// let cpu = CPUState::new();
    /// ```
    pub fn new() -> Self {
        Self {
            a: Wrapping(0),
            d: Wrapping(0),
            pc: 0,
            ram: std::array::from_fn(|_| Wrapping(0)),
            address_table: symbol_table::SymbolTable::new(),
            breakpoints: HashSet::new(),
        }
    }

    /// Resets the symbol table. This is necessary when replacing the ROM instructions with a new
    /// program
    pub fn reset_address_table(self: &mut Self) {
        self.address_table = symbol_table::SymbolTable::new();
    }

    /// Executes the next instruction, according to the program counter (PC) register
    pub fn interpret(self: &mut Self, instruction: &Instruction) {
        match instruction {
            Instruction::A(a) => self.a_instruction(&a),
            Instruction::C(c) => self.c_instruction(&c),
            Instruction::Label(_) | Instruction::None => self.pc += 1,
        }
    }

    /// Executes an A instruction
    fn a_instruction(self: &mut Self, a: &A) {
        self.a = Wrapping(a.dest);
        self.pc += 1;
    }

    /// Executes a C instruction
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

    /// Resets the RAM of the CPU to be all zeroes once more
    pub fn reset_ram(self: &mut Self) {
        self.ram.iter_mut().for_each(|x| *x = Wrapping(0));
    }
}
