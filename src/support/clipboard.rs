// This file is based on code from imgui-rs, originally licensed under MIT
// Modifications (c) 2025 8Oldr, distributed under GPLv3

use copypasta::{ClipboardContext, ClipboardProvider};
use imgui::ClipboardBackend;

pub struct ClipboardSupport(pub ClipboardContext);

pub fn init() -> Option<ClipboardSupport> {
    ClipboardContext::new().ok().map(ClipboardSupport)
}

impl ClipboardBackend for ClipboardSupport {
    fn get(&mut self) -> Option<String> {
        self.0.get_contents().ok()
    }
    fn set(&mut self, text: &str) {
        // ignore errors?
        let _ = self.0.set_contents(text.to_owned());
    }
}
