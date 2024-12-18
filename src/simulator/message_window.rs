use eframe::egui;

const MAX_MESSAGE_LEN: usize = 5000;

pub struct MessageWindow {
    messages: Vec<String>,
    pub is_opened_message_window: bool,
}

impl MessageWindow {
    pub fn new() -> Self {
        Self {
            messages: Vec::new(),
            is_opened_message_window: false,
        }
    }

    pub fn push_messages(&mut self, messages: &Vec<String>) {
        self.messages.extend_from_slice(messages);
        if self.messages.len() > MAX_MESSAGE_LEN {
            self.messages.drain(..self.messages.len() - MAX_MESSAGE_LEN);
        }
    }

    pub fn clear_messages(&mut self) {
        self.messages.clear();
    }

    pub fn show_window(&mut self, ctx: &egui::Context) {
        if !self.is_opened_message_window {
            return;
        }
        ctx.show_viewport_immediate(
            egui::ViewportId::from_hash_of("message_window"),
            egui::ViewportBuilder::default()
                .with_title("Emulator Messages")
                .with_inner_size([400.0, 480.0]),
            |ctx, _class| {
                egui::CentralPanel::default().show(ctx, |ui| {
                    ui.label(format!("message: {}", self.messages.len()));
                    ui.separator();

                    let text_style = egui::TextStyle::Body;
                    let row_height = ui.text_style_height(&text_style);
                    egui::ScrollArea::vertical()
                        .stick_to_bottom(true)
                        .auto_shrink(false)
                        .show_rows(ui, row_height, self.messages.len(), |ui, row_range| {
                            for row in row_range {
                                let text = &self.messages[row];
                                ui.label(text);
                            }
                        });

                    if ctx.input(|i| i.viewport().close_requested()) {
                        self.is_opened_message_window = false;
                    }
                });
            },
        )
    }

    pub fn show_open_close_button(&mut self, ui: &mut egui::Ui) {
        let text = if self.is_opened_message_window {
            "Close message window"
        } else {
            "Open message window"
        };
        if ui.button(text).clicked() {
            self.is_opened_message_window = !self.is_opened_message_window
        }
    }
}
