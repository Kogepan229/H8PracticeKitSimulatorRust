use crate::emulator::{self, Emulator};
use eframe::egui;
use message_window::MessageWindow;
use std::{collections::HashMap, time};
use terminal::Terminal;
use tokio::sync::mpsc::{self, Receiver};
use views::SimulatorUiStates;

mod message_window;
mod parse_messages;
mod registers;
mod terminal;
mod views;

pub struct Simulator {
    emulator: Option<Emulator>,
    emulator_exec_rx: Option<Receiver<Result<Emulator, String>>>,
    ui_states: SimulatorUiStates,
    message_window: MessageWindow,
    terminal: Terminal,
    io_ports: HashMap<u32, u8>,
    prev_timing: time::Instant,
}

impl Simulator {
    pub fn new() -> Self {
        Simulator {
            emulator: None,
            emulator_exec_rx: None,
            ui_states: SimulatorUiStates::new(),
            message_window: MessageWindow::new(),
            terminal: Terminal::new(),
            io_ports: HashMap::new(),
            prev_timing: time::Instant::now(),
        }
    }

    fn update(&mut self) {
        if let Some(rx) = self.emulator_exec_rx.as_mut() {
            if let Ok(result) = rx.try_recv() {
                match result {
                    Ok(emulator) => self.emulator = Some(emulator),
                    Err(e) => println!("{}", e),
                }
                self.emulator_exec_rx = None
            }
        }

        if let Some(emulator) = &self.emulator {
            if emulator.socket_receiver_handle.is_finished() {
                self.pop_emulator_messages();
                self.emulator = None;
            }
        }

        self.pop_emulator_messages();
    }

    fn execute_emulator(&mut self, ctx: &egui::Context) {
        self.init_io_ports();
        self.message_window.clear_messages();
        self.terminal.clear();

        let (tx, rx) = mpsc::channel(1);
        self.emulator_exec_rx = Some(rx);
        let _elf_path = self.ui_states.elf_path.lock().unwrap().clone();
        let _elf_args = self.ui_states.elf_args.clone();
        let _ctx = ctx.clone();
        tokio::spawn(async move {
            let emu = emulator::Emulator::execute(_elf_path, _elf_args, _ctx).await;
            if let Err(e) = tx.send(emu).await {
                eprintln!("{}", e)
            }
        });
    }

    fn stop_emulator(&self) {
        if let Some(emulator) = &self.emulator {
            emulator.send_message("cmd:stop");
        };
    }

    fn pop_emulator_messages(&mut self) {
        if let Some(emulator) = self.emulator.as_mut() {
            let messages = emulator.pop_messages();
            self.message_window.push_messages(&messages);

            for message in messages {
                self.parse_message(message);
            }
        };
    }

    fn init_io_ports(&mut self) {
        self.io_ports.clear();

        // Switch
        self.io_ports.insert(registers::P5DDR, 0); // P5DDR
        self.io_ports.insert(registers::P5DR, 0xff); // P5DR

        // LED
        self.io_ports.insert(registers::PBDDR, 0); // PBDDR
        self.io_ports.insert(registers::PBDR, 0); // PBDR
    }
}
