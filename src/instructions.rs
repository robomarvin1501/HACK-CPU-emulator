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

#[derive(Debug, PartialEq, Eq)]
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

#[derive(Debug, PartialEq, Eq)]
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

#[derive(Debug, PartialEq, Eq)]
pub enum Comp {
    Zero,
    One,
    MinusOne,
    D,
    A,
    NotD,
    NotA,
    MinusD,
    MinusA,
    DPlusOne,
    APlusOne,
    DMinusOne,
    AMinusOne,
    DPlusA,
    DMinusA,
    AMinusD,
    DAndA,
    DOrA,

    M,
    NotM,
    MinusM,
    MPlusOne,
    MMinusOne,
    DPlusM,
    DMinusM,
    MMinusD,
    DAndM,
    DOrM,

    LeftShiftA,
    LeftShiftD,
    LeftShiftM,
    RightShiftA,
    RightShiftD,
    RightShiftM,
}

impl Comp {
    fn new(comp: &str) -> Comp {
        match comp {
            "0" => Comp::Zero,
            "1" => Comp::One,
            "-1" => Comp::MinusOne,
            "D" => Comp::D,
            "A" => Comp::A,
            "!D" => Comp::NotD,
            "!A" => Comp::NotA,
            "-D" => Comp::MinusD,
            "-A" => Comp::MinusA,
            "D+1" => Comp::DPlusOne,
            "A+1" => Comp::APlusOne,
            "D-1" => Comp::DMinusOne,
            "A-1" => Comp::AMinusOne,
            "D+A" => Comp::DPlusA,
            "D-A" => Comp::DMinusA,
            "A-D" => Comp::AMinusD,
            "D&A" => Comp::DAndA,
            "D|A" => Comp::DOrA,

            "M" => Comp::M,
            "!M" => Comp::NotM,
            "-M" => Comp::MinusM,
            "M+1" => Comp::MPlusOne,
            "M-1" => Comp::MMinusOne,
            "D+M" => Comp::DPlusM,
            "D-M" => Comp::DMinusM,
            "M-D" => Comp::MMinusD,
            "D&M" => Comp::DAndM,
            "D|M" => Comp::DOrM,

            "A<<" => Comp::LeftShiftA,
            "D<<" => Comp::LeftShiftD,
            "M<<" => Comp::LeftShiftM,
            "A>>" => Comp::RightShiftA,
            "D>>" => Comp::RightShiftD,
            "M>>" => Comp::RightShiftM,

            _ => panic!(
                "Parse error: {} is not a valid comparison instruction",
                comp
            ),
        }
    }
}

#[derive(Debug)]
pub struct C {
    pub dest: Destination,
    pub comp: Comp,
    pub jump: Jump,
}

impl C {
    pub fn new(dest: &str, comp: &str, jump: &str) -> Self {
        Self {
            dest: Destination::new(dest),
            comp: Comp::new(comp),
            jump: Jump::new(jump),
        }
    }
}
