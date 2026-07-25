use crate::{cartridge::Mirroring, mapper::Mapper};

pub struct MapperMMC1 {
    shift_reg: u8,     
    shift_count: u8,   
    control: u8,       
    chr_bank_0: u8,    
    chr_bank_1: u8,    
    prg_bank: u8,      
    prg_ram_enabled: bool,
    num_prg_banks: u8,
    num_chr_banks: u8,
}

impl MapperMMC1 {
    pub fn new(prg_size: usize, chr_size: usize) -> Self {
        let num_prg_banks = (prg_size / 16384) as u8;
        let num_chr_banks = if chr_size > 0 { (chr_size / 4096) as u8 } else { 0 };
        
        MapperMMC1 {
            shift_reg: 0,
            shift_count: 0,
            control: 0x0C,  // PRG mode = 3 
            chr_bank_0: 0,
            chr_bank_1: 0,
            prg_bank: 0,
            prg_ram_enabled: false,
            num_prg_banks,
            num_chr_banks,
        }
    }
    
    fn apply_register(&mut self, address: u16) {
        let reg_value = self.shift_reg & 0x1F;
        
        match address {
            0x8000..=0x9FFF => {
                self.control = reg_value;
            }
            0xA000..=0xBFFF => {
                self.chr_bank_0 = reg_value;
            }
            0xC000..=0xDFFF => {
                self.chr_bank_1 = reg_value;
            }
            0xE000..=0xFFFF => {
                self.prg_bank = reg_value & 0x0F;
                self.prg_ram_enabled = (reg_value & 0x10) == 0;
            }
            _ => {}
        }
    }
    
    fn prg_mode(&self) -> u8 {
        (self.control >> 2) & 0x03
    }
    
    fn chr_mode(&self) -> u8 {
        (self.control >> 4) & 0x01
    }
    
    fn mirroring_from_control(&self) -> Mirroring {
        match self.control & 0x03 {
            0 => Mirroring::SingleScreenLower,
            1 => Mirroring::SingleScreenUpper,
            2 => Mirroring::Vertical,
            3 => Mirroring::Horizontal,
            _ => unreachable!(),
        }
    }
}

impl Mapper for MapperMMC1 {
    fn convert_cpu_address(&self, prg_rom: &Vec<u8>, address: usize) -> usize {
        let mode = self.prg_mode();
        let num_banks = (prg_rom.len() / 16384) as u8;
        
        let bank = match mode {
            0 | 1 => {
                // 32 KB mode
                let bank_32k = (self.prg_bank & 0x0E) >> 1;
                if address < 0x4000 {
                    bank_32k * 2
                } else {
                    bank_32k * 2 + 1
                }
            }
            2 => {
                // Fixe first bank + switchable
                if address < 0x4000 {
                    0
                } else {
                    self.prg_bank
                }
            }
            3 => {
                if address < 0x4000 {
                    self.prg_bank
                } else {
                    num_banks - 1
                }
            }
            _ => unreachable!(),
        };
        
        let offset_in_bank = address & 0x3FFF;
        (bank as usize * 16384) + offset_in_bank
    }

    fn update_mapper_cpu(&mut self, address: u16, value: u8) {
        if value & 0x80 != 0 {
            // Reset
            self.shift_reg = 0;
            self.shift_count = 0;
            self.control |= 0x0C;
            return;
        }
        
        self.shift_reg = ((value & 1) << 4) | (self.shift_reg >> 1);
        self.shift_count += 1;
        
        if self.shift_count == 5 {
            self.apply_register(address);
            self.shift_reg = 0;
            self.shift_count = 0;
        }
    }

    fn convert_ppu_address(&self, chr_rom: Vec<u8>, address: u16) -> usize {
        if chr_rom.is_empty() {
            // CHR-RAM
            return address as usize;
        }
        
        let mode = self.chr_mode();
        let num_1k_banks = (chr_rom.len() / 1024) as u8;
        
        let bank = if mode == 0 {
            // 8 KB mode
            let bank_8k = (self.chr_bank_0 & 0x1E) >> 1;
            let offset = (address & 0x1FFF) as usize;
            return (bank_8k as usize * 8192) + offset;
        } else {
            // 4 KB mode
            if address < 0x1000 {
                self.chr_bank_0
            } else {
                self.chr_bank_1
            }
        };
        
        let offset = (address & 0x0FFF) as usize;
        ((bank as usize % num_1k_banks as usize) * 4096) + offset
    }

    fn update_mapper_ppu(&mut self, _address: u16, _value: u8) {
        // CHR-ROM read-only, or CHR-RAM 
    }

    fn mirroring(&self) -> Mirroring {
        self.mirroring_from_control()
    }
}
