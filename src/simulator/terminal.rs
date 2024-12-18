use eframe::egui;

pub struct Terminal {
    lines: Vec<String>,
    should_clear: bool,
}

impl Terminal {
    pub fn new() -> Self {
        Self {
            lines: vec![String::new()],
            should_clear: true,
        }
    }

    pub fn push(&mut self, string: &String) {
        for c in string.chars() {
            match c {
                '\r' => continue,
                '\n' => self.lines.push(String::new()),
                _ => {
                    let i = self.lines.len() - 1;
                    self.lines[i].push(c);
                }
            }
        }
    }

    pub fn clear(&mut self) {
        if self.should_clear {
            self.lines.clear();
            self.lines.push(String::new());
        }
    }

    pub fn show(&mut self, ui: &mut egui::Ui) {
        ui.strong("Terminal (stdout)");
        ui.checkbox(&mut self.should_clear, "Clear on start");
        let text_style = egui::TextStyle::Body;
        let row_height = ui.text_style_height(&text_style);
        egui::ScrollArea::vertical()
            .stick_to_bottom(true)
            .auto_shrink(false)
            .show_rows(ui, row_height, self.lines.len(), |ui, row_range| {
                for row in row_range {
                    let text = &self.lines[row];
                    ui.label(text);
                }
            });
    }
}
