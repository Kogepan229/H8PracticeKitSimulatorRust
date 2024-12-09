use std::collections::HashMap;
use std::sync::mpsc::Receiver;

use eframe::egui;
use views::SimulatorUiStates;

use crate::emulator::{self, Emulator};

mod parse_messages;
mod registers;
mod views;

pub struct Simulator {
    emulator: Option<Emulator>,
    emulator_exec_rx: Option<Receiver<Result<Emulator, String>>>,
    ui_states: SimulatorUiStates,
    io_ports: HashMap<u32, u8>,
}

impl Simulator {
    pub fn new() -> Self {
        Simulator {
            emulator: None,
            emulator_exec_rx: None,
            ui_states: SimulatorUiStates::new(),
            io_ports: HashMap::new(),
        }
    }

    fn update(&mut self) {
        if let Some(rx) = &self.emulator_exec_rx {
            if let Ok(result) = rx.recv() {
                match result {
                    Ok(emulator) => self.emulator = Some(emulator),
                    Err(e) => println!("{}", e),
                }
            }
            self.emulator_exec_rx = None
        }

        if self.emulator.is_some() {
            if let Ok(r) = self.emulator.as_mut().unwrap().process.try_wait() {
                if let Some(status) = r {
                    println!("{}", status.to_string());
                    self.emulator = None;
                }
            }
        }

        if let Some(emulator) = self.emulator.as_mut() {
            let messages = emulator.pop_messages();
            self.ui_states.push_messages(&messages);

            for message in messages {
                self.parse_message(message);
            }
        };
    }

    fn execute_emulator(&mut self, ctx: &egui::Context) {
        self.init_io_ports();
        self.ui_states.clear_messages();

        let (tx, rx) = std::sync::mpsc::channel();
        self.emulator_exec_rx = Some(rx);
        let _elf_path = self.ui_states.elf_path.lock().unwrap().clone();
        let _elf_args = self.ui_states.elf_args.clone();
        let _ctx = ctx.clone();
        tokio::spawn(async move {
            let emu = emulator::Emulator::execute(_elf_path, _elf_args, _ctx).await;
            tx.send(emu).unwrap();
        });
    }

    fn stop_emulator(&self) {
        if let Some(emulator) = &self.emulator {
            emulator.send_message("cmd:stop");
        };
    }

    fn init_io_ports(&mut self) {
        self.io_ports.clear();

        // Switch
        self.io_ports.insert(registers::P5DDR, 0); // P5DDR
        self.io_ports.insert(registers::P5DR, 0); // P5DR

        // LED
        self.io_ports.insert(registers::PBDDR, 0); // PBDDR
        self.io_ports.insert(registers::PBDR, 0); // PBDR
    }
}
