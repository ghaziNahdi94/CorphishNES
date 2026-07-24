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
            cartridge: Cartridge { path: String::from("") }
        }
    }

    fn run_frame(&mut self) {
        self.cpu.execute(&mut self.bus);
    }
}