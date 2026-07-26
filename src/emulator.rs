use std::io;

use crate::{bus::Bus, cpu::Cpu};

pub struct Emulator {
    cpu: Cpu,
    bus: Bus,
}

impl Emulator {
    pub fn new() -> Emulator {
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

    pub fn get_framebuffer(&self) -> &[u8; 256 * 240 * 3] {
        &self.bus.ppu.as_ref().unwrap().framebuffer
    }

    pub fn run_frame(&mut self) {
        // Le PPU 3 cycle => CPU 1 Cycle
        // 1 Frame = 89342 cycles PPU ≈ 29780 cycles CPU
        let cycles_per_frame = 29780;
        let mut cpu_cycles = 0;

        while cpu_cycles < cycles_per_frame {
            // Handle NMI interruption
            if self.bus.ppu.as_ref().unwrap().nmi_pending {
                self.bus.ppu.as_mut().unwrap().nmi_pending = false;
                self.cpu.trigger_nmi(&mut self.bus);
            }

            let cycles = self.cpu.execute(&mut self.bus);
            cpu_cycles += cycles as usize;

            // Move PPU 3 steps per Cycle CPU 
            for _ in 0..(cycles * 3) {
                self.bus.ppu.as_mut().unwrap().step();
            }
        }
}
}