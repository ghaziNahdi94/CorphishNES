use std::{thread, time::{Duration, Instant}};

use minifb::{Key, Window, WindowOptions};
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
const TARGET_FPS: u64 = 60;
const FRAME_TIME: Duration = Duration::from_nanos(1_000_000_000 / TARGET_FPS); // ~16.67ms

fn main() {

    let mut emulator = Emulator::new();
    
    if let Err(e) = emulator.load_rom("/Volumes/Crucial X9/Ghazi/Games/NES/Super_Mario_Bros.nes") {
        eprintln!("Failed to load ROM: {}", e);
        return;
    }

    let mut window = Window::new(
        "NES Emulator",
        WIDTH,
        HEIGHT,
        WindowOptions::default(),
    ).expect("Failed to create window");

    while window.is_open() && !window.is_key_down(Key::Escape) {
        let frame_start = Instant::now();

        // Exécute une frame complète
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

        // Manual frame rate limiting
        let elapsed = frame_start.elapsed();
        if elapsed < FRAME_TIME {
            thread::sleep(FRAME_TIME - elapsed);
        }
    }
}