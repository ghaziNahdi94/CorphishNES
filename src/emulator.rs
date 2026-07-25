use std::io;

use crate::{bus::Bus, cartridge::Cartridge, cpu::Cpu, ppu::Ppu};

pub struct Emulator {
    cpu: Cpu,
    ppu: Ppu,
    bus: Bus,
    cartridge: Cartridge
}

impl Emulator {
    fn new() -> Emulator {
        Emulator {
            cpu: Cpu::init(),
            ppu: Ppu::init(),
            bus: Bus::new(),
            cartridge: Cartridge::load("").unwrap()
        }
    }

    pub fn load_rom(&mut self, path: &str) -> Result<(), io::Error> {
        self.bus.load_cartridge(path)?;
        self.cpu.reset(&self.bus);
        Ok(())
    }

    fn run_frame(&mut self) {
        self.cpu.execute(&mut self.bus);
    }
}