#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use crate::simulator::Simulator;
use eframe::egui;
use std::env::{self};
use update::Updater;

mod emulator;
mod simulator;
mod update;
mod utils;

#[tokio::main]
async fn main() -> Result<(), eframe::Error> {
    env::set_var("RUST_LOG", "info");
    env_logger::init(); // Log to stderr (if you run with `RUST_LOG=debug`).

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            // .with_inner_size([640.0, 480.0])
            .with_inner_size([960.0, 780.0])
            // .with_inner_size([1280.0, 720.0])
            .with_min_inner_size([320.0, 240.0]),
        ..Default::default()
    };
    eframe::run_native(
        "H8 Practice Kit Simulator",
        options,
        Box::new(|_cc| {
            let app = Box::<MyApp>::default();
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
    updater: Updater,
    simulator: Simulator,
}

impl Default for MyApp {
    fn default() -> Self {
        let emulator_version = emulator::check_version();
        Self {
            emulator_version: emulator_version.clone(),
            updater: Updater::new(emulator_version),
            simulator: Simulator::new(),
        }
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ctx.set_pixels_per_point(2.0);
            self.simulator.ui(ui, ctx);
            self.updater.update(ui, ctx);
        });
    }
}
