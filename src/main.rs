#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use crate::simulator::Simulator;
use eframe::egui;
use std::env::{self};
use update::UpdateStatusType;

mod emulator;
mod simulator;
mod update;
mod utils;

#[tokio::main]
async fn main() -> Result<(), eframe::Error> {
    env::set_var("RUST_LOG", "info");
    env_logger::init(); // Log to stderr (if you run with `RUST_LOG=debug`).

    tokio::spawn(async {
        update::get_latest_info()
            .await
            .inspect_err(|e| eprintln!("{}", e))
            .unwrap();
    });

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([640.0, 480.0])
            .with_min_inner_size([320.0, 240.0]),
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

            Ok(app)
        }),
    )
}

struct MyApp {
    emulator_version: Option<String>,
    update_status: UpdateStatusType,
    simulator: Simulator,
}

impl Default for MyApp {
    fn default() -> Self {
        Self {
            emulator_version: None,
            update_status: UpdateStatusType::UNCHECKED,
            simulator: Simulator::new(),
        }
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            self.simulator.ui(ui, ctx);
        });
    }
}
