use std::time;

use super::Simulator;

impl Simulator {
    pub fn parse_message(&mut self, message: String) {
        if message.starts_with("stdout:") {
            self.ui_states.stdout += &message.replacen("stdout:", "", 1);
            return;
        }

        let list: Vec<&str> = message.split(':').collect();
        match list[0] {
            "u8" => self.parse_u8(list),
            "ready" => {
                if let Some(emulator) = &self.emulator {
                    for (addr, value) in self.io_ports.iter() {
                        emulator.send_message(format!("u8:{:x}:{:x}", addr, value));
                    }
                    emulator.send_message("cmd:start");
                    self.prev_timing = time::Instant::now();
                }
            }
            "1sec" => {
                let duration = self.prev_timing.elapsed();
                self.prev_timing = time::Instant::now();
                self.ui_states.speed = 1f32 / duration.as_secs_f32();
            }
            _ => (),
        }
    }

    fn parse_u8(&mut self, list: Vec<&str>) {
        if list.len() != 3 {
            return;
        }
        let addr_result = u32::from_str_radix(&list[1], 16);
        let value_result = u8::from_str_radix(&list[2], 16);
        if let Ok(addr) = addr_result {
            if let Ok(value) = value_result {
                self.io_ports.insert(addr, value);
            }
        }
    }
}
