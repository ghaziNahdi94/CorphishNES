use crate::{cartridge::Mirroring, mapper::Mapper};

pub struct MapperCNROM {
    chr_bank: u8,
    num_chr_banks: u8,
    mirror_type: u8,
}

impl MapperCNROM {
    pub fn new(chr_size: usize, mirror_type: u8) -> Self {
        let num_chr_banks = (chr_size / 8192) as u8;
        MapperCNROM {
            chr_bank: 0,
            num_chr_banks,
            mirror_type,
        }
    }
}

impl Mapper for MapperCNROM {
    fn convert_cpu_address(&self, _prg_rom: &Vec<u8>, address: usize) -> usize {
        address
    }

    fn update_mapper_cpu(&mut self, _address: u16, value: u8) {
        self.chr_bank = value & 0x03;
        if self.chr_bank >= self.num_chr_banks {
            self.chr_bank = self.num_chr_banks.saturating_sub(1);
        }
    }

    fn convert_ppu_address(&self, _chr_rom: Vec<u8>, address: u16) -> usize {
        let offset_in_bank = (address & 0x1FFF) as usize;
        (self.chr_bank as usize * 8192) + offset_in_bank
    }

    fn update_mapper_ppu(&mut self, _address: u16, _value: u8) {
        // CHR-ROM read-only
    }

    fn mirroring(&self) -> Mirroring {
        if self.mirror_type == 0 {
            Mirroring::Horizontal
        } else {
            Mirroring::Vertical
        }
    }
}
