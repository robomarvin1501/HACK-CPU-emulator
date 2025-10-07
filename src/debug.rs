use imgui::Ui;

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Breakpoint {
    A(i16),
    D(i16),
    PC(u16),
    RAM(u16, i16),
}

impl Breakpoint {
    pub fn display(self: &Self, ui: &Ui) -> bool {
        match self {
            Breakpoint::A(v) => {
                ui.text(format!("A: {v}"));
            }
            Breakpoint::D(v) => {
                ui.text(format!("D: {v}"));
            }
            Breakpoint::PC(v) => {
                ui.text(format!("PC: {v}"));
            }
            Breakpoint::RAM(n, v) => {
                ui.text(format!("RAM[{n}]: {v}"));
            }
        }
        ui.same_line();
        ui.button("Remove")
    }
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum BreakpointSelector {
    A,
    D,
    PC,
    RAM,
}
