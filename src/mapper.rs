pub trait Mapper {
    fn convert_cpu_address(&self, prg_rom: &Vec<u8>, address: usize) -> usize;

    fn update_mapper_cpu(&mut self, address: u16, value: u8);

    fn update_mapper_ppu(&mut self, address: u16, value: u8);

    fn convert_ppu_address(&self, prg_rom: Vec<u8>, address: u16) -> usize;

    fn mirroring(&self) -> crate::cartridge::Mirroring;
}