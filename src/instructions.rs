#[derive(Debug)]
pub enum Instruction {
    A(A),
    Label(),
    C(C),
    None,
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
pub enum Destination {
    None,
    A,
    M,
    D,
    MD,
    AM,
    AD,
    AMD,
}

impl Destination {
    fn new(dest: &str) -> Destination {
        match dest {
            "" => Destination::None,
            "A" => Destination::A,
            "M" => Destination::M,
            "D" => Destination::D,
            "MD" => Destination::MD,
            "AM" => Destination::AM,
            "AD" => Destination::AD,
            "AMD" => Destination::AMD,
            _ => panic!("Parse error: {} is not a valid destination", dest),
        }
    }
}

#[derive(Debug)]
pub enum Jump {
    None,
    JGT,
    JEQ,
    JGE,
    JLT,
    JNE,
    JLE,
    JMP,
}

impl Jump {
    fn new(jump: &str) -> Jump {
        match jump {
            "" => Jump::None,
            "JGT" => Jump::JGT,
            "JEQ" => Jump::JEQ,
            "JGE" => Jump::JGE,
            "JLT" => Jump::JLT,
            "JNE" => Jump::JNE,
            "JLE" => Jump::JLE,
            "JMP" => Jump::JMP,
            _ => panic!("Parse error: {} is not a valid jump instruction", jump),
        }
    }
}

#[derive(Debug)]
pub struct C {
    pub dest: Destination,
    pub comp: String,
    pub jump: Jump,
}

impl C {
    pub fn new(dest: &str, comp: &str, jump: &str) -> Self {
        Self {
            dest: Destination::new(dest),
            comp: comp.to_string(),
            jump: Jump::new(jump),
        }
    }
}
