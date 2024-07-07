#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use std::{
    env::{self, current_dir},
    path::PathBuf,
    time::Duration,
};

use eframe::egui::{self, ScrollArea, TextStyle};
use rfd::AsyncFileDialog;
use std::sync::mpsc::{Receiver, Sender};
use tokio::runtime::Runtime;

mod emulator;

#[tokio::main]
async fn main() -> Result<(), eframe::Error> {
    env::set_var("RUST_LOG", "info");
    env_logger::init(); // Log to stderr (if you run with `RUST_LOG=debug`).

    let rt = Runtime::new().expect("Unable to create Runtime");

    // Enter the runtime so that `tokio::spawn` is available immediately.
    let _enter = rt.enter();

    // Execute the runtime in its own thread.
    // The future doesn't have to do anything. In this example, it just sleeps forever.
    std::thread::spawn(move || {
        rt.block_on(async {
            loop {
                tokio::time::sleep(Duration::from_secs(3600)).await;
            }
        })
    });

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([320.0, 240.0]),
        ..Default::default()
    };
    eframe::run_native(
        "H8 Practice Kit Simulator",
        options,
        Box::new(|_cc| {
            let mut app = Box::<MyApp>::default();
            app.emulator_version = emulator::check_version();
            if let Some(emulator_version) = &app.emulator_version {
                log::info!("Emulator version: {}", emulator_version);
            } else {
                log::info!("Emulator is not found.");
            }

            app
        }),
    )
}

struct MyApp {
    emulator_version: Option<String>,
    elf_path: String,
    elf_path_tx: Sender<PathBuf>,
    elf_path_rx: Receiver<PathBuf>,
    emu_exec_tx: Sender<Result<emulator::Emulator, String>>,
    emu_exec_rx: Receiver<Result<emulator::Emulator, String>>,
    emu: Option<emulator::Emulator>,
    emu_messages: Vec<String>,
}

impl Default for MyApp {
    fn default() -> Self {
        let (elf_path_tx, elf_path_rx) = std::sync::mpsc::channel();
        let (emu_exec_tx, emu_exec_rx) = std::sync::mpsc::channel();

        Self {
            emulator_version: None,
            elf_path: String::new(),
            elf_path_tx,
            elf_path_rx,
            emu_exec_tx,
            emu_exec_rx,
            emu: None,
            emu_messages: Vec::new(),
        }
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            // println!("main s");
            ui.label("Hello");
            if ui.button("Select elf").clicked() {
                select_elf(self.elf_path_tx.clone());
            }

            if let Ok(elf_path) = self.elf_path_rx.try_recv() {
                println!("{}", elf_path.to_str().unwrap().to_string());
                self.elf_path = elf_path.to_str().unwrap().to_string();
            }

            ui.text_edit_singleline(&mut self.elf_path);

            if ui.button("execute").clicked() {
                let _emu_exec_tx = self.emu_exec_tx.clone();

                let _elf_path = self.elf_path.clone();
                tokio::spawn(async move {
                    let emu = emulator::Emulator::execute(_elf_path).await;
                    _emu_exec_tx.send(emu).unwrap();
                });
            }

            if let Ok(emu_r) = self.emu_exec_rx.try_recv() {
                if let Ok(emu) = emu_r {
                    self.emu = Some(emu);
                } else {
                    println!("{}", emu_r.err().unwrap().to_string());
                }
            }

            if self.emu.is_some() {
                if let Ok(r) = self.emu.as_mut().unwrap().process.try_wait() {
                    if let Some(status) = r {
                        println!("{}", status.to_string());
                        self.emu = None;
                    }
                }
            }

            if let Some(emu) = &self.emu {
                if let Ok(mut emu_socket_received) = emu.socket_received.try_lock() {
                    self.emu_messages.append(&mut emu_socket_received);
                }
            }

            if self.emu.is_some() {
                ui.label("Emulator is available.");
            } else {
                ui.label("Emulator is None.");
            }

            let text_style = TextStyle::Body;
            let row_height = ui.text_style_height(&text_style);
            ScrollArea::vertical().auto_shrink(false).show_rows(
                ui,
                row_height,
                self.emu_messages.len(),
                |ui, row_range| {
                    for row in row_range {
                        let text = &self.emu_messages[row];
                        ui.label(text);
                    }
                },
            );
            // println!("main e");
        });
    }
}

fn select_elf(tx: Sender<PathBuf>) {
    tokio::spawn(async move {
        let file = AsyncFileDialog::new()
            .add_filter("elf", &["elf"])
            .pick_file()
            .await;
        if let Some(fi) = file {
            tx.send(fi.path().to_path_buf()).unwrap();
        }
    });
}
