use crate::debug::{Breakpoint, BreakpointSelector};
use crate::instructions::Instruction;
use crate::{CPUState, SCREEN_RATIO};
use crate::{
    INSTRUCTIONS_PER_REFRESH, KBD_LOCATION, MAX_INSTRUCTIONS, SCREEN_HEIGHT, SCREEN_LENGTH,
    SCREEN_LOCATION, SCREEN_WIDTH,
};
use glium::{
    backend::Facade,
    texture::{ClientFormat, RawImage2d},
    uniforms::{MagnifySamplerFilter, MinifySamplerFilter, SamplerBehavior},
    winit::keyboard::{Key, NamedKey},
    Texture2d,
};
use imgui::*;
use imgui_glium_renderer::{Renderer, Texture};
use std::borrow::Cow;
use std::rc::Rc;
use std::{error::Error, num::Wrapping, usize};

const RAM_AND_ROM_WIDTH: f32 = 350.0;
const CONTROL_WINDOW_HEIGHT: f32 = 155.0;

// Key codes
const NEWLINE_KEY: i16 = 128;
const BACKSPACE_KEY: i16 = 129;
const LEFT_KEY: i16 = 130;
const UP_KEY: i16 = 131;
const RIGHT_KEY: i16 = 132;
const DOWN_KEY: i16 = 133;
const HOME_KEY: i16 = 134;
const END_KEY: i16 = 135;
const PAGE_UP_KEY: i16 = 136;
const PAGE_DOWN_KEY: i16 = 137;
const INSERT_KEY: i16 = 138;
const DELETE_KEY: i16 = 139;
const ESC_KEY: i16 = 140;
const F1_KEY: i16 = 141;
const F2_KEY: i16 = 142;
const F3_KEY: i16 = 143;
const F4_KEY: i16 = 144;
const F5_KEY: i16 = 145;
const F6_KEY: i16 = 146;
const F7_KEY: i16 = 147;
const F8_KEY: i16 = 148;
const F9_KEY: i16 = 149;
const F10_KEY: i16 = 150;
const F11_KEY: i16 = 151;
const F12_KEY: i16 = 152;

pub struct HackGUI {
    pub screen_texture_id: Option<TextureId>,
    pub cpu: CPUState,
    pub instructions: [Instruction; MAX_INSTRUCTIONS],
    pub num_labels: usize,
    pub running: bool,
    next_breakpoint: Option<BreakpointSelector>,
    adram_value: i16,
    pcvalue: u16,
}

impl HackGUI {
    pub fn new(
        screen_texture_id: Option<TextureId>,
        cpu: CPUState,
        instructions: [Instruction; MAX_INSTRUCTIONS],
        num_labels: usize,
    ) -> Self {
        Self {
            screen_texture_id,
            cpu,
            instructions,
            num_labels,
            running: false,
            next_breakpoint: None,
            adram_value: 0,
            pcvalue: 0,
        }
    }
    pub fn register_textures<F>(
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

    pub fn show_textures(&mut self, ui: &Ui, renderer: &mut Renderer, key: &Option<Key>) {
        let [screen_width, screen_height] = ui.io().display_size;
        ui.window("CPU Emulator")
            .size([screen_width, screen_height], Condition::Always)
            .position([0.0, 0.0], Condition::Always)
            .movable(false)
            .collapsible(false)
            .resizable(true)
            .build(|| {
                ui.columns(2, "control_cols", true);
                ui.child_window("Controls")
                    .size([0.0, CONTROL_WINDOW_HEIGHT])
                    .build(|| {
                        let fm = ui.io().framerate;
                        ui.text(format!("Framerate: {}", fm));
                        let stop_ui = ui.begin_disabled(!self.running);
                        if ui.button("Stop") {
                            self.running = false;
                        }
                        stop_ui.end();
                        let running_ui = ui.begin_disabled(self.running);
                        if ui.button("Run") {
                            self.running = true;
                        }
                        if ui.button("Step") {
                            self.cpu.interpret(&self.instructions[self.cpu.pc as usize]);
                            if let Some(kbd_letter) = key {
                                self.cpu.ram[KBD_LOCATION] = get_keycode(kbd_letter);
                            } else {
                                self.cpu.ram[KBD_LOCATION] = Wrapping(0);
                            }
                        }
                        if ui.button("Reset") {
                            self.cpu.pc = 0;
                        }
                        running_ui.end();

                        if self.running {
                            'instructions: for _ in 0..INSTRUCTIONS_PER_REFRESH {
                                self.cpu.interpret(&self.instructions[self.cpu.pc as usize]);
                                for breakpoint in &self.cpu.breakpoints {
                                    match breakpoint {
                                        Breakpoint::A(v) => {
                                            if self.cpu.a.0 == *v {
                                                self.running = false;
                                                break 'instructions;
                                            }
                                        }
                                        Breakpoint::D(v) => {
                                            if self.cpu.d.0 == *v {
                                                self.running = false;
                                                break 'instructions;
                                            }
                                        }
                                        Breakpoint::PC(v) => {
                                            if self.cpu.pc == *v {
                                                self.running = false;
                                                break 'instructions;
                                            }
                                        }
                                        Breakpoint::RAM(n, v) => {
                                            if self.cpu.ram[*n as usize].0 == *v {
                                                self.running = false;
                                                break 'instructions;
                                            }
                                        }
                                    }
                                }
                            }
                            if let Some(kbd_letter) = key {
                                self.cpu.ram[KBD_LOCATION] = get_keycode(kbd_letter);
                            } else {
                                self.cpu.ram[KBD_LOCATION] = Wrapping(0);
                            }
                        }
                    });
                ui.next_column();
                ui.child_window("Debug")
                    .size([0.0, CONTROL_WINDOW_HEIGHT])
                    .build(|| {
                        ui.text("Debug");
                        ui.radio_button(
                            "A",
                            &mut self.next_breakpoint,
                            Some(BreakpointSelector::A),
                        );
                        ui.radio_button(
                            "D",
                            &mut self.next_breakpoint,
                            Some(BreakpointSelector::D),
                        );
                        ui.radio_button(
                            "PC",
                            &mut self.next_breakpoint,
                            Some(BreakpointSelector::PC),
                        );
                        ui.radio_button(
                            "RAM",
                            &mut self.next_breakpoint,
                            Some(BreakpointSelector::RAM),
                        );

                        if let Some(bs) = self.next_breakpoint {
                            match bs {
                                BreakpointSelector::A => {
                                    ui.text("A: ");
                                    ui.same_line();
                                    let val = &mut self.adram_value;
                                    let mut temp = *val as i32;
                                    if ui.input_int("##input_a", &mut temp).build() {
                                        *val = temp as _;
                                    }
                                }
                                BreakpointSelector::D => {
                                    ui.text("D: ");
                                    ui.same_line();
                                    let val = &mut self.adram_value;
                                    let mut temp = *val as i32;
                                    if ui.input_int("##input_d", &mut temp).build() {
                                        *val = temp as _;
                                    }
                                }
                                BreakpointSelector::PC => {
                                    ui.text("PC: ");
                                    ui.same_line();
                                    let val = &mut self.pcvalue;
                                    let mut temp = *val as i32;
                                    if ui.input_int("##input_pc", &mut temp).build() {
                                        *val = temp as _;
                                    }
                                }
                                BreakpointSelector::RAM => {
                                    ui.text("RAM: ");
                                    ui.same_line();
                                    let val = &mut self.pcvalue;
                                    let mut temp = *val as i32;
                                    if ui.input_int("##input_ram_target", &mut temp).build() {
                                        *val = temp as _;
                                    }
                                    ui.same_line();
                                    let val = &mut self.adram_value;
                                    let mut temp = *val as i32;
                                    if ui.input_int("##input_ram_value", &mut temp).build() {
                                        *val = temp as _;
                                    }
                                }
                            }
                            if ui.button("Add breakpoint") {
                                match bs {
                                    BreakpointSelector::A => {
                                        self.cpu
                                            .breakpoints
                                            .insert(Breakpoint::A(self.adram_value));
                                    }
                                    BreakpointSelector::D => {
                                        self.cpu
                                            .breakpoints
                                            .insert(Breakpoint::D(self.adram_value));
                                    }
                                    BreakpointSelector::PC => {
                                        self.cpu.breakpoints.insert(Breakpoint::PC(self.pcvalue));
                                    }
                                    BreakpointSelector::RAM => {
                                        self.cpu.breakpoints.insert(Breakpoint::RAM(
                                            self.pcvalue,
                                            self.adram_value,
                                        ));
                                    }
                                }
                                self.adram_value = 0;
                                self.pcvalue = 0;
                            }
                        }
                    });
                ui.separator();

                ui.columns(3, "main_cols", true);
                ui.set_column_width(0, RAM_AND_ROM_WIDTH);
                ui.child_window("ROM")
                    .child_flags(ChildFlags::BORDERS)
                    .build(|| {
                        ui.text("ROM");
                        let running_ui = ui.begin_disabled(self.running);
                        let val = &mut self.cpu.pc;
                        let mut temp = *val as i32;
                        ui.text("PC: ");
                        ui.same_line();
                        if ui.input_int("##pc", &mut temp).build() {
                            *val = temp as _;
                        }
                        let num_cols = 2;
                        let num_rows = (MAX_INSTRUCTIONS + self.num_labels) as i32;

                        let flags = imgui::TableFlags::ROW_BG
                            | imgui::TableFlags::RESIZABLE
                            | imgui::TableFlags::BORDERS_H
                            | imgui::TableFlags::BORDERS_V;

                        if let Some(_t) = ui.begin_table_with_sizing(
                            "longtable",
                            num_cols,
                            flags,
                            [-1.0, 0.0],
                            0.0,
                        ) {
                            ui.table_setup_column("");
                            ui.table_setup_column("Instructions");

                            // Freeze first row so headers are visible when scrolling
                            ui.table_setup_scroll_freeze(num_cols, 1);

                            ui.table_headers_row();

                            let clip = imgui::ListClipper::new(num_rows).begin(ui);
                            let mut offset = 0;
                            for row_num in clip.iter() {
                                ui.table_next_row();
                                ui.table_set_column_index(0);
                                if (row_num - offset) as u16 == self.cpu.pc {
                                    ui.table_set_bg_color(
                                        TableBgTarget::ROW_BG1,
                                        ImColor32::from_rgb(100, 100, 0),
                                    );
                                }
                                match self.instructions[row_num as usize] {
                                    Instruction::Label(_) => {
                                        offset += 1;
                                        ui.text("");
                                        ui.table_set_column_index(1);
                                        ui.text(format!("{}", self.instructions[row_num as usize]));
                                    }
                                    Instruction::A(_) | Instruction::C(_) | Instruction::None => {
                                        ui.text(format!("{}", row_num - offset));
                                        ui.table_set_column_index(1);
                                        ui.text(format!("{}", self.instructions[row_num as usize]));
                                    }
                                }
                            }
                        }
                        running_ui.end();
                    });

                ui.next_column();
                ui.set_column_width(1, RAM_AND_ROM_WIDTH);
                ui.child_window("RAM")
                    .child_flags(ChildFlags::BORDERS)
                    .build(|| {
                        ui.text("RAM");
                        let running_ui = ui.begin_disabled(self.running);
                        let val = &mut self.cpu.a.0;
                        let mut temp = *val as i32;
                        ui.text("A: ");
                        ui.same_line();
                        if ui.input_int("##a", &mut temp).build() {
                            *val = temp as _;
                        }
                        let num_cols = 2;
                        let num_rows = MAX_INSTRUCTIONS as i32;

                        let flags = imgui::TableFlags::ROW_BG
                            | imgui::TableFlags::RESIZABLE
                            | imgui::TableFlags::BORDERS_H
                            | imgui::TableFlags::BORDERS_V;

                        if let Some(_t) = ui.begin_table_with_sizing(
                            "longtable",
                            num_cols,
                            flags,
                            [-1.0, 0.0],
                            0.0,
                        ) {
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
                                if !self.running && row_num == self.cpu.a.0 as i32 {
                                    ui.table_set_bg_color(
                                        TableBgTarget::ROW_BG1,
                                        ImColor32::from_rgb(100, 100, 0),
                                    );
                                }

                                ui.table_set_column_index(1);
                                let val = &mut self.cpu.ram[row_num as usize].0;
                                let mut temp = *val as i32;
                                if ui.input_int(format!("##ram{}", row_num), &mut temp).build() {
                                    *val = temp as _;
                                }
                            }
                        }
                        running_ui.end();
                    });
                ui.next_column();

                let rem_width = ui.content_region_avail()[0];
                let height = rem_width / SCREEN_RATIO;
                ui.child_window("Screen pane")
                    .size([0.0, 0.0])
                    .child_flags(ChildFlags::BORDERS)
                    .build(|| {
                        ui.text("Screen");

                        if let Some(sti) = self.screen_texture_id {
                            if let Some(st) = renderer.textures().get_mut(sti) {
                                let screen_contents = hack_to_rgba(
                                    &self.cpu.ram[SCREEN_LOCATION..SCREEN_LOCATION + SCREEN_LENGTH],
                                );
                                let raw = RawImage2d {
                                    data: Cow::Owned(screen_contents),
                                    width: SCREEN_WIDTH as u32,
                                    height: SCREEN_HEIGHT as u32,
                                    format: ClientFormat::U8U8U8,
                                };
                                st.texture.write(
                                    glium::Rect {
                                        left: 0,
                                        bottom: 0,
                                        width: SCREEN_WIDTH as u32,
                                        height: SCREEN_HEIGHT as u32,
                                    },
                                    raw,
                                );
                            }
                            Image::new(sti, [rem_width, height]).build(ui);
                        };
                        let running_ui = ui.begin_disabled(self.running);
                        let val = &mut self.cpu.d.0;
                        let mut temp = *val as i32;
                        ui.text("D: ");
                        ui.same_line();
                        if ui.input_int("##d", &mut temp).build() {
                            *val = temp as _;
                        }
                        running_ui.end();

                        ui.child_window("Breakpoints")
                            .child_flags(ChildFlags::BORDERS)
                            .build(|| {
                                ui.text("Breakpoints");
                                let mut to_remove: Vec<Breakpoint> = vec![];
                                for breakpoint in self.cpu.breakpoints.iter() {
                                    if breakpoint.display(&ui) {
                                        to_remove.push(*breakpoint);
                                    }
                                }
                                for breakpoint in to_remove {
                                    self.cpu.breakpoints.remove(&breakpoint);
                                }
                            })
                    });
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

pub fn hack_to_rgba(screen: &[Wrapping<i16>]) -> Vec<u8> {
    // Preallocate fully: each pixel → 3 bytes (RGB)
    let mut framebuffer = vec![255u8; SCREEN_WIDTH * SCREEN_HEIGHT * 3];

    // Each row has 32 words, each word = 16 horizontal pixels
    for row in 0..SCREEN_HEIGHT {
        for word_index in 0..32 {
            let word = screen[row * 32 + word_index].0 as u16; // cast to unsigned for shift safety

            // Precompute base offset in framebuffer
            let base = (row * SCREEN_WIDTH + word_index * 16) * 3;

            // Iterate bits (col within this word)
            for bit in 0..16 {
                // Hack screen convention: LSB is leftmost
                if (word >> bit) & 1 == 1 {
                    let offset = base + bit * 3;
                    framebuffer[offset] = 0;
                    framebuffer[offset + 1] = 0;
                    framebuffer[offset + 2] = 0;
                }
            }
        }
    }

    framebuffer
}

fn get_keycode(key: &Key) -> Wrapping<i16> {
    match key.to_owned() {
        Key::Character(c) => {
            if c.len() == 1 {
                let ch = c.chars().next().unwrap();
                let key_code = ch as i16;

                if ch.is_ascii_uppercase() || ch.is_ascii_lowercase() {
                    Wrapping(key_code)
                } else {
                    match key_code {
                        BACKSPACE_KEY => Wrapping(BACKSPACE_KEY),
                        NEWLINE_KEY => Wrapping(NEWLINE_KEY),
                        ESC_KEY => Wrapping(ESC_KEY),
                        DELETE_KEY => Wrapping(DELETE_KEY),
                        _ => Wrapping(key_code),
                    }
                }
            } else {
                // Should not occur
                Wrapping(0)
            }
        }
        Key::Named(n) => match n {
            NamedKey::Space => Wrapping(32),
            NamedKey::Backspace => Wrapping(BACKSPACE_KEY),
            NamedKey::Enter => Wrapping(NEWLINE_KEY),
            NamedKey::Escape => Wrapping(ESC_KEY),
            NamedKey::Delete => Wrapping(DELETE_KEY),
            NamedKey::ArrowLeft => Wrapping(LEFT_KEY),
            NamedKey::ArrowRight => Wrapping(RIGHT_KEY),
            NamedKey::ArrowUp => Wrapping(UP_KEY),
            NamedKey::ArrowDown => Wrapping(DOWN_KEY),
            NamedKey::PageUp => Wrapping(PAGE_UP_KEY),
            NamedKey::PageDown => Wrapping(PAGE_DOWN_KEY),
            NamedKey::Home => Wrapping(HOME_KEY),
            NamedKey::End => Wrapping(END_KEY),
            NamedKey::F1 => Wrapping(F1_KEY),
            NamedKey::F2 => Wrapping(F2_KEY),
            NamedKey::F3 => Wrapping(F3_KEY),
            NamedKey::F4 => Wrapping(F4_KEY),
            NamedKey::F5 => Wrapping(F5_KEY),
            NamedKey::F6 => Wrapping(F6_KEY),
            NamedKey::F7 => Wrapping(F7_KEY),
            NamedKey::F8 => Wrapping(F8_KEY),
            NamedKey::F9 => Wrapping(F9_KEY),
            NamedKey::F10 => Wrapping(F10_KEY),
            NamedKey::F11 => Wrapping(F11_KEY),
            NamedKey::F12 => Wrapping(F12_KEY),
            NamedKey::Insert => Wrapping(INSERT_KEY),
            _ => Wrapping(0),
        },
        _ => Wrapping(0),
    }
}
