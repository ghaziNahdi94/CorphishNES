use std::io;

use crate::{apu::Apu, bus::Bus, cpu::Cpu};

pub struct Emulator {
    cpu: Cpu,
    pub bus: Bus,
    apu: Apu,
}

impl Emulator {
    pub fn new() -> Emulator {
        Emulator {
            cpu: Cpu::init(),
            bus: Bus::new(),
            apu: Apu::new()
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
        &self.bus.ppu.as_ref().unwrap().screen_pixels
    }

    pub fn run_frame(&mut self) {
        // 89342 cycles PPU par frame / 3 = 29780.666... cycles CPU
        // On exécute jusqu'à ce que le PPU ait fait un frame complet
        let mut ppu_cycles_this_frame = 0;
        const PPU_CYCLES_PER_FRAME: usize = 89342; // 262 scanlines * 341 cycles - 1 skipped cycle sur pre-render
    
        while ppu_cycles_this_frame < PPU_CYCLES_PER_FRAME {
            // Handle NMI interruption
            if self.bus.ppu.as_ref().unwrap().nmi_interrupt_pending {
                self.bus.ppu.as_mut().unwrap().nmi_interrupt_pending = false;
                self.cpu.trigger_nmi(&mut self.bus);
            }
        
            let cycles = self.cpu.execute(&mut self.bus);
            
            // Move PPU 3 steps per CPU Cycle 
            for _ in 0..(cycles * 3) {
                self.bus.ppu.as_mut().unwrap().step();
                ppu_cycles_this_frame += 1;
            }
        }
    }
}