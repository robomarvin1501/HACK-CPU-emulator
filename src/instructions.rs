const LEFT_SHIFT: &'static str = "<<";
const RIGHT_SHIFT: &'static str = ">>";

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

impl C {
    pub fn new(dest: &str, comp: &str, jump: &str) -> Self {
        Self {
            dest: dest.to_string(),
            comp: comp.to_string(),
            jump: jump.to_string(),

            shift: comp.contains(LEFT_SHIFT) || comp.contains(RIGHT_SHIFT),
        }
    }
}
