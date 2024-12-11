use crate::emulator;

use super::Simulator;
use eframe::egui::{self, ScrollArea, TextStyle, Vec2};
use rfd::AsyncFileDialog;
use std::sync::{Arc, Mutex};

const MAX_MESSAGE_LEN: usize = 5000;

pub struct SimulatorUiStates {
    elf_path: Arc<Mutex<String>>,
    elf_args: String,
    messages: Vec<String>,
}

impl SimulatorUiStates {
    pub fn new() -> Self {
        SimulatorUiStates {
            elf_path: Arc::new(Mutex::new(String::new())),
            elf_args: String::new(),
            messages: Vec::new(),
        }
    }

    pub fn push_messages(&mut self, messages: &Vec<String>) {
        self.messages.extend_from_slice(messages);
        if self.messages.len() > MAX_MESSAGE_LEN {
            self.messages.drain(..self.messages.len() - MAX_MESSAGE_LEN);
        }
    }
}

fn select_elf(elf_path: Arc<Mutex<String>>) {
    tokio::spawn(async move {
        let file = AsyncFileDialog::new()
            .add_filter("elf", &["elf"])
            .pick_file()
            .await;
        if let Some(fi) = file {
            if let Some(path) = fi.path().to_str() {
                let lock = elf_path.lock();
                if let Ok(mut l) = lock {
                    *l = path.to_string()
                }
            }
        }
    });
}

impl Simulator {
    pub fn ui(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        self.update();

        if ui.button("Select elf").clicked() {
            select_elf(self.ui_states.elf_path.clone());
        }

        ui.add_sized(
            Vec2::new(ui.available_width(), 0f32),
            egui::TextEdit::singleline(&mut *self.ui_states.elf_path.lock().unwrap()),
        );

        ui.add_space(4.0);

        ui.label("Args");
        ui.with_layout(egui::Layout::left_to_right(egui::Align::TOP), |ui| {
            ui.label("prog.elf");
            ui.add_sized(
                Vec2::new(ui.available_width(), 0f32),
                egui::TextEdit::singleline(&mut self.ui_states.elf_args),
            )
        });

        ui.add_space(4.0);

        if self.emulator.is_none() {
            if ui.button("execute").clicked() {
                self.ui_states.messages.clear();

                let (tx, rx) = std::sync::mpsc::channel();
                self.emulator_exec_rx = Some(rx);
                let _elf_path = self.ui_states.elf_path.clone().lock().unwrap().clone();
                let _elf_args = self.ui_states.elf_args.clone();
                let _ctx = ctx.clone();
                tokio::spawn(async move {
                    let emu = emulator::Emulator::execute(_elf_path, _elf_args, _ctx).await;
                    tx.send(emu).unwrap();
                });
            }
        } else {
            if ui.button("stop").clicked() {
                self.emulator = None;
            }
        }

        if self.emulator.is_some() {
            ui.label("Emulator is available.");
        } else {
            ui.label("Emulator is None.");
        }

        ui.separator();

        self.show_messages(ui);
    }

    fn show_messages(&self, ui: &mut egui::Ui) {
        ui.label(format!("message: {}", self.ui_states.messages.len()));

        let text_style = TextStyle::Body;
        let row_height = ui.text_style_height(&text_style);
        ScrollArea::vertical()
            .stick_to_bottom(true)
            .auto_shrink(false)
            .show_rows(
                ui,
                row_height,
                self.ui_states.messages.len(),
                |ui, row_range| {
                    for row in row_range {
                        let text = &self.ui_states.messages[row];
                        ui.label(text);
                    }
                },
            );
    }
}
