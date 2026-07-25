use crate::{cartridge::Mirroring, mapper::Mapper};

pub struct MapperAxROM {
    prg_bank: u8,
    num_banks: u8,
    mirroring_upper: bool,
}

impl MapperAxROM {
    pub fn new(prg_size: usize) -> Self {
        let num_banks = (prg_size / 32768) as u8;
        MapperAxROM {
            prg_bank: 0,
            num_banks,
            mirroring_upper: false,
        }
    }
}

impl Mapper for MapperAxROM {
    fn convert_cpu_address(&self, _prg_rom: &Vec<u8>, address: usize) -> usize {
        let offset = address & 0x7FFF;
        (self.prg_bank as usize * 32768) + offset
    }

    fn update_mapper_cpu(&mut self, _address: u16, value: u8) {
        self.prg_bank = value & 0x07;
        if self.prg_bank >= self.num_banks {
            self.prg_bank = self.num_banks - 1;
        }
        self.mirroring_upper = value & 0x10 != 0;
    }

    fn convert_ppu_address(&self, _chr_rom: Vec<u8>, address: u16) -> usize {
        address as usize
    }

    fn update_mapper_ppu(&mut self, _address: u16, _value: u8) {
        // CHR-RAM 
    }

    fn mirroring(&self) -> Mirroring {
        if self.mirroring_upper {
            Mirroring::SingleScreenUpper
        } else {
            Mirroring::SingleScreenLower
        }
    }
}
