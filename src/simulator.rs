use crate::emulator::{self, Emulator};
use eframe::egui;
use message_window::MessageWindow;
use std::time;
use terminal::Terminal;
use tokio::sync::mpsc::{self, Receiver};
use views::SimulatorUiStates;

mod message_window;
mod parse_messages;
mod terminal;
mod views;

pub const IO_PORT_SIZE: usize = 11;

pub struct Simulator {
    emulator: Option<Emulator>,
    emulator_exec_rx: Option<Receiver<Result<Emulator, String>>>,
    ui_states: SimulatorUiStates,
    message_window: MessageWindow,
    terminal: Terminal,
    io_port: [u8; IO_PORT_SIZE],
    prev_timing: time::Instant,
}

impl Simulator {
    pub fn new() -> Self {
        let mut simulator = Simulator {
            emulator: None,
            emulator_exec_rx: None,
            ui_states: SimulatorUiStates::new(),
            message_window: MessageWindow::new(),
            terminal: Terminal::new(),
            io_port: [0; IO_PORT_SIZE],
            prev_timing: time::Instant::now(),
        };
        simulator.init_io_port();
        simulator.write_io_port(0x5, 0x3);
        simulator
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

        if let Some(emulator) = self.emulator.as_mut() {
            if emulator.socket_receiver_handle.is_finished() && emulator.process.try_wait().is_ok()
            {
                self.pop_emulator_messages();
                self.emulator = None;
            }
        }

        self.pop_emulator_messages();
    }

    fn execute_emulator(&mut self, ctx: &egui::Context) {
        self.init_io_port();
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

    fn read_io_port(&self, port: u8) -> u8 {
        self.io_port[port as usize - 1]
    }

    fn write_io_port(&mut self, port: u8, value: u8) {
        self.io_port[port as usize - 1] = value;
    }

    fn init_io_port(&mut self) {
        // LED
        self.write_io_port(0xb, 0xff);
    }
}
