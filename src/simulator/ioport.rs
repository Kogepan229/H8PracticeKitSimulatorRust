pub struct IoPort {
    port4: Vec<(u8, usize)>,
    port5: u8,
    portb: u8,
}

impl IoPort {
    pub fn new() -> Self {
        Self {
            port4: Vec::new(),
            port5: 0,
            portb: 0,
        }
    }

    pub fn read(&self, port: u8) -> Option<u8> {
        return match port {
            5 => Some(self.port5),
            0xb => Some(self.portb),
            _ => None,
        };
    }

    pub fn write(&mut self, port: u8, value: u8, emulator_state: usize) {
        return match port {
            4 => self.port4.push((value, emulator_state)),
            5 => self.port5 = value,
            0xb => self.portb = value,
            _ => (),
        };
    }

    pub fn filter_port4(&mut self, threshold_state: usize) {
        let latest = match self.port4.last() {
            Some(latest) => *latest,
            None => return,
        };

        self.port4 = self
            .port4
            .clone()
            .into_iter()
            .filter(|port| port.1 >= threshold_state)
            .collect();

        if self.port4.is_empty() {
            self.port4.push(latest);
        }
    }

    pub fn read_port4(&self) -> &Vec<(u8, usize)> {
        return &self.port4;
    }

    pub fn init_led(&mut self) {
        // LED
        self.write(0xb, 0xff, 0);

        // 7Seg LED
        self.port4.clear();
    }

    pub fn init_switches(&mut self) {
        self.write(0x5, 0x3, 0);
    }
}
