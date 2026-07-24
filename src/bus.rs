pub struct Bus {
    pub ram_cpu: [u8; 2048],
    pub rom: Vec<u8>,
}

impl Bus {
    pub fn new() -> Bus {
        Bus {
            ram_cpu: [0; 2048],
            rom: Vec::new()
        }
    }

    pub fn read(&self, address: u16) -> u8 {
        match address {
            0x0000..=0x1FFF => self.ram_cpu[(address & 0x07FF) as usize],
            0x8000..=0xFFFF => {
                let rom_index = (address - 0x8000) as usize;
                if rom_index < self.rom.len() {
                    self.rom[rom_index]
                } else {
                    0x00
                }
            }
            _ => 0,
        }
    }

    pub fn write(&mut self, address: u16, value: u8) {
        match address {
            0x0000..=0x1FFF => self.ram_cpu[(address & 0x07FF) as usize] = value,
            _ => {}
        }
    }
}