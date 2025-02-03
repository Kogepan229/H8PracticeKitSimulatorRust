use super::Simulator;
use eframe::egui::{self, Color32, FontId, TextFormat, Vec2};
use egui_extras::Column;
use rfd::AsyncFileDialog;
use std::{
    cell::RefCell,
    sync::{Arc, Mutex},
};

pub struct SimulatorUiStates {
    pub elf_path: Arc<Mutex<String>>,
    pub elf_args: String,
    pub toggle_switches: RefCell<[bool; 5]>,
    pub push_switches: RefCell<[bool; 5]>,
    pub speed: f64,
    pub speed_buf: Vec<f64>,
}

impl SimulatorUiStates {
    pub fn new() -> Self {
        SimulatorUiStates {
            elf_path: Arc::new(Mutex::new(String::new())),
            elf_args: String::new(),
            toggle_switches: RefCell::new([false; 5]),
            push_switches: RefCell::new([false; 5]),
            speed: 0f64,
            speed_buf: Vec::new(),
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

        if self.ui_states.speed == 0f64 {
            if let Some(speed) = self.ui_states.speed_buf.first() {
                self.ui_states.speed = *speed;
            }
        }
        if self.ui_states.speed_buf.len() >= 10 {
            self.ui_states.speed = self.ui_states.speed_buf.iter().sum::<f64>()
                / self.ui_states.speed_buf.len() as f64;
            self.ui_states.speed_buf.clear();
        }
        ui.label(format!("Speed: x{:.6}", self.ui_states.speed));

        ui.separator();

        self.show_modules(ui);

        ui.separator();

        self.show_registers(ui);

        ui.separator();

        self.terminal.show(ui);

        self.message_window.show_window(ctx);
    }

    fn show_modules(&mut self, ui: &mut egui::Ui) {
        ui.columns(2, |columns| {
            self.show_led(&mut columns[0]);
            columns[0].add_space(4.0);
            self.show_digit_led(&mut columns[0]);

            self.show_toggle_switches(&mut columns[1]);
            columns[1].add_space(4.0);
            self.show_push_switches(&mut columns[1]);
        });
    }

    fn show_led(&mut self, ui: &mut egui::Ui) {
        ui.add_enabled_ui(self.emulator.is_some(), |ui| {
            ui.strong("LED");
            ui.with_layout(egui::Layout::left_to_right(egui::Align::TOP), |ui| {
                ui.style_mut().spacing.item_spacing.x = 0f32;

                let pattern = self.io_port.read(0xb).unwrap();
                for i in 0..=7 {
                    ui.add(Self::single_led((pattern >> (7 - i)) & 1 == 0));
                }
            });
        });
    }

    fn single_led(on: bool) -> impl egui::Widget {
        move |ui: &mut egui::Ui| {
            let desired_size = ui.spacing().interact_size.y * egui::vec2(1.2, 1.2);
            let (rect, response) = ui.allocate_exact_size(desired_size, egui::Sense::hover());

            if ui.is_rect_visible(rect) {
                let visuals = ui.style().noninteractive();
                let rect = rect.expand(visuals.expansion);
                let radius = 0.5 * rect.width();
                let center = egui::pos2(rect.center().x, rect.top() + radius);
                let color = if on { Color32::RED } else { visuals.bg_fill };
                ui.painter()
                    .circle(center, 0.75 * radius, color, visuals.fg_stroke);
            }

            response
        }
    }

    fn show_digit_led(&mut self, ui: &mut egui::Ui) {
        use egui::text::LayoutJob;

        ui.add_enabled_ui(self.emulator.is_some(), |ui| {
            ui.strong("7Seg LED");

            let red_text = TextFormat {
                color: Color32::RED,
                font_id: FontId::monospace(24.0),
                ..Default::default()
            };
            let transparent_text = TextFormat {
                color: Color32::TRANSPARENT,
                font_id: FontId::monospace(24.0),
                ..Default::default()
            };

            self.io_port.filter_port4(
                self.get_corrected_current_emulator_state()
                    .saturating_sub(200_000 * 3),
            );
            let port4 = self.io_port.read_port4();

            let mut diaplay_num: [Option<u8>; 4] = [None; 4];
            for item in port4 {
                for i in 0..4 {
                    if (item.0 >> i) & 1 == 1 {
                        diaplay_num[i] = Some(item.0 >> 4);
                    }
                }
            }

            let mut job = LayoutJob::default();
            for (i, num) in diaplay_num.iter().enumerate() {
                let leading_space = if i == 0 { 0.0 } else { 4.0 };
                if let Some(num) = num {
                    job.append(num.to_string().as_str(), leading_space, red_text.clone());
                } else {
                    job.append("0", leading_space, transparent_text.clone());
                }
            }

            ui.label(job);
        });
    }

    fn show_toggle_switches(&mut self, ui: &mut egui::Ui) {
        ui.strong("Toggle Swtich");

        let prev_toggle_switches = self.ui_states.toggle_switches.borrow_mut().clone();
        ui.with_layout(egui::Layout::left_to_right(egui::Align::TOP), |ui| {
            for switch in self.ui_states.toggle_switches.borrow_mut().iter_mut() {
                ui.add(Self::toggle_switch(switch));
            }
        });
        let mut is_changed = false;
        for (i, switch) in self.ui_states.toggle_switches.borrow().iter().enumerate() {
            if prev_toggle_switches[i] != *switch {
                is_changed = true;
                break;
            }
        }
        if is_changed {
            let switches = self.ui_states.toggle_switches.borrow();
            if switches[0] {
                self.io_port
                    .write(5, self.io_port.read(5).unwrap() | (1 << 2), 0); // TODO
            } else {
                self.io_port
                    .write(5, self.io_port.read(5).unwrap() & !(1 << 2), 0); // TODO
            }
            if switches[1] {
                self.io_port
                    .write(5, self.io_port.read(5).unwrap() | (1 << 3), 0); // TODO
            } else {
                self.io_port
                    .write(5, self.io_port.read(5).unwrap() & !(1 << 3), 0); // TODO
            }
            if let Some(emulator) = self.emulator.as_mut() {
                emulator.send_message(format!(
                    "ioport:{:x}:{:x}",
                    0x5,
                    self.io_port.read(5).unwrap()
                ));
            }
        }
    }

    fn show_push_switches(&mut self, ui: &mut egui::Ui) {
        ui.strong("Push Swtich");

        let prev_push_switches = self.ui_states.push_switches.borrow_mut().clone();
        ui.with_layout(egui::Layout::left_to_right(egui::Align::TOP), |ui| {
            for switch in self.ui_states.push_switches.borrow_mut().iter_mut() {
                ui.add(Self::push_switch(switch));
            }
        });
        let mut is_changed = false;
        for (i, switch) in self.ui_states.push_switches.borrow().iter().enumerate() {
            if prev_push_switches[i] != *switch {
                is_changed = true;
                break;
            }
        }
        if is_changed {
            let switches = self.ui_states.push_switches.borrow();
            if switches[0] {
                self.io_port
                    .write(5, self.io_port.read(5).unwrap() & !(1 << 0), 0); // TODO
            } else {
                self.io_port
                    .write(5, self.io_port.read(5).unwrap() | (1 << 0), 0); // TODO
            }
            if switches[1] {
                self.io_port
                    .write(5, self.io_port.read(5).unwrap() & !(1 << 1), 0); // TODO
            } else {
                self.io_port
                    .write(5, self.io_port.read(5).unwrap() | (1 << 1), 0); // TODO
            }
            if let Some(emulator) = self.emulator.as_mut() {
                emulator.send_message(format!(
                    "ioport:{:x}:{:x}",
                    0x5,
                    self.io_port.read(5).unwrap()
                ));
            }
        }
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
                    body.row(16.0, |mut row| {
                        row.col(|ui| {
                            ui.label("Port5");
                        });
                        row.col(|ui| {
                            ui.label(format!("{:x}", self.io_port.read(5).unwrap()));
                        });
                    });
                    body.row(16.0, |mut row| {
                        row.col(|ui| {
                            ui.label("PortB");
                        });
                        row.col(|ui| {
                            ui.label(format!("{:x}", self.io_port.read(0xb).unwrap()));
                        });
                    });
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
                let how_on = ui.ctx().animate_bool_responsive(response.id, !*on);
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

    fn push_switch(on: &mut bool) -> impl egui::Widget + '_ {
        move |ui: &mut egui::Ui| {
            let desired_size = ui.spacing().interact_size.y * egui::vec2(1.0, 1.0);
            let (rect, mut response) = ui.allocate_exact_size(desired_size, egui::Sense::click());
            if response.clicked() {
                *on = !*on;
                response.mark_changed();
            }
            response.widget_info(|| {
                egui::WidgetInfo::selected(egui::WidgetType::Checkbox, ui.is_enabled(), *on, "")
            });

            if ui.is_rect_visible(rect) {
                let visuals = ui.style().interact_selectable(&response, false);
                let rect = rect.expand(visuals.expansion);
                let radius = 0.5 * rect.width();
                ui.painter()
                    .rect(rect, radius, visuals.bg_fill, visuals.bg_stroke);
                let center = egui::pos2(rect.center().x, rect.top() + radius);
                let mut stroke = visuals.fg_stroke;
                stroke.width = 1.0;
                let color = if *on {
                    Color32::from_gray(160)
                } else {
                    visuals.bg_fill
                };
                ui.painter().circle(center, 0.75 * radius, color, stroke);
            }

            response
        }
    }
}
