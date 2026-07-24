pub struct Ppu {
    name_table: [u8; 2048],
    palette_table: [u8; 32]
}

impl Ppu {
    pub fn init() -> Ppu {
        Ppu { 
            name_table: [0; 2048], 
            palette_table: [0; 32] 
        }
    }
}