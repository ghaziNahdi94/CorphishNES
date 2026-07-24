use std::io;
use crate::cartridge::Cartridge;

pub struct Bus {
    pub ram_cpu: [u8; 2048],
    pub cartridge: Option<Cartridge>
}

impl Bus {
    pub fn new() -> Bus {
        Bus {
            ram_cpu: [0; 2048],
            cartridge: None
        }
    }

    fn nobody(&self) -> Option<&'static str> { None }

    pub fn read(&self, address: u16) -> u8 {
        match address {
            0x0000..=0x1FFF => self.ram_cpu[(address & 0x07FF) as usize],
            0x2000..=0x3FFF => 0, // PPU TODO
            0x4000..=0x401F => 0, // APU TODO
            0x8000..=0xFFFF => {
                match &self.cartridge {
                    Some(cartridge) => {
                        let rom_index = (address - 0x8000) as usize;
                        cartridge.prg_rom[rom_index % cartridge.prg_rom.len()]
                    },
                    None => 0
                }
            }
            _ => 0,
        }
    }

    pub fn write(&mut self, address: u16, value: u8) {
        match address {
            0x0000..=0x1FFF => self.ram_cpu[(address & 0x07FF) as usize] = value,
            0x2000..=0x3FFF => todo!(),
            0x4000..=0x401F => todo!(),
            0x8000..=0xFFFF => {}
            _ => {}
        }
    }

    pub fn load_cartridge(&mut self, file_path: &str) -> Result<(), io::Error> {
        let cartridge = Cartridge::load(file_path)?;
        println!("Cartridge loaded, Mapper {}", cartridge.mapper);
        self.cartridge = Some(cartridge);

        Ok(())
    }
}