use crate::{bus::Bus};

mod emulator;
mod bus;
mod cpu;
mod ppu;
mod cartridge;
mod mapper;
mod mapper_nrom;
mod mapper_axrom;
mod mapper_cnrom;
mod mapper_mmc1;
mod mapper_mmc3;
mod mapper_unrom;

fn main() {

    let mut bus = Bus::new();

    match bus.load_cartridge("/Volumes/Crucial X9/Ghazi/Games/NES/Super_Mario_Bros.nes") {
        Ok(()) => {
            // Test : read reset vector
            let low = bus.read(0xFFFC);
            let high = bus.read(0xFFFD);
            let reset_addr = ((high as u16) << 8) | (low as u16);
            println!("Reset vector: ${:04X}", reset_addr);
            
        }
        Err(e) => eprintln!("Error : {}", e),
    }



    
}
