use crate::{cartridge::Mirroring, mapper::Mapper};

pub struct MapperMMC3 {
    bank_select: u8,
    r: [u8; 8],
    
    mirroring: Mirroring,
    
    prg_ram_enabled: bool,
    prg_ram_write_protect: bool,
    
    irq_latch: u8,
    irq_counter: u8,
    irq_reload: bool,
    irq_enabled: bool,
    irq_pending: bool,
    last_a12: bool,
}

impl MapperMMC3 {
    pub fn new() -> Self {
        MapperMMC3 {
            bank_select: 0,
            r: [0; 8],
            mirroring: Mirroring::Vertical,
            prg_ram_enabled: false,
            prg_ram_write_protect: false,
            irq_latch: 0,
            irq_counter: 0,
            irq_reload: false,
            irq_enabled: false,
            irq_pending: false,
            last_a12: false,
        }
    }
    
    fn prg_inversion(&self) -> bool {
        self.bank_select & 0x40 != 0
    }
    
    fn chr_inversion(&self) -> bool {
        self.bank_select & 0x80 != 0
    }
    
    fn selected_register(&self) -> usize {
        (self.bank_select & 0x07) as usize
    }
    
    pub fn signal_scanline(&mut self) {
        if self.irq_counter == 0 || self.irq_reload {
            self.irq_counter = self.irq_latch;
            self.irq_reload = false;
        } else {
            self.irq_counter -= 1;
            if self.irq_counter == 0 && self.irq_enabled {
                self.irq_pending = true;
            }
        }
    }
    
    pub fn irq_pending(&self) -> bool {
        self.irq_pending
    }
    
    pub fn clear_irq(&mut self) {
        self.irq_pending = false;
    }
}

impl Mapper for MapperMMC3 {
    fn convert_cpu_address(&self, prg_rom: &Vec<u8>, address: usize) -> usize {
        let num_8k_banks = (prg_rom.len() / 8192) as u8;
        let last_bank = num_8k_banks.saturating_sub(1);
        let second_last = last_bank.saturating_sub(1);
        
        let (bank, offset) = if !self.prg_inversion() {
            // Normal mode
            match address {
                0x0000..=0x1FFF => (self.r[6], address & 0x1FFF),
                0x2000..=0x3FFF => (self.r[7], address & 0x1FFF),
                0x4000..=0x5FFF => (second_last, address & 0x1FFF),
                0x6000..=0x7FFF => (last_bank, address & 0x1FFF),
                _ => (0, 0),
            }
        } else {
            // Reverse mode
            match address {
                0x0000..=0x1FFF => (second_last as u8, address & 0x1FFF),
                0x2000..=0x3FFF => (last_bank as u8, address & 0x1FFF),
                0x4000..=0x5FFF => (self.r[6] as u8, address & 0x1FFF),
                0x6000..=0x7FFF => (self.r[7] as u8, address & 0x1FFF),
                _ => (0, 0),
            }
        };
        
        let bank = (bank as usize) % num_8k_banks as usize;
        (bank * 8192) + offset
    }

    fn update_mapper_cpu(&mut self, address: u16, value: u8) {
        match address {
            // Bank select ($8000-$9FFE, even)
            0x8000..=0x9FFE if address & 1 == 0 => {
                self.bank_select = value;
            }
            // Bank data ($8001-$9FFF, odd)
            0x8001..=0x9FFF if address & 1 == 1 => {
                let reg = self.selected_register();
                self.r[reg] = value;
            }
            // Mirroring ($A000-$BFFE, even)
            0xA000..=0xBFFE if address & 1 == 0 => {
                self.mirroring = if value & 1 == 0 {
                    Mirroring::Vertical
                } else {
                    Mirroring::Horizontal
                };
            }
            // PRG-RAM protect ($A001-$BFFF, odd)
            0xA001..=0xBFFF if address & 1 == 1 => {
                self.prg_ram_enabled = value & 0x80 != 0;
                self.prg_ram_write_protect = value & 0x40 != 0;
            }
            // IRQ latch ($C000-$DFFE, even)
            0xC000..=0xDFFE if address & 1 == 0 => {
                self.irq_latch = value;
            }
            // IRQ reload ($C001-$DFFF, odd)
            0xC001..=0xDFFF if address & 1 == 1 => {
                self.irq_reload = true;
            }
            // IRQ disable ($E000-$FFFE, even)
            0xE000..=0xFFFE if address & 1 == 0 => {
                self.irq_enabled = false;
                self.irq_pending = false;
            }
            // IRQ enable ($E001-$FFFF, odd)
            0xE001..=0xFFFF if address & 1 == 1 => {
                self.irq_enabled = true;
            }
            _ => {}
        }
    }

    fn convert_ppu_address(&self, chr_rom: Vec<u8>, address: u16) -> usize {
        let num_1k_banks = (chr_rom.len() / 1024) as u8;
        let addr = (address & 0x1FFF) as usize;
        
        let (bank, offset) = if !self.chr_inversion() {
            match addr {
                0x0000..=0x07FF => {
                    let b = self.r[0] & 0xFE;
                    (b as usize, addr & 0x07FF)
                }
                0x0800..=0x0FFF => {
                    let b = self.r[0] | 0x01;
                    (b as usize, addr & 0x07FF)
                }
                0x1000..=0x13FF => (self.r[1] as usize, addr & 0x03FF),
                0x1400..=0x17FF => (self.r[2] as usize, addr & 0x03FF),
                0x1800..=0x1BFF => (self.r[3] as usize, addr & 0x03FF),
                0x1C00..=0x1FFF => (self.r[4] as usize, addr & 0x03FF),
                _ => (0, 0),
            }
        } else {
            match addr {
                0x0000..=0x03FF => (self.r[2] as usize, addr & 0x03FF),
                0x0400..=0x07FF => (self.r[3] as usize, addr & 0x03FF),
                0x0800..=0x0BFF => (self.r[4] as usize, addr & 0x03FF),
                0x0C00..=0x0FFF => (self.r[5] as usize, addr & 0x03FF),
                0x1000..=0x17FF => {
                    let b = self.r[0] & 0xFE;
                    (b as usize, addr & 0x07FF)
                }
                0x1800..=0x1FFF => {
                    let b = self.r[0] | 0x01;
                    (b as usize, addr & 0x07FF)
                }
                _ => (0, 0),
            }
        };
        
        let bank = bank % num_1k_banks as usize;
        (bank * 1024) + offset
    }

    fn update_mapper_ppu(&mut self, _address: u16, _value: u8) {
        // CHR-ROM read-only, or CHR-RAM
    }

    fn mirroring(&self) -> Mirroring {
        self.mirroring
    }
}
