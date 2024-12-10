use std::collections::HashMap;
use std::sync::mpsc::Receiver;

use views::SimulatorUiStates;

use crate::emulator::Emulator;

mod views;

pub struct Simulator {
    emulator: Option<Emulator>,
    emulator_exec_rx: Option<Receiver<Result<Emulator, String>>>,
    ui_states: SimulatorUiStates,
    io_ports: HashMap<String, u8>,
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

        if let Some(emulator) = &self.emulator {
            self.ui_states.push_messages(emulator.pop_messages());
        };
    }
}
