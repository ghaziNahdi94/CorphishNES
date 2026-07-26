use std::{io};
use crate::{cartridge::{Cartridge, Mirroring}, mapper::Mapper, mapper_axrom::MapperAxROM, mapper_cnrom::MapperCNROM, mapper_mmc1::MapperMMC1, mapper_mmc3::MapperMMC3, mapper_nrom::MapperNROM, mapper_unrom::MapperUNROM, ppu::Ppu};

pub struct Bus {
    pub ram_cpu: [u8; 2048],
    pub cartridge: Option<Cartridge>,
    pub ppu: Option<Ppu>,
    pub mapper: Option<Box<dyn Mapper>>,
}

impl Bus {

    pub fn new() -> Bus {
        Bus {
            ram_cpu: [0; 2048],
            cartridge: None,
            ppu: None,
            mapper: None
        }
    }

    pub fn read(&mut self, address: u16) -> u8 {
        match address {
            0x0000..=0x1FFF => self.ram_cpu[(address & 0x07FF) as usize],
            0x2000..=0x3FFF => {
                if let Some(ppu) = self.ppu.as_mut() {
                    ppu.read_cpu(0x2000 + (address & 0x07))
                } else {
                    0
                }
            },
            0x4000..=0x401F => 0, // APU TODO
            0x8000..=0xFFFF => {
                if let (Some(cartridge), Some(mapper)) =  (&self.cartridge, &self.mapper) {
                    let rom_index = (address - 0x8000) as usize;
                    let mapped_index = mapper.convert_cpu_address(&cartridge.prg_rom, rom_index);
                    cartridge.prg_rom[mapped_index]
                } else {
                    0x00
                }
            }
            _ => 0,
        }
    }

    pub fn write(&mut self, address: u16, value: u8) {
        match address {
            0x0000..=0x1FFF => self.ram_cpu[(address & 0x07FF) as usize] = value,
            0x2000..=0x3FFF => {
                if let Some(ppu) = self.ppu.as_mut() {
                    ppu.write_cpu(0x2000 + (address & 0x07), value);
                }
            },
            0x4000..=0x401F => todo!(),
            0x8000..=0xFFFF => {
                if let Some(mapper) = &mut self.mapper {
                    mapper.update_mapper_cpu(address, value);
                }
            }
            _ => {}
        }
    }

    pub fn init_ppu(&mut self, chr: &Vec<u8>, mirroring: Mirroring) {
        self.ppu = Some(Ppu::new(chr.to_vec(), mirroring));
    }

    pub fn load_cartridge(&mut self, file_path: &str) -> Result<(), io::Error> {
        let cartridge = Cartridge::load(file_path)?;
        println!("Cartridge loaded, Mapper {}", cartridge.mapper);

        self.mapper = match cartridge.mapper {
            0 => Some(Box::new(MapperNROM::new())),
            1 => Some(Box::new(MapperMMC1::new(
                cartridge.prg_rom.len(),
                cartridge.chr_rom.len(),
            ))),
            2 => Some(Box::new(MapperUNROM::new(cartridge.prg_rom.len()))),
            3 => Some(Box::new(MapperCNROM::new(
                cartridge.chr_rom.len(),
                if cartridge.mirroring_type == Mirroring::Horizontal { 0 } else { 1 },
            ))),
            4 => Some(Box::new(MapperMMC3::new())),
            7 => Some(Box::new(MapperAxROM::new(cartridge.prg_rom.len()))),
            _ => {
                return Err(io::Error::new(
                    io::ErrorKind::Unsupported,
                    format!("Mapper {} not supported", cartridge.mapper),
                ));
            }
        };

        self.cartridge = Some(cartridge);

        Ok(())
    }
}