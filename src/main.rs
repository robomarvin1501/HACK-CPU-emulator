use crate::hack_cpu::CPUState;
use crate::hack_gui::HackGUI;
use core::panic;
use instructions::Instruction;
use parser::{parse, MAX_INSTRUCTIONS};
use std::{env, fs, path::PathBuf, usize};
mod instructions;
mod parser;
mod symbol_table;
use glium::backend::Facade;

mod hack_cpu;
mod hack_gui;
mod support;

const ASM_FILE_EXTENSION: &'static str = "asm";
const SCREEN_WIDTH: usize = 512;
const SCREEN_HEIGHT: usize = 256;
const SCREEN_RATIO: f32 = 2.0;
const SCREEN_LOCATION: usize = 16384;
const SCREEN_LENGTH: usize = 8192;
const KBD_LOCATION: usize = 24576;
const INSTRUCTIONS_PER_REFRESH: usize = 100_000;

fn main() {
    let instructions = read_arg_file();

    let mut state = CPUState::new();
    let instructions = parse(instructions, &mut state.address_table);

    let num_labels = instructions
        .iter()
        .filter(|&x| match x {
            Instruction::Label(_) => true,
            _ => false,
        })
        .count();

    let cpu_display = std::rc::Rc::new(std::cell::RefCell::new(HackGUI {
        screen_texture_id: None,
        cpu: state,
        instructions: instructions,
        num_labels: num_labels,
        running: false,
    }));
    let cpu_display_clone = cpu_display.clone();

    support::init_with_startup(
        file!(),
        move |_ctx, renderer, display| {
            cpu_display_clone
                .borrow_mut()
                .register_textures(display.get_context(), renderer.textures())
                .expect("Failed to register textures");
        },
        move |_, ui, renderer, key| {
            cpu_display.borrow_mut().show_textures(ui, renderer, key);
        },
    );
}

fn read_arg_file() -> [String; MAX_INSTRUCTIONS] {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        panic!("Invalid usage, please use: cpuemulator <input path>")
    }
    let argument_path = env::args().nth(1).expect("No path provided");
    let argument_path = fs::canonicalize(&argument_path).expect("Invalid path provided");
    let input_path: PathBuf = if argument_path.is_dir() {
        panic!("Directories are not supported");
    } else {
        argument_path
    };

    if let Some(extension) = input_path.extension() {
        if extension.to_str().unwrap_or("").to_lowercase() != ASM_FILE_EXTENSION {
            panic!("Expected asm file, got {}", extension.to_str().unwrap());
        }
    }

    let input_path = PathBuf::from(input_path);

    let contents: String =
        fs::read_to_string(input_path).expect("Should have been able to read file");
    let instructions: Vec<String> = contents.split("\n").map(|s| s.trim().to_string()).collect();
    if instructions.len() > MAX_INSTRUCTIONS {
        panic!(
            "Too many instructions, expected a maximum of {}, got {}",
            MAX_INSTRUCTIONS,
            instructions.len()
        );
    }
    let mut ret: [String; MAX_INSTRUCTIONS] = [const { String::new() }; MAX_INSTRUCTIONS];
    for (i, instruction) in instructions.iter().enumerate() {
        ret[i] = instruction.to_string();
    }
    ret
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::hack_gui::hack_to_rgba;
    use std::num::Wrapping;

    #[test]
    fn truth_itself() {
        assert_eq!("Veer Gala woz 'ere", "Veer Gala woz 'ere");
    }

    #[test]
    fn test_speed() {
        let contents: String =
            fs::read_to_string("asm/AutoFill.asm").expect("Should have been able to read file");
        let instructions: Vec<String> =
            contents.split("\n").map(|s| s.trim().to_string()).collect();
        let mut s_instructions: [String; MAX_INSTRUCTIONS] =
            [const { String::new() }; MAX_INSTRUCTIONS];
        for (i, instruction) in instructions.iter().enumerate() {
            s_instructions[i] = instruction.to_string();
        }
        let mut cpu = CPUState::new();
        let instructions = parse(s_instructions, &mut cpu.address_table);

        for _ in 0..1000000000 {
            cpu.interpret(&instructions[cpu.pc as usize]);
            hack_to_rgba(&cpu.ram[SCREEN_LOCATION..SCREEN_LOCATION + SCREEN_LENGTH]);
        }

        for i in SCREEN_LOCATION..SCREEN_LOCATION + SCREEN_LENGTH {
            assert_eq!(cpu.ram[i], Wrapping(-1))
        }
    }
}
