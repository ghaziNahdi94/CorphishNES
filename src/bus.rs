use std::{io};
use crate::{apu::Apu, cartridge::{Cartridge, Mirroring}, mapper::Mapper, mapper_axrom::MapperAxROM, mapper_cnrom::MapperCNROM, mapper_mmc1::MapperMMC1, mapper_mmc3::MapperMMC3, mapper_nrom::MapperNROM, mapper_unrom::MapperUNROM, ppu::Ppu};

pub struct Bus {
    pub ram_cpu: [u8; 2048],
    pub cartridge: Option<Cartridge>,
    pub ppu: Option<Ppu>,
    pub apu: Apu,
    pub mapper: Option<Box<dyn Mapper>>,
    // === IO / Controllers ===

    pub controller_state: [u8; 2],   // joystick (bit 0=A, 1=B, 2=Select, 3=Start, 4=Up, 5=Down, 6=Left, 7=Right)
    controller_shift: [u8; 2],       // shift registers
    controller_strobe: bool,         // Bit strobe active
}

impl Bus {

    pub fn new() -> Bus {
        Bus {
            ram_cpu: [0; 2048],
            cartridge: None,
            ppu: None,
            apu: Apu::new(),
            mapper: None,
            controller_state: [0; 2],
            controller_shift: [0; 2],
            controller_strobe: false,
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
            0x4000..=0x401F => {
                match address {
                    // APU Status ($4015) — return 0 (there is no APU)
                    0x4015 => self.apu.read_cpu(address),
                
                    // Controller 1 read ($4016)
                    0x4016 => {
                        let bit = self.controller_shift[0] & 0x01;
                        self.controller_shift[0] >>= 1;
                        self.controller_shift[0] |= 0x80; // NES shift-in  1 after 8 buttons
                        bit
                    }
                
                    // Controller 2 read ($4017)
                    0x4017 => {
                        let bit = self.controller_shift[1] & 0x01;
                        self.controller_shift[1] >>= 1;
                        self.controller_shift[1] |= 0x80;
                        bit
                    }
                
                    _ => 0,
                }
            },
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
            0x4000..=0x401F => {
                match address {
                    // APU registers ($4000-$4013, $4015, $4017)
                    0x4000..=0x4013 | 0x4015 | 0x4017 => {
                        self.apu.write_cpu(address, value); 
                    }
                
                    // === OAMDMA ($4014) ===
                    // Transfer 256 bytes from the page CPU to the PPU OAM 
                    0x4014 => {
                        let page = (value as u16) << 8;
                        let mut buffer = [0u8; 256];
                    
                        // Read CPU page
                        for i in 0..256 {
                            buffer[i] = self.read(page + i as u16);
                        }
                    
                        // write on the OAM of the PPU
                        if let Some(ppu) = self.ppu.as_mut() {
                            let start = ppu.oam_address_register as usize;
                            for i in 0..256 {
                                ppu.sprite_oam[(start + i) & 0xFF] = buffer[i];
                            }
                            // NES: oam_address wraps to 0 after DMA (or stays? actually it wraps)
                            ppu.oam_address_register = ppu.oam_address_register.wrapping_add(0); // stays, but DMA always writes full page

                        }
                    }
                
                    // === Controller strobe ($4016) ===
                    0x4016 => {
                        self.controller_strobe = value & 0x01 != 0;
                        if self.controller_strobe {
                            // Set shift register when strbe = 1
                            self.controller_shift[0] = self.controller_state[0];
                            self.controller_shift[1] = self.controller_state[1];
                        }
                    }
                
                    _ => {}
                }
            },
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