use std::time;

use super::Simulator;

impl Simulator {
    pub fn parse_message(&mut self, message: String) {
        if message.starts_with("stdout:") {
            self.terminal.push(&message.replacen("stdout:", "", 1));
            return;
        }

        let list: Vec<&str> = message.split(':').collect();
        match list[0] {
            "ioport" => self.parse_ioport(list),
            "ready" => {
                if let Some(emulator) = &self.emulator {
                    self.send_initial_ioport(emulator);

                    emulator.send_message("cmd:start");
                    self.onesec_timing = time::Instant::now();
                }
            }
            "1sec" => {
                if list.len() != 2 {
                    return;
                }
                if let Ok(state) = list[1].parse::<usize>() {
                    self.emulator_state = state;
                } else {
                    return;
                }
                let duration = self.onesec_timing.elapsed();
                self.onesec_timing = time::Instant::now();
                self.speed = 1f64 / duration.as_secs_f64();
            }
            _ => (),
        }
    }

    fn parse_ioport(&mut self, list: Vec<&str>) {
        if list.len() != 4 {
            return;
        }

        if let Ok(port) = u8::from_str_radix(&list[1], 16) {
            if let Ok(value) = u8::from_str_radix(&list[2], 16) {
                if let Ok(state) = list[3].parse::<usize>() {
                    match port {
                        // 7SegLED | LED
                        0x4 | 0xb => self.io_port.write(port, value, state),
                        _ => (),
                    }
                }
            }
        }
    }
}
