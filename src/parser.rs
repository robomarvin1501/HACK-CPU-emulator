use std::fmt;

use regex::Regex;

use crate::instructions::{Instruction, A, C};
use crate::symbol_table::SymbolTable;

const COMMENT_BEGIN: &'static str = "//";
const LABEL_BEGIN: char = '(';
const LABEL_END: char = ')';
const VARIABLE_DECLARATION: char = '@';

pub const MAX_INSTRUCTIONS: usize = i16::MAX as usize;
pub const MAX_RAM: usize = 24577;

/// Represents an invalid line in the source code. Used for showing the user the error.
#[derive(Debug)]
pub enum LineParsingError {
    InvalidLine(u16, String),
}
impl fmt::Display for LineParsingError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Invalid line")
    }
}

/// Parses a series of lines that make up the source code for the program to be run.
pub fn parse(
    lines: [String; MAX_INSTRUCTIONS],
    address_table: &mut SymbolTable,
) -> Result<[Instruction; MAX_INSTRUCTIONS], LineParsingError> {
    let whitespace_cleaned_lines = clear_whitespace(lines);
    labels_and_variables(&whitespace_cleaned_lines, address_table);
    let mut parsed_lines: [Instruction; MAX_INSTRUCTIONS] =
        [const { Instruction::None }; MAX_INSTRUCTIONS];
    let mut offset = 0;
    for (i, line) in whitespace_cleaned_lines.iter().enumerate() {
        if line.is_empty() {
            continue;
        }

        // A instruction
        if line.starts_with(VARIABLE_DECLARATION) {
            // Unchecked unwrap is acceptable, since all the destinations are put into the address
            // table in labels_and_variables
            parsed_lines[i - offset] = Instruction::A(A::new(
                &address_table.table.get(&line[1..]).unwrap().to_string(),
            ));
        } else if line.starts_with(LABEL_BEGIN) && line.ends_with(LABEL_END) {
            offset += 1;
            // parsed_lines[i] = Instruction::Label(line[1..line.len() - 1].to_string());
        }
        // C instruction
        else {
            let temp_line = split_line(&line);
            let instruction;
            if temp_line.len() == 2 {
                if line.contains(';') {
                    instruction = Instruction::C(C::new("", temp_line[0], temp_line[1]));
                } else {
                    instruction = Instruction::C(C::new(temp_line[0], temp_line[1], ""));
                }
            } else {
                let dest = match address_table.table.get(temp_line[0]) {
                    Some(d) => d,
                    None => return Err(LineParsingError::InvalidLine(i as u16, line.to_owned())),
                };
                instruction = Instruction::C(C::new(&dest.to_string(), temp_line[1], temp_line[2]));
            }
            parsed_lines[i - offset] = instruction;
        }
    }
    Ok(parsed_lines)
}

/// Given a line which appears to be a C instruction, it splits the line on the chars that
/// delineate the parts of C instructions.
fn split_line(line: &String) -> Vec<&str> {
    let re = Regex::new(r"[ ,=;]").unwrap();
    re.split(line).collect()
}

/// Clears whitespace out of provided source code. Whitespace includes empty lines, and comments.
fn clear_whitespace(lines: [String; MAX_INSTRUCTIONS]) -> [String; MAX_INSTRUCTIONS] {
    let mut whitespace_cleaned_lines: [String; MAX_INSTRUCTIONS] =
        [const { String::new() }; MAX_INSTRUCTIONS];
    let mut count = 0;
    for (i, line) in lines.iter().enumerate() {
        if line.is_empty() || line.starts_with(COMMENT_BEGIN) {
            count += 1;
        } else if let Some(comment_index) = line.find(COMMENT_BEGIN) {
            let trimmed = &line[..comment_index].trim();
            whitespace_cleaned_lines[i - count] = trimmed.replace(' ', "").to_string();
        } else {
            whitespace_cleaned_lines[i - count] = line.replace(' ', "").to_string();
        }
    }
    whitespace_cleaned_lines
}

/// Given the source code, this scans it for labels and variables, and stores them, and their
/// representative addresses in the [SymbolTable].
fn labels_and_variables(lines: &[String; MAX_INSTRUCTIONS], address_table: &mut SymbolTable) {
    let mut labels_count: u16 = 0;
    // Add labels to address_table
    for (i, line) in lines.iter().enumerate() {
        if line.starts_with(LABEL_BEGIN) && line.ends_with(LABEL_END) {
            let label_name: String = line[1..line.len() - 1].to_string();
            address_table
                .table
                .insert(label_name, (i - labels_count as usize) as u16);
            labels_count += 1;
        }
    }

    // Add variables to address_table
    for line in lines {
        if line.is_empty() {
            continue;
        }
        let potential_var = line[1..].to_string();
        if line.starts_with(VARIABLE_DECLARATION)
            && !address_table.table.contains_key(&potential_var)
        {
            let pv = potential_var.parse::<u16>();
            match pv {
                Ok(r) => {
                    address_table.table.insert(potential_var, r);
                }
                Err(_) => {
                    address_table
                        .table
                        .insert(potential_var, address_table.current_variable);
                    address_table.current_variable += 1;
                }
            }
        }
    }
}
