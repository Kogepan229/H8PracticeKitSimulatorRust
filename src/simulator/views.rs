use super::{registers, Simulator};
use eframe::egui::{self, Vec2};
use egui_extras::Column;
use rfd::AsyncFileDialog;
use std::sync::{Arc, Mutex};

pub struct SimulatorUiStates {
    pub elf_path: Arc<Mutex<String>>,
    pub elf_args: String,
    pub speed: f32,
}

impl SimulatorUiStates {
    pub fn new() -> Self {
        SimulatorUiStates {
            elf_path: Arc::new(Mutex::new(String::new())),
            elf_args: String::new(),
            speed: 0f32,
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

        ui.strong("Args");
        ui.with_layout(egui::Layout::left_to_right(egui::Align::TOP), |ui| {
            ui.label("prog.elf");
            ui.add_sized(
                Vec2::new(ui.available_width(), 0f32),
                egui::TextEdit::singleline(&mut self.ui_states.elf_args),
            )
        });

        ui.add_space(4.0);

        ui.horizontal_wrapped(|ui| {
            if self.emulator.is_none() {
                ui.add_enabled_ui(self.emulator_exec_rx.is_none(), |ui| {
                    if ui.button("execute").clicked() {
                        self.execute_emulator(ctx);
                    }
                });
            } else {
                if ui.button("stop").clicked() {
                    self.stop_emulator();
                }
            }
            ui.with_layout(egui::Layout::right_to_left(egui::Align::TOP), |ui| {
                self.message_window.show_open_close_button(ui);
            })
        });

        if self.emulator.is_some() {
            ui.label("Emulator is running.");
        } else {
            ui.label("Emulator is stopped.");
        }
        ui.label(format!("Speed: x{}", self.ui_states.speed));

        ui.separator();

        self.show_modules(ui);

        ui.separator();

        self.show_registers(ui);

        ui.separator();

        self.terminal.show(ui);

        self.message_window.show_window(ctx);
    }

    fn show_modules(&self, ui: &mut egui::Ui) {
        ui.add_enabled_ui(self.emulator.is_some(), |ui| {
            let mut led = String::new();
            if let Some(p5ddr) = self.io_ports.get(&registers::PBDDR) {
                if let Some(p5dr) = self.io_ports.get(&registers::PBDR) {
                    let pattern = p5ddr & (!p5dr);
                    for i in 0..=7 {
                        if (pattern >> (7 - i)) & 1 == 0 {
                            led += "x";
                        } else {
                            led += "o";
                        }
                    }
                }
            }
            ui.strong("LED");
            ui.label(led);
        });
    }

    fn show_registers(&self, ui: &mut egui::Ui) {
        ui.push_id("show_registers", |ui| {
            egui_extras::TableBuilder::new(ui)
                .column(Column::auto())
                .column(Column::remainder())
                .header(20.0, |mut header| {
                    header.col(|ui| {
                        ui.strong("Address");
                    });
                    header.col(|ui| {
                        ui.strong("Value");
                    });
                })
                .body(|mut body| {
                    let mut io_ports: Vec<(&u32, &u8)> = self.io_ports.iter().collect();
                    io_ports.sort_by(|a, b| a.0.cmp(&b.0));

                    for (addr, value) in io_ports {
                        body.row(16.0, |mut row| {
                            row.col(|ui| {
                                ui.label(format!("{:x}", addr));
                            });
                            row.col(|ui| {
                                ui.label(format!("{:x}", value));
                            });
                        });
                    }
                });
        });
    }
}
