use std::num::Wrapping;


fn main() {
    let mut a: [i16; 10] = [0; 10];
    dbg!(a);
    a[9] = 10;
    dbg!(a);
}

#[derive(Debug)]
struct CPUState {
    a: i16,
    d: i16,
    pc: u16,
}

fn interpret(c: Instruction) {
    match c {
        Instruction::A(A) => a_instruction(A),
    }
}

fn a_instruction(a: A) {
    let loc = a.dest.parse::<i16>();
    if loc.is_ok() {
        A_REGISTER = Wrapping(loc.unwrap());
        return;
    }
}

#[derive(Debug)]
pub enum Instruction {
    A(A),
    Label(),
    C(C),
}

#[derive(Debug)]
pub struct A {
    pub dest: String,
}
impl A {
    pub fn new(dest: &str) -> Self {
        Self {
            dest: dest.to_string(),
        }
    }
}

#[derive(Debug)]
pub struct C {
    pub dest: String,
    pub comp: String,
    pub jump: String,
    pub shift: bool,
}
