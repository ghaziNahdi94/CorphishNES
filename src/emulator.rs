use std::io;

use crate::{bus::Bus, cpu::Cpu};

pub struct Emulator {
    cpu: Cpu,
    bus: Bus,
}

impl Emulator {
    fn new() -> Emulator {
        Emulator {
            cpu: Cpu::init(),
            bus: Bus::new(),
        }
    }

    pub fn load_rom(&mut self, path: &str) -> Result<(), io::Error> {
        self.bus.load_cartridge(path)?;
        
        if let Some(cartridge) = &self.bus.cartridge {
            self.bus.init_ppu(&cartridge.chr_rom.clone(), cartridge.mirroring_type);
        };

        self.cpu.reset(&mut self.bus);
        Ok(())
    }

    fn run_frame(&mut self) {
        self.cpu.execute(&mut self.bus);
    }
}