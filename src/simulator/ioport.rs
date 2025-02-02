pub struct IoPort {
    port4: u8,
    port5: u8,
    portb: u8,
}

impl IoPort {
    pub fn new() -> Self {
        Self {
            port4: 0,
            port5: 0,
            portb: 0,
        }
    }

    pub fn read(&self, port: u8) -> Option<u8> {
        return match port {
            4 => Some(self.port4),
            5 => Some(self.port5),
            0xb => Some(self.portb),
            _ => None,
        };
    }

    pub fn write(&mut self, port: u8, value: u8, emulator_state: usize) {
        return match port {
            4 => self.port4 = value,
            5 => self.port5 = value,
            0xb => self.portb = value,
            _ => (),
        };
    }

    pub fn init_led(&mut self) {
        // LED
        self.write(0xb, 0xff, 0);

        // 7Seg LED
        self.write(4, 0, 0);
    }

    pub fn init_switches(&mut self) {
        self.write(0x5, 0x3, 0);
    }
}
