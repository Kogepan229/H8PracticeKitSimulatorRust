#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use std::{env, path::PathBuf};

use eframe::egui;
use rfd::AsyncFileDialog;
use std::sync::mpsc::{Receiver, Sender};

#[tokio::main]
async fn main() -> Result<(), eframe::Error> {
    env::set_var("RUST_LOG", "info");
    env_logger::init(); // Log to stderr (if you run with `RUST_LOG=debug`).

    let output = std::process::Command::new("./emulator/koge29_h8-3069f_emulator")
        .arg("--version")
        .output();
    if let Ok(o) = output {
    } else {
        log::info!("Not found Emulator.");
    }

    let output = std::process::Command::new("ls .").arg("--version").output();
    if let Ok(o) = output {
        println!("{}", String::from_utf8_lossy(&o.stdout));
    } else {
        log::info!("ls failed");
        log::error!("{}", output.err().unwrap().kind().to_string())
    }

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([320.0, 240.0]),
        ..Default::default()
    };
    eframe::run_native(
        "H8 Practice Kit Simulator",
        options,
        Box::new(|_cc| Box::<MyApp>::default()),
    )
}

struct Emulator {
    version: String,
}

struct MyApp {
    elf_path: String,
    elf_path_tx: Sender<PathBuf>,
    elf_path_rx: Receiver<PathBuf>,
}

impl Default for MyApp {
    fn default() -> Self {
        let (elf_path_tx, elf_path_rx) = std::sync::mpsc::channel();

        Self {
            elf_path: String::new(),
            elf_path_tx,
            elf_path_rx,
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

            ui.text_edit_singleline(&mut self.elf_path)
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

fn exec_emulator(elf_path: PathBuf) {
    // tokio::process::Command::new(program)
}
