use crate::{cartridge::Mirroring};

const NES_PALETTE: [(u8, u8, u8); 64] = [
    // $00-$0F
    (0x75, 0x75, 0x75), (0x27, 0x1B, 0x8F), (0x00, 0x00, 0xAB), (0x47, 0x00, 0x9F),
    (0x8F, 0x00, 0x77), (0xAB, 0x00, 0x13), (0xA7, 0x00, 0x00), (0x7F, 0x0B, 0x00),
    (0x43, 0x2F, 0x00), (0x00, 0x47, 0x00), (0x00, 0x51, 0x00), (0x00, 0x3F, 0x17),
    (0x1B, 0x3F, 0x5F), (0x00, 0x00, 0x00), (0x00, 0x00, 0x00), (0x00, 0x00, 0x00),
    
    // $10-$1F
    (0xBC, 0xBC, 0xBC), (0x00, 0x73, 0xEF), (0x23, 0x3B, 0xEF), (0x83, 0x00, 0xF3),
    (0xBF, 0x00, 0xBF), (0xE7, 0x00, 0x5B), (0xDB, 0x2B, 0x00), (0xCB, 0x4F, 0x0F),
    (0x8B, 0x73, 0x00), (0x00, 0x97, 0x00), (0x00, 0xAB, 0x00), (0x00, 0x93, 0x3B),
    (0x00, 0x83, 0x8B), (0x00, 0x00, 0x00), (0x00, 0x00, 0x00), (0x00, 0x00, 0x00),
    
    // $20-$2F
    (0xFF, 0xFF, 0xFF), (0x3F, 0xBF, 0xFF), (0x5F, 0x97, 0xFF), (0xA7, 0x8B, 0xFD),
    (0xF7, 0x7B, 0xFF), (0xFF, 0x77, 0xB7), (0xFF, 0x77, 0x63), (0xFF, 0x9F, 0x43),
    (0xF3, 0xBF, 0x3F), (0x83, 0xD3, 0x13), (0x4F, 0xDF, 0x4B), (0x58, 0xF8, 0x98),
    (0x00, 0xEB, 0xDB), (0x00, 0x00, 0x00), (0x00, 0x00, 0x00), (0x00, 0x00, 0x00),
    
    // $30-$3F
    (0xFF, 0xFF, 0xFF), (0xAB, 0xE7, 0xFF), (0xC7, 0xD7, 0xFF), (0xD7, 0xCB, 0xFF),
    (0xFF, 0xC7, 0xFF), (0xFF, 0xC7, 0xDB), (0xFF, 0xBF, 0xB3), (0xFF, 0xDB, 0xAB),
    (0xFF, 0xE7, 0xA3), (0xE3, 0xFF, 0xA3), (0xAB, 0xF3, 0xBF), (0xB3, 0xFF, 0xCF),
    (0x9F, 0xFF, 0xF3), (0x00, 0x00, 0x00), (0x00, 0x00, 0x00), (0x00, 0x00, 0x00),
];

const FRAME_WIDTH: usize = 256;
const FRAME_HEIGHT: usize = 240;

#[derive(Clone, Debug, PartialEq)]
struct Sprite {
    position_x: u8,
    position_y: u8,
    attributes: u8,
    tile_index: u8
}

#[derive(Clone, Debug, PartialEq)]
pub struct Ppu {
    vram: [u8; 4096],         // Nametables
    palette: [u8; 32],       // Palettes
    pub oam: [u8; 256],     // Object Attribute Memory (64 sprites × 4 bytes)

    // Internal registers
    v: u16,                  // Current VRAM address (15 bits)
    t: u16,                  // Temporary VRAM address (15 bits)
    x: u8,                   // Fine X scroll (3 bits)
    w: bool,                 // First/second write toggle

    // CPU-visible registers
    ctrl: u8,
    mask: u8,
    status: u8,
    pub oam_address: u8,
    
    // $2007 read buffer (internal, not directly visible to CPU)
    ppu_data_latch: u8,

    // CHR-ROM/RAM from cartridge
    cartridge_chr: Vec<u8>,
    mirroring: Mirroring,

    // Timing
    scanline: u16,
    cycle: u16,
    frames_counter: u64,

    pub nmi_pending: bool,
    sprites_buffer: Vec<Sprite>,

    // Framebuffer RGB
    pub framebuffer: [u8; FRAME_WIDTH * FRAME_HEIGHT * 3],
}

impl Ppu {
    pub fn new(chr: Vec<u8>, mirroring: Mirroring) -> Self {
        let cartridge_chr = if chr.is_empty() {
            vec![0u8; 8192]  // CHR-RAM (For Games using CHR-RAM)
        } else {
            chr
        };

        Self {
            vram: [0; 4096],
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
            ppu_data_latch: 0,
            cartridge_chr: cartridge_chr,
            mirroring: mirroring,
            scanline: 261,        // Start on pre-render scanline
            cycle: 0,
            nmi_pending: false,
            sprites_buffer: Vec::new(),
            frames_counter: 0,
            framebuffer: [0; FRAME_WIDTH * FRAME_HEIGHT * 3],
        }
    }

    pub fn step(&mut self) {        

        if self.scanline == 240 && self.cycle == 1 {
            println!("Frame {} ended, S0HIT={}", self.frames_counter, (self.status >> 6) & 1);
        }

        // ============================================
        // VBLANK / STATUS FLAGS (cycle 1 of scanline)
        // ============================================
        if self.cycle == 1 {
            match self.scanline {
                241 => {
                    // Set VBlank flag, trigger NMI if enabled
                    self.status |= 0x80;
                    if self.ctrl & 0x80 != 0 {
                        self.nmi_pending = true;
                    }
                }
                261 => {
                    // Clear VBlank, sprite 0 hit, and overflow flags at pre-render
                    let before = self.status;
                    self.status &= !0x80;
                    self.status &= !0x40;
                    self.status &= !0x20;
                    println!("CLEAR frame={} scanline=261 status {:02X} -> {:02X}", 
                    self.frames_counter, before, self.status);
                }
                _ => {}
            }
        }


        // ============================================
        // SPRITE EVALUATION (cycle 257 only)
        // ============================================
        // On the real NES, this happens during cycles 257-320, but we can
        // simplify by doing it once at cycle 257. We scan all 64 OAM entries
        // and select up to 8 sprites that intersect the NEXT scanline.
        if self.scanline < 240 && self.cycle >= 257 && self.cycle <= 320 {
            self.sprites_buffer.clear();
            let mut overflow = false;
        
            for i in 0..64 {
                // Each OAM entry is 4 bytes: [y, tile, attr, x]
                let sprite_y = self.oam[i * 4];
                
                if sprite_y >= 239 {
                    continue; // Sprite is off-screen
                }

                let y_start = sprite_y as u16 + 1;
                let y_end = y_start + 8;
                if self.scanline >= y_start && self.scanline < y_end && self.scanline < 240 {

                        if self.sprites_buffer.len() < 8 {
                            let sprite_data = &self.oam[i * 4..i * 4 + 4];
                            let sprite = Sprite {
                                position_x: sprite_data[3],
                                position_y: sprite_y,
                                tile_index: sprite_data[1],
                                attributes: sprite_data[2],
                            };
                        self.sprites_buffer.push(sprite);
                    } else {
                        // More than 8 sprites on this scanline — set overflow flag
                        overflow = true;
                    }
                }
            }
        
            if overflow {
                self.status |= 0x20;
            }
        }
    
        // ============================================
        // RENDERING (visible scanlines 0-239, cycles 1-256)
        // ============================================
        let bg_enabled = self.mask & 0x08 != 0;
        let sprites_enabled = self.mask & 0x10 != 0;
    
        if self.scanline < (FRAME_HEIGHT as u16) 
            && self.cycle > 0 
            && self.cycle <= (FRAME_WIDTH as u16) {
            
            let screen_x = (self.cycle - 1) as usize;
            let screen_y = self.scanline as usize;
            let idx = (screen_y * FRAME_WIDTH + screen_x) * 3;
            
            // ----------------------------------------
            // BACKGROUND RENDERING
            // ----------------------------------------
            let mut bg_pixel: u8 = 0;
            let mut bg_palette: u8 = 0;
            
            if bg_enabled {
                // Nametable base from PPUCTRL bits 0-1
                let nametable = (self.ctrl & 0x03) as u16;
                let nametable_base = 0x2000u16 | (nametable << 10);
                let attr_base = 0x23C0u16 | (nametable << 10);
            
                let tile_x = (screen_x >> 3) as u8;
                let tile_y = (screen_y >> 3) as u8;
                let pixel_x = (screen_x & 0x07) as u8;
                let pixel_y = (screen_y & 0x07) as u8;
            
                // Fetch tile index from nametable
                let tile_addr = nametable_base + (tile_y as u16) * 32 + (tile_x as u16);
                let tile_index = self.read_vram(tile_addr) as u16;
            
                // Background pattern table from PPUCTRL bit 4
                let bg_table = if self.ctrl & 0x10 == 0 { 0 } else { 1 };
            
                // Get 2-bit pixel value from pattern table
                bg_pixel = self.get_tile_pixel(tile_index, bg_table, pixel_x, pixel_y);
            
                // Fetch palette from attribute table
                let attr_x = tile_x >> 2;
                let attr_y = tile_y >> 2;
                let attr_addr = attr_base + (attr_y as u16) * 8 + (attr_x as u16);
                let attr_byte = self.read_vram(attr_addr);
                let shift = ((tile_y & 0x02) << 1) | (tile_x & 0x02);
                bg_palette = (attr_byte >> shift) & 0x03;
            }
        
            // Compute background color (palette $3F00 + palette * 4 + pixel)
            let bg_color_addr = if bg_pixel == 0 {
                0x3F00  // Universal background color
            } else {
                0x3F00 + (bg_palette as u16) * 4 + (bg_pixel as u16)
            };
            let bg_color_index = self.read_vram(bg_color_addr);
        
            // ----------------------------------------
            // SPRITE RENDERING
            // ----------------------------------------
            let mut sprite_pixel: u8 = 0;
            let mut sprite_palette: u8 = 0;
            let mut sprite_priority_front = true;  // true = in front of bg
            let mut sprite_0_hit = false;
        
            if sprites_enabled {
                // Iterate through evaluated sprites (0 = highest priority)
                // We break on first non-transparent pixel found
                for (buffer_index, sprite) in self.sprites_buffer.iter().enumerate() {
                    let sprite_x = sprite.position_x as usize;
                    
                    // Check if current pixel X intersects this sprite's 8-pixel width
                    if screen_x >= sprite_x && screen_x < sprite_x + 8 {
                        let mut col = screen_x - sprite_x;
                        let mut row = screen_y - (sprite.position_y as usize + 1);
                    
                        // Apply horizontal flip (bit 6 of attributes)
                        if sprite.attributes & 0x40 != 0 {
                            col = if col <= 7 { 7 - col } else { 0 };
                        }
                        
                        // Apply vertical flip (bit 7 of attributes)
                        if sprite.attributes & 0x80 != 0 {
                            row = if row <= 7 { 7 - row } else { 0 };
                        }
                    
                        // Sprite pattern table from PPUCTRL bit 3
                        let sprite_table = if self.ctrl & 0x08 == 0 { 0 } else { 1 };
                    
                        // Fetch pixel from pattern table
                        let pixel = self.get_tile_pixel(
                            sprite.tile_index as u16,
                            sprite_table,
                            col as u8,
                            row as u8,
                        );
                    
                        // Non-transparent pixel found — this sprite wins (priority)
                        if pixel != 0 {
                            sprite_pixel = pixel;
                            sprite_palette = sprite.attributes & 0x03;
                            // Priority: bit 5 = 0 means in front, 1 means behind
                            sprite_priority_front = sprite.attributes & 0x20 == 0;
                            
                            // Check if this is the original OAM sprite 0
                            // Note: buffer_index 0 doesn't guarantee original index 0
                            // We need to track the original OAM index. For SMB, this works
                            // because sprite 0 is usually among the first 8.
                            // A more accurate approach would store the original index.
                            if buffer_index == 0 {
                                sprite_0_hit = true;
                            }
                            
                            break;  // First matching sprite has highest priority
                        }
                    }
                }
            }

            if self.scanline >= 240 {
                // No sprite 0 hit during VBlank or pre-render
                sprite_0_hit = false;
            }
        
            // ----------------------------------------
            // PIXEL MIXING & SPRITE 0 HIT
            // ----------------------------------------
            let mut final_color_index = bg_color_index;
        
            if sprite_pixel != 0 {
                // Sprite 0 hit: occurs when sprite 0's non-transparent pixel overlaps
                // with a non-transparent background pixel, and x >= 8 (left clip)
                if sprite_0_hit && bg_pixel != 0 && screen_x >= 8 {
                    self.status |= 0x40;
                }
            
                // Determine final pixel: sprite wins if it's in front OR bg is transparent
                if sprite_priority_front || bg_pixel == 0 {
                    // Sprite palettes are at $3F10-$3F1F
                    let sprite_color_addr = 0x3F10 + (sprite_palette as u16) * 4 + (sprite_pixel as u16);
                    final_color_index = self.read_vram(sprite_color_addr);
                }
            }
        
            // Write final color to framebuffer
            let (r, g, b) = NES_PALETTE[final_color_index as usize];
            self.framebuffer[idx] = r;
            self.framebuffer[idx + 1] = g;
            self.framebuffer[idx + 2] = b;
        }
    
        // ============================================
        // CYCLE / SCANLINE ADVANCE
        // ============================================
        self.cycle += 1;
        if self.cycle > 340 {
            self.cycle = 0;
            self.scanline += 1;
            if self.scanline > 261 {
                self.scanline = 0;
                self.frames_counter += 1;
            }
        } 
    }

    fn get_tile_pixel(&self, tile_index: u16, ctrl_base_address: u16, pixel_x: u8, pixel_y: u8) -> u8 {
        // Calculate offset in cartridge_chr
        let table_offset = (ctrl_base_address as usize) * 4096;
        let tile_start = table_offset + (tile_index as usize) * 16;

        // Read low and high bitplanes for this row
        let byte_low = self.cartridge_chr[tile_start + (pixel_y as usize)];
        let byte_high = self.cartridge_chr[tile_start + (pixel_y as usize) + 8];

        // Extract the bit for this pixel column
        let shift = 7 - pixel_x;
        let pixel_low = (byte_low >> shift) & 0x01;
        let pixel_high = (byte_high >> shift) & 0x01;

        // Combine into 2-bit color (0-3)
        ((pixel_high << 1) | pixel_low) as u8
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
                    let buffered = self.ppu_data_latch;
                    self.ppu_data_latch = self.read_vram(self.v);
                    buffered
                } else {
                    self.ppu_data_latch = self.read_vram(self.v - 0x2FFF);
                    self.read_vram(self.v)
                };

                // Increment v (check bit 2 of ctrl register : 0 = +1, 1 = +32)
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
            let idx = address as usize;
            if idx < self.cartridge_chr.len() {
                self.cartridge_chr[idx]
            } else {
                0 
            }
        } else if address < 0x3F00 {
            // Nametables ($2000-$3EFF, avec mirror $3000-$3EFF)
            let mirrored = self.apply_mirroring(address);
            self.vram[(mirrored - 0x2000) as usize]
        } else {
            // Palettes ($3F00-$3FFF)
            let mut index = address & 0x1F;
            
            // Mirroring of "color 0" => background
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
            self.vram[(mirrored - 0x2000) as usize] = value;
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
            Mirroring::FourScreen => table,               // ABCD (Need RAM Cartridge)
            _ => todo!("Unsupported Mirroring {}", self.mirroring)
        };

        0x2000 | (mapped_table << 10) | offset
    }
}