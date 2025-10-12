use core::fmt;

/// Represents the different kinds of instructions that are run on the CPU.
/// Label is never constructed, but left for the future, since there is an intention to show the
/// labels in the emulator
#[allow(dead_code)]
#[derive(Debug, PartialEq, Eq)]
pub enum Instruction {
    A(A),
    Label(String),
    C(C),
    None,
}

impl fmt::Display for Instruction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Instruction::A(a) => write!(f, "{}", a),
            Instruction::C(c) => writeln!(f, "{}", c),
            Instruction::Label(l) => write!(f, "({})", l),
            Instruction::None => write!(f, ""),
        }
    }
}

/// Represents an A(ddress) instruction. This sets the A register to some 15 bit value.
#[derive(Debug, PartialEq, Eq)]
pub struct A {
    pub dest: i16,
}
impl A {
    /// Create a new [A] instruction from an input string. Useful for building from source files.
    pub fn new(dest: &str) -> Self {
        Self {
            dest: match dest.parse::<i16>() {
                Ok(d) => d,
                Err(e) => panic!("Failed to parse the destination of the A instruction: {e}"),
            },
        }
    }
}

impl fmt::Display for A {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "@{}", self.dest)
    }
}

/// Represents the destination in which the computed value of a [C] instruction should be stored.
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
    /// Create a new destination for a [C] instruction.
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

impl fmt::Display for Destination {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Destination::None => "",
            Destination::A => "A",
            Destination::M => "M",
            Destination::D => "D",
            Destination::MD => "MD",
            Destination::AM => "AM",
            Destination::AD => "AD",
            Destination::AMD => "AMD",
        };
        write!(f, "{}", s)
    }
}

/// Stores the target location to which a [C] instruction should jump upon completion. [None]
/// indicates that no jump will take place, but rather the program counter will be incremented by 1.
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
    /// Create a new jump location for a [C] instruction
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

impl fmt::Display for Jump {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Jump::None => "",
            Jump::JGT => "JGT",
            Jump::JEQ => "JEQ",
            Jump::JGE => "JGE",
            Jump::JLT => "JLT",
            Jump::JNE => "JNE",
            Jump::JLE => "JLE",
            Jump::JMP => "JMP",
        };
        write!(f, "{}", s)
    }
}

/// Stores the type of computation that should be carried out by a [C] instruction.
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
    /// Create a new computation for a [C] instruction.
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

impl fmt::Display for Comp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Comp::Zero => "0",
            Comp::One => "1",
            Comp::MinusOne => "-1",
            Comp::D => "D",
            Comp::A => "A",
            Comp::NotD => "!D",
            Comp::NotA => "!A",
            Comp::MinusD => "-D",
            Comp::MinusA => "-A",
            Comp::DPlusOne => "D+1",
            Comp::APlusOne => "A+1",
            Comp::DMinusOne => "D-1",
            Comp::AMinusOne => "A-1",
            Comp::DPlusA => "D+A",
            Comp::DMinusA => "D-A",
            Comp::AMinusD => "A-D",
            Comp::DAndA => "D&A",
            Comp::DOrA => "D|A",

            Comp::M => "M",
            Comp::NotM => "!M",
            Comp::MinusM => "-M",
            Comp::MPlusOne => "M+1",
            Comp::MMinusOne => "M-1",
            Comp::DPlusM => "D+M",
            Comp::DMinusM => "D-M",
            Comp::MMinusD => "M-D",
            Comp::DAndM => "D&M",
            Comp::DOrM => "D|M",

            Comp::LeftShiftA => "A<<",
            Comp::LeftShiftD => "D<<",
            Comp::LeftShiftM => "M<<",
            Comp::RightShiftA => "A>>",
            Comp::RightShiftD => "D>>",
            Comp::RightShiftM => "M>>",
        };
        write!(f, "{}", s)
    }
}

/// Represents a C(ompute) instruction. These do 3 things, they compute (comp) something, and store
/// it in dest, followed by a jump to another location.
#[derive(Debug, PartialEq, Eq)]
pub struct C {
    pub dest: Destination,
    pub comp: Comp,
    pub jump: Jump,
}

impl C {
    /// Create a new [C] instruction based off the inputs from the source file.
    pub fn new(dest: &str, comp: &str, jump: &str) -> Self {
        Self {
            dest: Destination::new(dest),
            comp: Comp::new(comp),
            jump: Jump::new(jump),
        }
    }
}

impl fmt::Display for C {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.dest != Destination::None {
            write!(f, "{}=", self.dest)?;
        }
        write!(f, "{}", self.comp)?;
        if self.jump != Jump::None {
            write!(f, ";{}", self.jump)?;
        }
        Ok(())
    }
}
