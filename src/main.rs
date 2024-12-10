#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use crate::simulator::Simulator;
use eframe::egui;
use std::env::{self};

mod emulator;
mod simulator;

#[tokio::main]
async fn main() -> Result<(), eframe::Error> {
    env::set_var("RUST_LOG", "info");
    env_logger::init(); // Log to stderr (if you run with `RUST_LOG=debug`).

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
    simulator: Simulator,
}

impl Default for MyApp {
    fn default() -> Self {
        Self {
            emulator_version: None,
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
