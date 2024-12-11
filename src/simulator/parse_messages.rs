use super::Simulator;

impl Simulator {
    pub fn parse_message(&mut self, message: String) {
        let list: Vec<&str> = message.split(':').collect();
        match list[0] {
            "u8" => self.parse_u8(list),
            _ => (),
        }
    }

    fn parse_u8(&mut self, list: Vec<&str>) {
        if list.len() != 3 {
            return;
        }
        let addr_result = u32::from_str_radix(&list[1][2..], 16);
        let value_result = u8::from_str_radix(&list[2][2..], 16);
        if let Ok(addr) = addr_result {
            if let Ok(value) = value_result {
                self.io_ports.insert(addr, value);
            }
        }
    }
}
