use std::process;

use crate::{bus::Bus, cartridge::Cartridge, cpu::Cpu};

mod emulator;
mod bus;
mod cpu;
mod ppu;
mod cartridge;

fn main() {
    let _cartridge = 
        Cartridge::load("/Volumes/Crucial X9/Ghazi/Games/NES/Super_Mario_Bros.nes");

    let mut bus = Bus::new();
    if let Err(e) = bus.load_cartridge("/Volumes/Crucial X9/Ghazi/Games/NES/Super_Mario_Bros.nes") {
        println!("Error : {}", e);
        process::exit(1);
    }

    

    let mut cpu = Cpu::init();
}
