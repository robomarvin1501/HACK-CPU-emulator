use std::collections::HashMap;

/// Represents the symbol table used for translating A instructions from names to locations in the
/// RAM.
#[derive(Debug)]
pub struct SymbolTable {
    pub table: HashMap<String, u16>,
    pub current_variable: u16,
}

impl SymbolTable {
    /// Creates a new symbol table. Includes various default values from the specification, such as
    /// register locations, the screen, the keyboard, and so on.
    /// # Example
    /// ```
    /// let symbol_table = SymbolTable::new();
    /// ```
    pub fn new() -> Self {
        let t: HashMap<String, u16> = HashMap::from([
            (String::from("R0"), 0),
            (String::from("R1"), 1),
            (String::from("R2"), 2),
            (String::from("R3"), 3),
            (String::from("R4"), 4),
            (String::from("R5"), 5),
            (String::from("R6"), 6),
            (String::from("R7"), 7),
            (String::from("R8"), 8),
            (String::from("R9"), 9),
            (String::from("R10"), 10),
            (String::from("R11"), 11),
            (String::from("R12"), 12),
            (String::from("R13"), 13),
            (String::from("R14"), 14),
            (String::from("R15"), 15),
            (String::from("SCREEN"), 16384),
            (String::from("KBD"), 24576),
            (String::from("SP"), 0),
            (String::from("LCL"), 1),
            (String::from("ARG"), 2),
            (String::from("THIS"), 3),
            (String::from("THAT"), 4),
        ]);

        Self {
            table: t,
            current_variable: 16,
        }
    }
}
