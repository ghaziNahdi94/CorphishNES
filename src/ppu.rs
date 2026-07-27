use crate::cartridge::Mirroring;

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
    oam_index: u8,
    position_x: u8,
    position_y: u8,
    attributes: u8,
    tile_index: u8,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Ppu {
    vram: [u8; 4096],
    palette: [u8; 32],
    pub oam: [u8; 256],
    v: u16,
    t: u16,
    x: u8,
    w: bool,
    ctrl: u8,
    mask: u8,
    status: u8,
    pub oam_address: u8,
    ppu_data_latch: u8,
    cartridge_chr: Vec<u8>,
    mirroring: Mirroring,
    scanline: u16,
    cycle: u16,
    frames_counter: u64,
    pub nmi_pending: bool,
    sprites_buffer: Vec<Sprite>,
    pub framebuffer: [u8; FRAME_WIDTH * FRAME_HEIGHT * 3],
    v_start_of_scanline: u16,
    // Pour le sprite 0 hit : le PPU a un délai de pipeline interne
    sprite_0_hit_delay: bool,
}

impl Ppu {
    pub fn new(chr: Vec<u8>, mirroring: Mirroring) -> Self {
        let cartridge_chr = if chr.is_empty() {
            vec![0u8; 8192]
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
            cartridge_chr,
            mirroring,
            scanline: 261,
            cycle: 0,
            nmi_pending: false,
            sprites_buffer: Vec::new(),
            frames_counter: 0,
            framebuffer: [0; FRAME_WIDTH * FRAME_HEIGHT * 3],
            v_start_of_scanline: 0,
            sprite_0_hit_delay: false,
        }
    }

    pub fn step(&mut self) {
        let rendering_enabled = (self.mask & 0x18) != 0;

        // ============================================
        // VBLANK / STATUS FLAGS (cycle 1)
        // ============================================
        if self.cycle == 1 {
            match self.scanline {
                241 => {
                    self.status |= 0x80;
                    if self.ctrl & 0x80 != 0 {
                        self.nmi_pending = true;
                    }
                }
                261 => {
                    self.status &= !0x80;
                    self.status &= !0x40;
                    self.status &= !0x20;
                    self.sprite_0_hit_delay = false;
                }
                _ => {}
            }
        }

        // ============================================
        // SAUVEGARDE DE v AU DÉBUT DE LA SCANLINE
        // ============================================
        if self.cycle == 0 && (self.scanline < 240 || self.scanline == 261) {
            self.v_start_of_scanline = self.v;
        }

        // ============================================
        // SCROLLING & VRAM ADDRESS UPDATES
        // ============================================
        if rendering_enabled {
            // Incrément horizontal de coarse X tous les 8 cycles
            if self.scanline < 240 || self.scanline == 261 {
                if (self.cycle > 0 && self.cycle <= 256 && self.cycle % 8 == 0) ||
                   (self.cycle >= 321 && self.cycle <= 336 && self.cycle % 8 == 0) {
                    if (self.v & 0x001F) == 31 {
                        self.v &= !0x001F;
                        self.v ^= 0x0400;
                    } else {
                        self.v += 1;
                    }
                }
            }

            // Vertical increment à cycle 256
            if (self.scanline < 240 || self.scanline == 261) && self.cycle == 256 {
                let fine_y = (self.v >> 12) & 0x07;
                if fine_y < 7 {
                    self.v += 0x1000;
                } else {
                    self.v &= !0x7000;
                    let coarse_y = (self.v >> 5) & 0x1F;
                    if coarse_y == 29 {
                        self.v &= !0x03E0;
                        self.v ^= 0x0800;
                    } else if coarse_y == 31 {
                        self.v &= !0x03E0;
                    } else {
                        self.v += 0x0020;
                    }
                }
            }

            // Horizontal copy from t to v (cycle 257) - inclure scanline 261
            if (self.scanline < 240 || self.scanline == 261) && self.cycle == 257 {
                self.v = (self.v & !0x041F) | (self.t & 0x041F);
            }

            // Vertical copy from t to v (cycles 280-304 de pre-render)
            if self.scanline == 261 && self.cycle >= 280 && self.cycle <= 304 {
                self.v = (self.v & !0x7BE0) | (self.t & 0x7BE0);
            }
        }

        // ============================================
        // SPRITE EVALUATION (cycle 257)
        // ============================================
        if rendering_enabled && self.scanline < 240 && self.cycle == 257 {
            self.sprites_buffer.clear();
            let mut overflow = false;

            for i in 0..64 {
                let sprite_y = self.oam[i * 4];
                
                if sprite_y >= 0xEF {
                    continue;
                }

                let y_start = (sprite_y as u16).wrapping_add(1);
                let y_end = y_start.wrapping_add(8);
                
                if self.scanline >= y_start && self.scanline < y_end {
                    if self.sprites_buffer.len() < 8 {
                        self.sprites_buffer.push(Sprite {
                            oam_index: i as u8,
                            position_x: self.oam[i * 4 + 3],
                            position_y: sprite_y,
                            tile_index: self.oam[i * 4 + 1],
                            attributes: self.oam[i * 4 + 2],
                        });
                    } else {
                        overflow = true;
                    }
                }
            }

            if overflow {
                self.status |= 0x20;
            }
        }

        // ============================================
        // RENDERING (scanlines 0-239, cycles 1-256)
        // ============================================
        let bg_enabled = self.mask & 0x08 != 0;
        let sprites_enabled = self.mask & 0x10 != 0;

        if self.scanline < (FRAME_HEIGHT as u16) && self.cycle > 0 && self.cycle <= (FRAME_WIDTH as u16) {
            let screen_x = (self.cycle - 1) as usize;
            let screen_y = self.scanline as usize;
            let idx = (screen_y * FRAME_WIDTH + screen_x) * 3;

            // ----------------------------------------
            // BACKGROUND RENDERING - CORRECTION fine_x
            // ----------------------------------------
            let mut bg_pixel: u8 = 0;
            let mut bg_palette: u8 = 0;

            if bg_enabled {
                let fine_x = self.x;
                let fine_y = (self.v_start_of_scanline >> 12) & 0x07;
                let coarse_x = self.v_start_of_scanline & 0x1F;
                let coarse_y = (self.v_start_of_scanline >> 5) & 0x1F;
                let nt = (self.v_start_of_scanline >> 10) & 0x03;

                // CORRECTION : Quand fine_x > 0, le premier tile commence "avant" l'écran
                // On doit soustraire fine_x pour avoir la bonne position de tile
                let effective_screen_x = screen_x as u16 + fine_x as u16;
                let tile_col = coarse_x + (effective_screen_x / 8);
                let pixel_x = (effective_screen_x % 8) as u8;
                let pixel_y = fine_y as u8;

                let mut current_nt = nt;
                let mut final_tile_col = tile_col;
                
                // Wrap-around du nametable horizontal
                if tile_col >= 32 {
                    current_nt ^= 1;
                    final_tile_col = tile_col & 0x1F;
                }

                let nametable_base = 0x2000 | (current_nt << 10);
                let tile_addr = nametable_base + (coarse_y << 5) + final_tile_col;

                let tile_index = self.read_vram(tile_addr) as u16;
                let bg_table = if self.ctrl & 0x10 == 0 { 0 } else { 1 };
                
                bg_pixel = self.get_tile_pixel(tile_index, bg_table, pixel_x, pixel_y);

                let attr_x = final_tile_col >> 2;
                let attr_y = coarse_y >> 2;
                let attr_addr = (nametable_base + 0x03C0) + (attr_y << 3) + attr_x;
                let attr_byte = self.read_vram(attr_addr);
                let shift = ((coarse_y & 0x02) << 1) | (final_tile_col & 0x02);
                bg_palette = (attr_byte >> shift) & 0x03;
            }

            let bg_color_addr = if bg_pixel == 0 {
                0x3F00
            } else {
                0x3F00 + (bg_palette as u16) * 4 + (bg_pixel as u16)
            };
            let bg_color_index = self.read_vram(bg_color_addr);

            // ----------------------------------------
            // SPRITE RENDERING - CORRECTION : ordre inverse
            // ----------------------------------------
            let mut sprite_pixel: u8 = 0;
            let mut sprite_palette: u8 = 0;
            let mut sprite_priority_front = true;
            let mut sprite_0_hit_this_pixel = false;

            if sprites_enabled {
                // CORRECTION : Sur NES, les sprites sont rendus de l'index 63 vers 0
                // Le sprite avec l'index le plus BAS apparaît par-dessus
                // Donc on parcourt le buffer à l'envers pour le rendu
                for sprite in self.sprites_buffer.iter().rev() {
                    let sprite_x = sprite.position_x as usize;

                    if screen_x >= sprite_x && screen_x < sprite_x + 8 {
                        let mut col = screen_x - sprite_x;
                        let mut row = screen_y.wrapping_sub((sprite.position_y as usize).wrapping_add(1));

                        if col < 8 && row < 8 {
                            if sprite.attributes & 0x40 != 0 { col = 7 - col; }
                            if sprite.attributes & 0x80 != 0 { row = 7 - row; }

                            let sprite_table = if self.ctrl & 0x08 == 0 { 0 } else { 1 };
                            let pixel = self.get_tile_pixel(sprite.tile_index as u16, sprite_table, col as u8, row as u8);

                            if pixel != 0 {
                                sprite_pixel = pixel;
                                sprite_palette = sprite.attributes & 0x03;
                                sprite_priority_front = sprite.attributes & 0x20 == 0;

                                if sprite.oam_index == 0 {
                                    sprite_0_hit_this_pixel = true;
                                }
                                // On ne fait PAS break ici car on veut que le sprite 0 
                                // (qui est en premier dans le buffer) soit prioritaire
                            }
                        }
                    }
                }
            }

            // ----------------------------------------
            // PIXEL MIXING & SPRITE 0 HIT
            // ----------------------------------------
            let mut final_color_index = bg_color_index;

            if sprite_pixel != 0 {
                // CORRECTION : Sprite 0 hit avec timing précis
                // Le hit ne se produit que si le sprite 0 est VRAIMENT visible
                // (pas masqué par un autre sprite avec priorité plus haute)
                if sprite_0_hit_this_pixel && bg_pixel != 0 {
                    let left_clip_bg = (self.mask & 0x02) != 0;
                    let left_clip_spr = (self.mask & 0x04) != 0;
                    
                    let in_left_clip = screen_x < 8;
                    let clipped = in_left_clip && (left_clip_bg || left_clip_spr);
                    
                    // Le sprite 0 hit ne se produit PAS au pixel 255
                    // et il y a un léger délai de pipeline (on le décale de 1 pixel)
                    if screen_x != 255 && !clipped && !self.sprite_0_hit_delay {
                        // Décalage de 1 pixel pour le pipeline du PPU
                        if screen_x >= 1 {
                            self.status |= 0x40;
                            self.sprite_0_hit_delay = true;
                        }
                    }
                }

                if sprite_priority_front || bg_pixel == 0 {
                    let sprite_color_addr = 0x3F10 + (sprite_palette as u16) * 4 + (sprite_pixel as u16);
                    final_color_index = self.read_vram(sprite_color_addr);
                }
            }

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
        let table_offset = (ctrl_base_address as usize) * 4096;
        let tile_start = table_offset + (tile_index as usize) * 16;

        let byte_low = self.cartridge_chr[tile_start + (pixel_y as usize)];
        let byte_high = self.cartridge_chr[tile_start + (pixel_y as usize) + 8];

        let shift = 7 - pixel_x;
        let pixel_low = (byte_low >> shift) & 0x01;
        let pixel_high = (byte_high >> shift) & 0x01;

        ((pixel_high << 1) | pixel_low) as u8
    }

    pub fn read_cpu(&mut self, address: u16) -> u8 {
        match address {
            0x2002 => {
                let val = self.status;
                self.status &= !0x80;
                self.w = false;
                val
            }
            0x2004 => self.oam[self.oam_address as usize],
            0x2007 => {
                let val = if self.v < 0x3F00 {
                    let buffered = self.ppu_data_latch;
                    self.ppu_data_latch = self.read_vram(self.v);
                    buffered
                } else {
                    let palette_val = self.read_vram(self.v);
                    self.ppu_data_latch = self.read_vram(self.v - 0x1000);
                    palette_val
                };
                self.v = self.v.wrapping_add(if self.ctrl & 0x04 == 0 { 1 } else { 32 });
                val
            }
            _ => 0,
        }
    }

    pub fn write_cpu(&mut self, address: u16, value: u8) {
        match address {
            0x2000 => {
                self.ctrl = value;
                self.t = (self.t & !0x0C00) | (((value as u16) & 0x03) << 10);
            }
            0x2001 => self.mask = value,
            0x2003 => self.oam_address = value,
            0x2004 => {
                self.oam[self.oam_address as usize] = value;
                self.oam_address = self.oam_address.wrapping_add(1);
            }
            0x2005 => {
                if !self.w {
                    self.x = value & 0x07;
                    self.t = (self.t & 0xFFE0) | ((value as u16) >> 3);
                    self.w = true;
                } else {
                    self.t = (self.t & 0x8C1F)
                        | (((value as u16) & 0x07) << 12)
                        | (((value as u16) & 0xF8) << 2);
                    self.w = false;
                }
            }
            0x2006 => {
                if !self.w {
                    self.t = (self.t & 0x00FF) | (((value as u16) & 0x3F) << 8);
                    self.w = true;
                } else {
                    self.t = (self.t & 0xFF00) | (value as u16);
                    self.v = self.t;
                    self.w = false;
                }
            }
            0x2007 => {
                self.write_vram(self.v, value);
                self.v = self.v.wrapping_add(if self.ctrl & 0x04 == 0 { 1 } else { 32 });
            }
            _ => {}
        }
    }

    fn read_vram(&self, address: u16) -> u8 {
        let address = address & 0x3FFF;
        if address < 0x2000 {
            let idx = address as usize;
            if idx < self.cartridge_chr.len() {
                self.cartridge_chr[idx]
            } else {
                0
            }
        } else if address < 0x3F00 {
            let mirrored = self.apply_mirroring(address);
            self.vram[(mirrored - 0x2000) as usize]
        } else {
            let mut index = address & 0x1F;
            if index >= 0x10 && (index & 0x03) == 0 {
                index -= 0x10;
            }
            self.palette[index as usize]
        }
    }

    fn write_vram(&mut self, address: u16, value: u8) {
        let address = address & 0x3FFF;
        if address < 0x2000 {
            if self.cartridge_chr.len() > address as usize {
                self.cartridge_chr[address as usize] = value;
            }
        } else if address < 0x3F00 {
            let mirrored = self.apply_mirroring(address);
            self.vram[(mirrored - 0x2000) as usize] = value;
        } else {
            let mut index = address & 0x1F;
            if index >= 0x10 && (index & 0x03) == 0 {
                index -= 0x10;
            }
            self.palette[index as usize] = value;
        }
    }

    fn apply_mirroring(&self, address: u16) -> u16 {
        let offset = address & 0x03FF;
        let table = (address >> 10) & 0x03;
        let mapped_table = match self.mirroring {
            Mirroring::Vertical => table & 0x01,
            Mirroring::Horizontal => (table & 0x02) >> 1,
            Mirroring::FourScreen => table,
            _ => 0,
        };
        0x2000 | (mapped_table << 10) | offset
    }
}