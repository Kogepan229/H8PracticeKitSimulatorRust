use super::Simulator;
use eframe::egui::{self, Vec2};
use egui_extras::Column;
use rfd::AsyncFileDialog;
use std::{
    cell::RefCell,
    sync::{Arc, Mutex},
};

pub struct SimulatorUiStates {
    pub elf_path: Arc<Mutex<String>>,
    pub elf_args: String,
    pub speed: f32,
    pub switch: RefCell<bool>,
}

impl SimulatorUiStates {
    pub fn new() -> Self {
        SimulatorUiStates {
            elf_path: Arc::new(Mutex::new(String::new())),
            elf_args: String::new(),
            speed: 0f32,
            switch: RefCell::new(false),
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
            let pattern = self.read_io_port(0xb);
            for i in 0..=7 {
                if (pattern >> (7 - i)) & 1 == 0 {
                    led += "o";
                } else {
                    led += "x";
                }
            }
            ui.strong("LED");
            ui.label(led);
        });
        ui.strong("Toggle Swtich");

        egui::widgets::global_theme_preference_buttons(ui);
        ui.with_layout(egui::Layout::left_to_right(egui::Align::TOP), |ui| {
            ui.add(Self::toggle_switch(&mut self.ui_states.switch.borrow_mut()));
            ui.add(Self::toggle_switch(&mut self.ui_states.switch.borrow_mut()));
            ui.add(Self::toggle_switch(&mut self.ui_states.switch.borrow_mut()));
            ui.add(Self::toggle_switch(&mut self.ui_states.switch.borrow_mut()));
            ui.add(Self::toggle_switch(&mut self.ui_states.switch.borrow_mut()));
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
                    for (i, value) in self.io_port.iter().enumerate() {
                        if i + 1 != 0x5 && i + 1 != 0xb {
                            continue;
                        }
                        body.row(16.0, |mut row| {
                            row.col(|ui| {
                                ui.label(format!("Port{:X}", i + 1));
                            });
                            row.col(|ui| {
                                ui.label(format!("{:x}", value));
                            });
                        });
                    }
                });
        });
    }

    fn toggle_switch(on: &mut bool) -> impl egui::Widget + '_ {
        move |ui: &mut egui::Ui| {
            let desired_size = ui.spacing().interact_size.y * egui::vec2(1.0, 2.0);
            let (rect, mut response) = ui.allocate_exact_size(desired_size, egui::Sense::click());
            if response.clicked() {
                *on = !*on;
                response.mark_changed();
            }
            response.widget_info(|| {
                egui::WidgetInfo::selected(egui::WidgetType::Checkbox, ui.is_enabled(), *on, "")
            });

            if ui.is_rect_visible(rect) {
                let how_on = ui.ctx().animate_bool_responsive(response.id, *on);
                let visuals = ui.style().interact_selectable(&response, false);
                let rect = rect.expand(visuals.expansion);
                let radius = 0.5 * rect.width();
                ui.painter()
                    .rect(rect, radius, visuals.bg_fill, visuals.bg_stroke);
                let circle_y = egui::lerp((rect.top() + radius)..=(rect.bottom() - radius), how_on);
                let center = egui::pos2(rect.center().x, circle_y);
                let mut stroke = visuals.fg_stroke;
                stroke.width = 1.0;
                ui.painter()
                    .circle(center, 0.75 * radius, visuals.bg_fill, stroke);
            }

            response
        }
    }
}
