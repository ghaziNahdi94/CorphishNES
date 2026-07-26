#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Mirroring {
    Horizontal,
    Vertical,
    SingleScreen,
    FourScreen,
}

pub struct Ppu {
    vram: [u8; 2048],        // Nametables (2KB)
    palette: [u8; 32],       // Palettes
    oam: [u8; 256],          // Object Attribute Memory (64 sprites × 4 bytes)

    // Internal registers
    v: u16,                  // Current VRAM address (15 bits)
    t: u16,                  // Temporary VRAM address (15 bits)
    x: u8,                   // Fine X scroll (3 bits)
    w: bool,                 // First/second write toggle

    // CPU-visible registers
    ctrl: u8,
    mask: u8,
    status: u8,
    oam_address: u8,
    
    // $2007 read buffer (internal, not directly visible to CPU)
    // Quand on lit $2007, si v < $3F00, on retourne cette valeur
    // et on charge la vraie valeur VRAM ici pour le prochain read.
    ppu_data_buffer: u8,

    // CHR-ROM/RAM from cartridge
    cartridge_chr: Vec<u8>,
    mirroring: Mirroring,

    // Timing
    scanline: u16,
    cycle: u16,
    frames_counter: u64,

    // Framebuffer RGB
    framebuffer: [u8; 256 * 240 * 3],
}

impl Ppu {
    pub fn new(chr: Vec<u8>, mirroring: Mirroring) -> Self {
        Self {
            vram: [0; 2048],
            palette: [0; 32],
            oam: [0; 256],
            v: 0,
            t: 0,
            x: 0,
            w: false,
            ctrl: 0,
            mask: 0,
            status: 0,
            oam_address: 0,
            ppu_data_buffer: 0,
            cartridge_chr: chr,
            mirroring,
            scanline: 261,        // Start on pre-render scanline
            cycle: 0,
            frames_counter: 0,
            framebuffer: [0; 256 * 240 * 3],
        }
    }

    pub fn step(&mut self) -> bool {
        let mut nmi_triggered = false;

        if self.cycle == 1 {
            match self.scanline {
                241 => {
                    self.status |= 0x80; // VBlank = 1
                    if self.ctrl & 0x80 != 0 {
                        nmi_triggered = true;
                    }
                }
                261 => {
                    // End of VBlank + pre-render
                    self.status &= !0x80; // VBlank = 0
                    self.status &= !0x40; // Sprite 0 hit = 0
                    self.status &= !0x20; // Sprite overflow = 0
                }
                _ => {}
            }
        }

        // === Render (scanlines visibles 0-239) ===
        if self.scanline < 240 && self.cycle > 0 && self.cycle <= 256 {
            let x = (self.cycle - 1) as usize;
            let y = self.scanline as usize;
            let idx = (y * 256 + x) * 3;

            // TODO: Render background + sprites
            self.framebuffer[idx] = 0;     // R
            self.framebuffer[idx + 1] = 0;   // G
            self.framebuffer[idx + 2] = 0;   // B
        }

        self.cycle += 1;
        if self.cycle > 340 {
            self.cycle = 0;
            self.scanline += 1;
            if self.scanline > 261 {
                self.scanline = 0;
                self.frames_counter += 1;
            }
        }

        nmi_triggered
    }

    // =========================================================================
    // CPU READ
    // =========================================================================
    pub fn read_cpu(&mut self, address: u16) -> u8 {
        match address {
            // PPUSTATUS ($2002)
            0x2002 => {
                let val = self.status;
                self.status &= !0x80; // Clear VBlank flag
                self.w = false;       // Reset write toggle
                val
            }

            // OAMDATA ($2004)
            0x2004 => {
                self.oam[self.oam_address as usize]
            }

            // PPUDATA ($2007)
            0x2007 => {
                let val = if self.v < 0x3F00 {
                    let buffered = self.ppu_data_buffer;
                    self.ppu_data_buffer = self.read_vram(self.v);
                    buffered
                } else {
                    self.ppu_data_buffer = self.read_vram(self.v & 0x2FFF);
                    self.read_vram(self.v)
                };

                // Incrément de v (bit 2 de ctrl : 0 = +1, 1 = +32)
                self.v = self.v.wrapping_add(if self.ctrl & 0x04 == 0 { 1 } else { 32 });
                val
            }
            _ => 0,
        }
    }

    // =========================================================================
    // CPU WRITE
    // =========================================================================
    pub fn write_cpu(&mut self, address: u16, value: u8) {
        match address {
            0x2000 => self.ctrl = value,
            0x2001 => self.mask = value,
            
            // OAMADDR ($2003)
            0x2003 => self.oam_address = value,

            // OAMDATA ($2004)
            0x2004 => {
                self.oam[self.oam_address as usize] = value;
                self.oam_address = self.oam_address.wrapping_add(1);
            }

            // PPUSCROLL ($2005)
            0x2005 => {
                if !self.w {
                    // X scroll (Write 1)
                    self.x = value & 0x07;
                    self.t = (self.t & 0xFFE0) | ((value as u16) >> 3);
                    self.w = true;
                } else {
                    // Y scroll (Write 2)
                    self.t = (self.t & 0x8C1F)
                        | (((value as u16) & 0x07) << 12)
                        | (((value as u16) & 0xF8) << 2);
                    self.w = false;
                }
            }

            // PPUADDR ($2006)
            0x2006 => {
                if !self.w {
                    // High Byte (Write 1)
                    self.t = (self.t & 0x00FF) | (((value as u16) & 0x3F) << 8);
                    self.w = true;
                } else {
                    // Low Byte (Write 2)
                    self.t = (self.t & 0xFF00) | (value as u16);
                    self.v = self.t;
                    self.w = false;
                }
            }

            // PPUDATA ($2007)
            0x2007 => {
                self.write_vram(self.v, value);
                self.v = self.v.wrapping_add(if self.ctrl & 0x04 == 0 { 1 } else { 32 });
            }

            _ => {}
        }
    }

    // =========================================================================
    // VRAM READ
    // =========================================================================
    fn read_vram(&self, address: u16) -> u8 {
        let address = address & 0x3FFF;

        if address < 0x2000 {
            // CHR-ROM / CHR-RAM
            // Si c'est du CHR-ROM (read-only), la cartouche doit l'empêcher en write,
            // mais en read c'est toujours OK.
            self.cartridge_chr[address as usize]
        } else if address < 0x3F00 {
            // Nametables ($2000-$3EFF, avec mirror $3000-$3EFF)
            let mirrored = self.apply_mirroring(address);
            self.vram[(mirrored as usize) & 0x07FF]
        } else {
            // Palettes ($3F00-$3FFF)
            let mut index = address & 0x1F;
            
            // Mirroring des "color 0" des sprites vers le background
            // $3F10, $3F14, $3F18, $3F1C -> $3F00, $3F04, $3F08, $3F0C
            if index >= 0x10 && (index & 0x03) == 0 {
                index -= 0x10;
            }

            self.palette[index as usize]
        }
    }

    // =========================================================================
    // VRAM WRITE
    // =========================================================================
    fn write_vram(&mut self, address: u16, value: u8) {
        let address = address & 0x3FFF;

        if address < 0x2000 {
            // CHR-RAM 
            if self.cartridge_chr.len() > address as usize {
                self.cartridge_chr[address as usize] = value;
            }
        } else if address < 0x3F00 {
            // Nametables
            let mirrored = self.apply_mirroring(address);
            self.vram[(mirrored as usize) & 0x07FF] = value;
        } else {
            // Palettes
            let mut index = address & 0x1F;
            if index >= 0x10 && (index & 0x03) == 0 {
                index -= 0x10;
            }
            self.palette[index as usize] = value;
        }
    }

    // =========================================================================
    // MIRRORING
    // =========================================================================
    fn apply_mirroring(&self, address: u16) -> u16 {
        let offset = address & 0x03FF;
        let table = (address >> 10) & 0x03;

        let mapped_table = match self.mirroring {
            Mirroring::Vertical => table & 0x01,          // ABAB
            Mirroring::Horizontal => (table & 0x02) >> 1, // AABB
            Mirroring::SingleScreen => 0,                 // AAAA
            Mirroring::FourScreen => table,               // ABCD (nécessite RAM cartouche)
        };

        0x2000 | (mapped_table << 10) | offset
    }
}