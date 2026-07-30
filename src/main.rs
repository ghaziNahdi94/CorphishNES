use minifb::{Key, Scale, Window, WindowOptions};
use crate::emulator::Emulator;

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

const WIDTH: usize = 256;
const HEIGHT: usize = 240;

fn main() {

    let mut emulator = Emulator::new();
    
    if let Err(e) = emulator.load_rom("/Volumes/Crucial X9/Ghazi/Games/NES/Super_Mario_Bros.nes") {
        eprintln!("Failed to load ROM: {}", e);
        return;
    }

    let mut window = Window::new(
        "CorphishNES Emulator",
        WIDTH,
        HEIGHT,
        WindowOptions {
        scale: Scale::X4,
        ..WindowOptions::default()
    },
    ).expect("Failed to create window");

    window.set_target_fps(60);

    while window.is_open() && !window.is_key_down(Key::Escape) {

        let mut controller = 0u8;
        if window.is_key_down(Key::Z)      { controller |= 0x01; } // A
        if window.is_key_down(Key::X)      { controller |= 0x02; } // B
        if window.is_key_down(Key::Enter)  { controller |= 0x04; } // Select
        if window.is_key_down(Key::Space)  { controller |= 0x08; } // Start
        if window.is_key_down(Key::Up)     { controller |= 0x10; } // Up
        if window.is_key_down(Key::Down)   { controller |= 0x20; } // Down
        if window.is_key_down(Key::Left)   { controller |= 0x40; } // Left
        if window.is_key_down(Key::Right)  { controller |= 0x80; } // Right
        
        emulator.bus.controller_state[0] = controller;


        // execute a Frame
        emulator.run_frame();

        // Get framebuffer and convert to u32 RGB
        let framebuffer = emulator.get_framebuffer();

        let buffer: Vec<u32> = framebuffer
            .chunks_exact(3)
            .map(|rgb| {
                let r = rgb[0] as u32;
                let g = rgb[1] as u32;
                let b = rgb[2] as u32;
                (r << 16) | (g << 8) | b
            })
            .collect();
        

        window.update_with_buffer(&buffer, WIDTH, HEIGHT)
            .expect("Failed to update window");
    }
}