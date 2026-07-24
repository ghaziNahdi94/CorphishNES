pub struct Bus {
    pub ram_cpu: [u8; 2048],
}

impl Bus {
    pub fn new() -> Bus {
        Bus {
            ram_cpu: [0; 2048]
        }
    }

    pub fn read(&self, address: u16) -> u8 {
        match address {
            0x0000..=0x1FFF => self.ram_cpu[(address & 0x07FF) as usize],
            _ => 0,
        }
    }

    pub fn write(&mut self, address: u16, value: u8){
        match address {
            0x0000..=0x1FFF => self.ram_cpu[(address & 0x07FF) as usize] = value,
            _ => {}
        }
    }
}