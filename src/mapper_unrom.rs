use crate::{cartridge::Mirroring, mapper::Mapper};

pub struct MapperUNROM {
    prg_bank: u8,  
    num_banks: u8, 
}

impl MapperUNROM {
    pub fn new(prg_size: usize) -> Self {
        let num_banks = (prg_size / 16384) as u8;
        MapperUNROM {
            prg_bank: 0,
            num_banks,
        }
    }
}

impl Mapper for MapperUNROM {
    fn convert_cpu_address(&self, _prg_rom: &Vec<u8>, address: usize) -> usize {
        let bank = if address < 0x4000 {
            self.prg_bank
        } else {
            self.num_banks - 1
        };
        
        let offset_in_bank = address & 0x3FFF;
        (bank as usize * 16384) + offset_in_bank
    }

    fn update_mapper_cpu(&mut self, _address: u16, value: u8) {
        self.prg_bank = value & 0x0F;

        if self.prg_bank >= self.num_banks {
            self.prg_bank = self.num_banks - 1;
        }
    }

    fn convert_ppu_address(&self, _chr_rom: Vec<u8>, address: u16) -> usize {
        address as usize
    }

    fn update_mapper_ppu(&mut self, _address: u16, _value: u8) {
    }

    fn mirroring(&self) -> Mirroring {
        Mirroring::Horizontal
    }
}