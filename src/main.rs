use core::panic;
use instructions::{Comp, Destination, Instruction, Jump, A, C};
use parser::{parse, MAX_INSTRUCTIONS};
use std::{
    env,
    error::Error,
    fmt::Debug,
    fs,
    io::Read,
    num::Wrapping,
    ops::{Neg, Not},
    path::PathBuf,
    usize,
};
mod instructions;
mod parser;
mod symbol_table;
use glium::{
    backend::Facade,
    texture::{ClientFormat, RawImage2d},
    uniforms::{MagnifySamplerFilter, MinifySamplerFilter, SamplerBehavior},
    Texture2d,
};
use imgui::*;
use imgui_glium_renderer::Texture;
use std::borrow::Cow;
use std::io::Cursor;
use std::rc::Rc;

mod support;

const ASM_FILE_EXTENSION: &'static str = "asm";
const SCREEN_WIDTH: usize = 512;
const SCREEN_HEIGHT: usize = 256;
const SCREEN_LOCATION: usize = 16384;
const SCREEN_LENGTH: usize = 8192;

fn main() {
    let instructions = read_arg_file();

    let mut state = CPUState::new();
    let instructions = parse(instructions, &mut state.address_table);

    let cpu_display = std::rc::Rc::new(std::cell::RefCell::new(HackGUI {
        screen_texture_id: None,
        cpu: state,
        instructions: instructions,
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
        move |_, ui| {
            cpu_display.borrow_mut().show_textures(ui);
        },
    );

    // let instructions = parse(instructions, &mut state.address_table);
    // dbg!(&instructions);

    // while state.pc < MAX_INSTRUCTIONS as u16 {
    //     dbg!(&instructions[state.pc as usize]);
    //     state.interpret(&instructions[state.pc as usize]);
    //     dbg!(&state.ram[0..20]);
    //     std::io::stdin().bytes().next();
    // }
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

#[derive(Debug)]
struct CPUState {
    a: Wrapping<i16>,
    d: Wrapping<i16>,
    pc: u16,
    ram: [Wrapping<i16>; MAX_INSTRUCTIONS],
    address_table: symbol_table::SymbolTable,
}

impl CPUState {
    pub fn new() -> Self {
        Self {
            a: Wrapping(0),
            d: Wrapping(0),
            pc: 0,
            ram: std::array::from_fn(|_| Wrapping(0)),
            address_table: symbol_table::SymbolTable::new(),
        }
    }

    pub fn interpret(self: &mut Self, instruction: &Instruction) {
        match instruction {
            Instruction::A(a) => self.a_instruction(&a),
            Instruction::C(c) => self.c_instruction(&c),
            Instruction::Label() | Instruction::None => self.pc += 1,
        }
    }

    fn a_instruction(self: &mut Self, a: &A) {
        let destination = self.address_table.table.get(&a.dest);
        match destination {
            Some(loc) => self.a = Wrapping((*loc) as i16),
            None => panic!("Invalid instruction: {:?}", a),
        }
        self.pc += 1;
    }

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
}

struct HackGUI {
    screen_texture_id: Option<TextureId>,
    cpu: CPUState,
    instructions: [Instruction; MAX_INSTRUCTIONS],
}

impl HackGUI {
    fn register_textures<F>(
        &mut self,
        gl_ctx: &F,
        textures: &mut Textures<Texture>,
    ) -> Result<(), Box<dyn Error>>
    where
        F: Facade,
    {
        if self.screen_texture_id.is_none() {
            let texture = generate_screen_texture(&self.cpu, gl_ctx)?;
            let texture_id = textures.insert(texture);

            self.screen_texture_id = Some(texture_id);
        }

        Ok(())
    }

    fn show_textures(&mut self, ui: &Ui) {
        ui.window("Controls")
            .size([100.0, 100.0], Condition::FirstUseEver)
            .build(|| {
                if ui.button("Step") {
                    self.cpu.interpret(&self.instructions[self.cpu.pc as usize]);
                    //     dbg!(&instructions[state.pc as usize]);
                    //     state.interpret(&instructions[state.pc as usize]);
                    //     dbg!(&state.ram[0..20]);
                    //     std::io::stdin().bytes().next();
                }
            });
        // TODO not going to update with each frame, need to draw the texture here
        ui.window("Screen")
            .size(
                [SCREEN_WIDTH as f32, SCREEN_HEIGHT as f32],
                Condition::FirstUseEver,
            )
            .build(|| {
                if let Some(my_texture_id) = self.screen_texture_id {
                    Image::new(my_texture_id, [SCREEN_WIDTH as f32, SCREEN_HEIGHT as f32])
                        .build(ui);
                }
            });

        ui.window("Program view")
            .size([100.0, 500.0], Condition::FirstUseEver)
            .build(|| {
                let num_cols = 2;
                let num_rows = MAX_INSTRUCTIONS as i32;

                let flags = imgui::TableFlags::ROW_BG
                    | imgui::TableFlags::RESIZABLE
                    | imgui::TableFlags::BORDERS_H
                    | imgui::TableFlags::BORDERS_V;

                if let Some(_t) =
                    ui.begin_table_with_sizing("longtable", num_cols, flags, [300.0, 100.0], 0.0)
                {
                    ui.table_setup_column("");
                    ui.table_setup_column("Instructions");

                    // Freeze first row so headers are visible when scrolling
                    ui.table_setup_scroll_freeze(num_cols, 1);

                    ui.table_headers_row();

                    let clip = imgui::ListClipper::new(num_rows).begin(ui);
                    for row_num in clip.iter() {
                        ui.table_next_row();
                        ui.table_set_column_index(0);
                        ui.text(format!("{}", row_num));
                        ui.table_set_column_index(1);
                        ui.text(format!("{}", 0));
                    }
                }
            });

        ui.window("Memory view")
            .size([100.0, 500.0], Condition::FirstUseEver)
            .build(|| {
                let num_cols = 2;
                let num_rows = MAX_INSTRUCTIONS as i32;

                let flags = imgui::TableFlags::ROW_BG
                    | imgui::TableFlags::RESIZABLE
                    | imgui::TableFlags::BORDERS_H
                    | imgui::TableFlags::BORDERS_V;

                if let Some(_t) =
                    ui.begin_table_with_sizing("longtable", num_cols, flags, [300.0, 100.0], 0.0)
                {
                    ui.table_setup_column("");
                    ui.table_setup_column("Memory");

                    // Freeze first row so headers are visible when scrolling
                    ui.table_setup_scroll_freeze(num_cols, 1);

                    ui.table_headers_row();

                    let clip = imgui::ListClipper::new(num_rows).begin(ui);
                    for row_num in clip.iter() {
                        ui.table_next_row();
                        ui.table_set_column_index(0);
                        ui.text(format!("{}", row_num));
                        ui.table_set_column_index(1);
                        ui.text(format!("{}", 0));
                    }
                }
            });
    }
}

fn generate_screen_texture<F>(cpu: &CPUState, gl_ctx: &F) -> Result<Texture, Box<dyn Error>>
where
    F: Facade,
{
    // Generate dummy texture
    let screen_contents_data =
        hack_to_rgba(&cpu.ram[SCREEN_LOCATION..SCREEN_LOCATION + SCREEN_LENGTH]);

    let raw = RawImage2d {
        data: Cow::Owned(screen_contents_data),
        width: SCREEN_WIDTH as u32,
        height: SCREEN_HEIGHT as u32,
        format: ClientFormat::U8U8U8,
    };
    let gl_texture = Texture2d::new(gl_ctx, raw)?;
    let texture = Texture {
        texture: Rc::new(gl_texture),
        sampler: SamplerBehavior {
            magnify_filter: MagnifySamplerFilter::Linear,
            minify_filter: MinifySamplerFilter::Linear,
            ..Default::default()
        },
    };
    return Ok(texture);
}

fn get_pixel(screen: &[Wrapping<i16>], row: usize, col: usize) -> bool {
    let word_index = row * 32 + (col / 16);
    let bit_index = col % 16;
    let word = screen[word_index].0 as u16;
    ((word >> bit_index) & 1) == 1
}

fn hack_to_rgba(screen: &[Wrapping<i16>]) -> Vec<u8> {
    let mut framebuffer = Vec::with_capacity(SCREEN_WIDTH * SCREEN_HEIGHT);
    for col in 0..SCREEN_WIDTH {
        for row in 0..SCREEN_HEIGHT {
            let p = get_pixel(screen, row, col);
            let colour = if p { 0u8 } else { 255u8 };
            // Insert RGB values
            framebuffer.push(colour);
            framebuffer.push(colour);
            framebuffer.push(colour);
        }
    }
    framebuffer
}
