use imgui::Ui;

use crate::hack_cpu::CPUState;

pub const RED: [f32; 4] = [1.0, 0.0, 0.0, 1.0];

/// Represents a portion of the [CPUState], at which we can then instruct the execution to halt. This is
/// designed to be useful for debugging programs when running them on the emulator. The 4 different
/// enumerations depict the 4 different states that may be of interest, the 3 registers, and a
/// specific RAM address.
#[derive(Debug, PartialEq, Clone, Copy, Eq, Hash)]
pub enum Breakpoint {
    A(i16),
    D(i16),
    PC(u16),
    RAM(u16, i16),
}

impl Breakpoint {
    /// Draws the breakpoint, along with a `remove` button, to the list of breakpoints in the GUI.
    /// The returning of a boolean is designed to inform whether or not the `remove` button has
    /// been clicked.
    pub fn display(self: &Self, ui: &Ui, cpustate: &CPUState) -> bool {
        match self {
            Breakpoint::A(v) => {
                let text = format!("A: {v}");
                if &cpustate.a.0 == v {
                    ui.text_colored(RED, text);
                } else {
                    ui.text(text);
                }
            }
            Breakpoint::D(v) => {
                let text = format!("D: {v}");
                if &cpustate.d.0 == v {
                    ui.text_colored(RED, text);
                } else {
                    ui.text(text);
                }
            }
            Breakpoint::PC(v) => {
                let text = format!("PC: {v}");
                if &cpustate.pc == v {
                    ui.text_colored(RED, text);
                } else {
                    ui.text(text);
                }
            }
            Breakpoint::RAM(n, v) => {
                let text = format!("RAM[{n}]: {v}");
                if &cpustate.ram[*n as usize].0 == v {
                    ui.text_colored(RED, text);
                } else {
                    ui.text(text);
                }
            }
        }
        ui.same_line();
        match self {
            Breakpoint::A(v) => ui.button(format!("Remove##A{v}")),
            Breakpoint::D(v) => ui.button(format!("Remove##D{v}")),
            Breakpoint::PC(v) => ui.button(format!("Remove##PC{v}")),
            Breakpoint::RAM(n, v) => ui.button(format!("Remove##RAM{n}{v}")),
        }
    }
}

/// Represents a choice between the possible [Breakpoint]s. This is used for a radio button when
/// construction a new [Breakpoint].
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum BreakpointSelector {
    A,
    D,
    PC,
    RAM,
}
