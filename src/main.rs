#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use std::{
    env::{self, current_dir},
    path::PathBuf,
};

use eframe::egui;
use rfd::AsyncFileDialog;
use std::sync::mpsc::{Receiver, Sender};

mod emulator;

#[tokio::main]
async fn main() -> Result<(), eframe::Error> {
    env::set_var("RUST_LOG", "info");
    env_logger::init(); // Log to stderr (if you run with `RUST_LOG=debug`).

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
    emu_exec_tx: Sender<emulator::Emulator>,
    emu_exec_rx: Receiver<emulator::Emulator>,
    emu: Option<emulator::Emulator>,
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
        }
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
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
                tokio::spawn(async move {
                    let emu = emulator::Emulator::execute().await;
                    _emu_exec_tx.send(emu).unwrap();
                });
            }

            if let Ok(emu) = self.emu_exec_rx.try_recv() {
                self.emu = Some(emu);
            }

            if let Some(emu) = &self.emu {
                if let Ok(emu_socket_received) = emu.socket_received.try_lock() {
                    emu_socket_received.iter();
                }
            }
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
