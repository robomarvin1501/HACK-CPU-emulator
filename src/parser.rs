use regex::Regex;

use crate::symbol_table::SymbolTable;
use crate::instructions::{Instruction, A, C};

const COMMENT_BEGIN: &'static str = "//";
const LABEL_BEGIN: char = '(';
const LABEL_END: char = ')';
const VARIABLE_DECLARATION: char = '@';

pub fn parse(lines: &Vec<String>, address_table: &mut SymbolTable) -> Vec<Instruction> {
    let whitespace_cleaned_lines = clear_whitespace(lines);
    labels_and_variables(&whitespace_cleaned_lines, address_table);

    let mut parsed_lines: Vec<Instruction> = vec![];
    for line in whitespace_cleaned_lines {
        // A instruction
        if line.starts_with(VARIABLE_DECLARATION) {
            parsed_lines.push(Instruction::A(A::new(&line[1..])));
        } else if line.starts_with(LABEL_BEGIN) && line.ends_with(LABEL_END) {
            parsed_lines.push(Instruction::Label());
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
                instruction = Instruction::C(C::new(temp_line[0], temp_line[1], temp_line[2]));
            }
            parsed_lines.push(instruction);
        }
    }
    parsed_lines
}

fn split_line(line: &String) -> Vec<&str> {
    let re = Regex::new(r"[ ,=;]").unwrap();
    re.split(line).collect()
}

fn clear_whitespace(lines: &Vec<String>) -> Vec<String> {
    let mut whitespace_cleaned_lines: Vec<String> = vec![];
    for line in lines {
        if line.is_empty() || line.starts_with(COMMENT_BEGIN) {
        } else if let Some(comment_index) = line.find(COMMENT_BEGIN) {
            let trimmed = &line[..comment_index].trim();
            whitespace_cleaned_lines.push(trimmed.replace(' ', ""));
        } else {
            whitespace_cleaned_lines.push(line.replace(' ', ""));
        }
    }
    whitespace_cleaned_lines
}

fn labels_and_variables(lines: &Vec<String>, address_table: &mut SymbolTable) {
    let mut labels_count: u16 = 0;
    // Add labels to address_table
    for (i, line) in lines.iter().enumerate() {
        if line.starts_with(LABEL_BEGIN) && line.ends_with(LABEL_END) {
            let label_name: String = line[1..line.len() - 1].to_string();
            address_table
                .table
                .insert(label_name, i as u16 - labels_count);
            labels_count += 1;
        }
    }

    // Add variables to address_table
    for line in lines {
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
