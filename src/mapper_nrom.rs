use crate::{cartridge::Mirroring, mapper::Mapper};


pub struct MapperNROM;

impl MapperNROM {
    pub fn new() -> MapperNROM {
        MapperNROM
    }
}

impl Mapper for MapperNROM {
    fn convert_cpu_address(&self, prg_rom: &Vec<u8>, address: usize) -> usize {
        address % prg_rom.len()
    }

    fn update_mapper_cpu(&mut self, _address: u16, _value: u8) {
    }

    fn convert_ppu_address(&self, _prg_rom: Vec<u8>, address: u16) -> usize {
        address as usize
    }

    fn mirroring(&self) -> Mirroring {
        Mirroring::Horizontal
    }
    
    fn update_mapper_ppu(&mut self, _address: u16, _value: u8) {
        // NROM CHR-ROM is read-only
    }
}