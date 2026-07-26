use core::fmt;
use std::fs;
use std::io;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Mirroring {
    Horizontal,
    Vertical,
    FourScreen,
    SingleScreenLower,
    SingleScreenUpper,
}

impl fmt::Display for Mirroring {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Horizontal => write!(f, "Horizontal"),
            Self::Vertical => write!(f, "Vertical"),
            Self::FourScreen => write!(f, "FourScreen"),
            Self::SingleScreenLower => write!(f, "SingleScreenLower"),
            Self::SingleScreenUpper => write!(f, "SingleScreenUpper"),
        }
    }
}

pub struct Cartridge {
    pub prg_rom: Vec<u8>,
    pub chr_rom: Vec<u8>,
    
    pub mapper: u8,
    pub mirroring_type: Mirroring
}

impl Cartridge {
    pub fn load(file_path: &str) -> Result<Cartridge, io::Error> {
        let bytes = fs::read(file_path)?;

        if bytes.len() < 16 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "File is not a valid ROM NES",
            ));
        }

        if &bytes[0..4] != b"NES\x1A" {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Invalid Signature, please use and iNES file",
            ));
        }

        let prg_rom_size = bytes[4] as usize * 16 * 1024;
        let chr_rom_size = bytes[5] as usize * 8 * 1024;
        
        let flag_6 = bytes[6];
        let flag_7 = bytes[7];

        // Calculate mapper from Flag 6 and Flag 7
        let mapper = (flag_7 & 0xF0) | ((flag_6 & 0xF0) >> 4);

        // Calculate mirroring from Flag 6
        let mirroring_type = if flag_6 & 0x08 != 0 {
            Mirroring::FourScreen
        } else if flag_6 & 0x01 != 0 {
            Mirroring::Vertical
        } else {
            Mirroring::Horizontal
        };

        let has_trainer = (flag_6 & 0x04) != 0;

        // Start at the offset 16 after the header 
        let prg_rom_start = if has_trainer { 528 } else { 16 };
        let prg_rom_end = prg_rom_start + prg_rom_size;
        
        let chr_rom_start = prg_rom_end;
        let chr_rom_end = chr_rom_start + chr_rom_size;

        if bytes.len() < chr_rom_end {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Corrupted file: not enough data",
            ));
        }

        let prg_rom = bytes[prg_rom_start..prg_rom_end].to_vec();
        let chr_rom = bytes[chr_rom_start..chr_rom_end].to_vec();

        Ok(Cartridge {
            prg_rom,
            chr_rom,
            mapper,
            mirroring_type,
        })
    }
}