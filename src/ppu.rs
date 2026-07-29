use crate::cartridge::Mirroring;

// =============================================================================
// NES COLOR PALETTE
// =============================================================================
// The NES can only show 64 different colors total. Each color is stored as
// (red, green, blue) values. The PPU uses color INDEXES (0-63) and we look
// up the actual RGB values from this table.
const NES_COLOR_PALETTE: [(u8, u8, u8); 64] = [
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

// =============================================================================
// SCREEN SIZE
// =============================================================================
// The NES screen is always 256 pixels wide and 240 pixels tall.
// Each pixel needs 3 bytes in memory: one for red, one for green, one for blue.
const SCREEN_WIDTH_IN_PIXELS: usize = 256;
const SCREEN_HEIGHT_IN_PIXELS: usize = 240;
const BYTES_PER_PIXEL: usize = 3;
const TOTAL_FRAMEBUFFER_BYTES: usize = SCREEN_WIDTH_IN_PIXELS * SCREEN_HEIGHT_IN_PIXELS * BYTES_PER_PIXEL;

// =============================================================================
// REGISTER BIT DEFINITIONS
// =============================================================================
// These constants give names to the individual bits inside the PPU registers.
// Using names instead of raw hex numbers makes the code much easier to read.

// Control Register ($2000) bit flags
const CONTROL_FLAG_NMI_ENABLE: u8 = 0x80;               // If set, the PPU tells the CPU when vertical blank starts
const CONTROL_FLAG_SPRITE_SIZE: u8 = 0x20;              // If set, sprites are 16 pixels tall instead of 8
const CONTROL_FLAG_BACKGROUND_PATTERN_TABLE: u8 = 0x10; // If set, background uses the second pattern table ($1000)
const CONTROL_FLAG_SPRITE_PATTERN_TABLE: u8 = 0x08;     // If set, sprites use the second pattern table ($1000)
const CONTROL_FLAG_VRAM_INCREMENT_32: u8 = 0x04;        // If set, reading/writing $2007 moves the address by 32 instead of 1
const CONTROL_FLAG_NAMETABLE_SELECT: u8 = 0x03;       // Bits 0-1: which name table to use at power-on (0, 1, 2, or 3)

// Mask Register ($2001) bit flags
const MASK_FLAG_SPRITES_ENABLED: u8 = 0x10;             // If set, sprites are drawn on screen
const MASK_FLAG_BACKGROUND_ENABLED: u8 = 0x08;          // If set, the background is drawn on screen
const MASK_FLAG_LEFT_CLIP_SPRITES: u8 = 0x04;           // If set, sprites are hidden in the left 8 pixels
const MASK_FLAG_LEFT_CLIP_BACKGROUND: u8 = 0x02;        // If set, background is hidden in the left 8 pixels

// Status Register ($2002) bit flags
const STATUS_FLAG_VERTICAL_BLANK: u8 = 0x80;          // Set when the PPU is in the blanking period (not drawing)
const STATUS_FLAG_SPRITE_ZERO_HIT: u8 = 0x40;           // Set when sprite 0 overlaps the background
const STATUS_FLAG_SPRITE_OVERFLOW: u8 = 0x20;           // Set when more than 8 sprites are on one scanline

// =============================================================================
// SPRITE (OAM ENTRY)
// =============================================================================
// The NES can display up to 64 sprites (moving objects) on screen.
// Each sprite is described by 4 bytes in a special memory area called OAM.
// This struct holds one sprite in a friendly format.
#[derive(Clone, Debug, PartialEq)]
struct Sprite {
    // Which slot this sprite occupies in OAM memory (0 to 63).
    // Sprite 0 is special because it can trigger the "sprite 0 hit" flag.
    index_in_oam: u8,

    // Horizontal position on screen (0-255). This is the X coordinate.
    x_position_on_screen: u8,

    // Vertical position on screen (0-255). This is the Y coordinate MINUS 1.
    // A value of 0 means the sprite starts at Y = 1 on screen.
    y_position_on_screen: u8,

    // A byte full of settings:
    //   bit 7: flip the sprite vertically
    //   bit 6: flip the sprite horizontally
    //   bit 5: draw the sprite BEHIND the background (instead of in front)
    //   bits 1-0: which palette to use for this sprite (0-3)
    attribute_byte: u8,

    // Which tile (graphics shape) to draw from the pattern tables.
    // Think of this like choosing which letter from a font to display.
    tile_number: u8,
}

// =============================================================================
// THE PPU (Picture Processing Unit)
// =============================================================================
// This is the graphics chip of the NES. Its job is to draw the screen,
// one pixel at a time, 60 times per second.
#[derive(Clone, Debug, PartialEq)]
pub struct Ppu {
    // -------------------------------------------------------------------------
    // VIDEO MEMORY
    // -------------------------------------------------------------------------
    // Name tables are like "maps" that tell the PPU which tiles to draw
    // for the background. There are 4 name tables, each 1KB, totaling 4KB.
    // The NES only has 2KB of actual RAM for this, so two of the tables
    // are "mirrors" (copies) of the other two, depending on the cartridge.
    name_table_vram: [u8; 4096],

    // The palette RAM stores which colors to use. It holds 32 bytes.
    // These are not actual colors, but INDEXES into the NES_COLOR_PALETTE.
    palette_ram: [u8; 32],

    // -------------------------------------------------------------------------
    // SPRITE MEMORY (OAM)
    // -------------------------------------------------------------------------
    // OAM stands for Object Attribute Memory. It holds 64 sprites.
    // Each sprite uses 4 bytes, so 64 * 4 = 256 bytes total.
    // The CPU writes to this memory to place sprites on screen.
    pub sprite_oam: [u8; 256],

    // -------------------------------------------------------------------------
    // INTERNAL SCROLLING AND ADDRESS REGISTERS
    // -------------------------------------------------------------------------
    // The NES PPU has two internal 15-bit address registers called "v" and "t".
    // They are tricky to understand at first, but here is the simple version:
    //
    //   current_vram_address (v):  This is the address the PPU is actively
    //                               reading from RIGHT NOW while drawing.
    //
    //   temporary_vram_address (t): This is like a "draft" address. The CPU
    //                               writes scrolling info here first, and then
    //                               the PPU copies it to "v" at specific times.
    //
    // Think of "t" as writing a rough draft, and "v" as the final published copy.

    // The live address the PPU is fetching graphics from during rendering.
    current_vram_address: u16,

    // The temporary address where the CPU writes new scrolling positions.
    temporary_vram_address: u16,

    // Fine X scroll: how many pixels to shift the background horizontally.
    // This can be 0 to 7. It creates smooth scrolling within a single tile.
    fine_x_scroll: u8,

    // The "write toggle" or "address latch". The NES only has 5 CPU registers,
    // but some operations need 2 bytes of data. This flag remembers whether
    // we are writing the FIRST byte or the SECOND byte.
    //   false = first write (waiting for the first byte)
    //   true  = second write (waiting for the second byte)
    address_latch_second_write: bool,

    // -------------------------------------------------------------------------
    // CPU-ACCESSIBLE REGISTERS
    // -------------------------------------------------------------------------
    // The CPU talks to the PPU through 8 memory addresses ($2000-$2007).
    // These variables store the values written to those addresses.

    // $2000 - Control Register: tells the PPU basic settings like
    //         "send interrupts to the CPU" and "which pattern table to use".
    control_register: u8,

    // $2001 - Mask Register: turns rendering on/off and controls clipping.
    mask_register: u8,

    // $2002 - Status Register: the CPU reads this to know what the PPU is doing.
    //         For example, "are we in vertical blank?" or "did sprite 0 hit?"
    status_register: u8,

    // $2003 - OAM Address Register: tells which byte of OAM the CPU wants to access.
    pub oam_address_register: u8,

    // -------------------------------------------------------------------------
    // DATA BUFFERING
    // -------------------------------------------------------------------------
    // The PPU is a bit slow. When the CPU reads from video memory ($2007),
    // the PPU gives the OLD value it read previously, and stores the new value
    // for the NEXT read. This buffer holds that "old" value.
    vram_read_buffer: u8,

    // -------------------------------------------------------------------------
    // CARTRIDGE DATA
    // -------------------------------------------------------------------------
    // The CHR data contains the actual graphics tiles (shapes) of the game.
    // Each tile is 16 bytes. There are usually 512 tiles total (8KB).
    // This data comes directly from the game cartridge.
    cartridge_chr_data: Vec<u8>,

    // Mirroring tells us how the 4 name tables are arranged in memory.
    // Games use this to create scrolling effects.
    screen_mirroring: Mirroring,

    // -------------------------------------------------------------------------
    // TIMING
    // -------------------------------------------------------------------------
    // The PPU draws the screen line by line, like an old CRT television.
    // There are 262 horizontal lines (scanlines) per frame, numbered 0-261.
    // Each scanline takes 341 clock cycles (pixels) to draw.

    // Which scanline we are currently processing (0 to 261).
    //   0-239   = visible scanlines (the actual picture)
    //   240     = idle scanline (nothing happens)
    //   241-260 = vertical blanking (VBLANK) - the PPU rests here
    //   261     = pre-render scanline (prepares for the next frame)
    current_scanline: u16,

    // Which clock cycle (pixel) we are on within the current scanline (0 to 340).
    // Cycle 0 is usually idle. Cycles 1-256 draw the visible pixels.
    current_cycle: u16,

    // Counts how many complete frames (full screens) have been drawn.
    frame_count: u64,

    // -------------------------------------------------------------------------
    // INTERRUPTS
    // -------------------------------------------------------------------------
    // When vertical blank starts, the PPU can send an interrupt signal (NMI)
    // to the CPU. This flag tells the rest of the emulator that an NMI
    // should be triggered.
    pub nmi_interrupt_pending: bool,

    // -------------------------------------------------------------------------
    // RENDERING HELPERS
    // -------------------------------------------------------------------------
    // During each visible scanline, the PPU figures out which sprites appear
    // on that line. This list holds up to 8 sprites for the current scanline.
    visible_sprites_this_scanline: Vec<Sprite>,

    // The final picture! This is a flat array of RGB values.
    // It gets filled pixel by pixel and then displayed on screen.
    pub screen_pixels: [u8; TOTAL_FRAMEBUFFER_BYTES],

    // The PPU copies "current_vram_address" to this variable at a specific
    // moment during rendering. We use this SNAPSHOT to draw the scanline,
    // so that mid-scanline address changes do not mess up the picture.
    render_vram_address_copy: u16,

    // Sprite 0 hit detection is delayed by 1 pixel in this emulator.
    // This variable stores the X position where the hit was DETECTED,
    // and the flag is actually set one cycle later.
    sprite_zero_hit_detected_at_x: Option<u16>,
}

impl Ppu {
    // =========================================================================
    // CONSTRUCTOR
    // =========================================================================
    // Creates a new PPU with the graphics data (CHR) from the cartridge.
    // If the cartridge has no CHR data (some early games stored graphics
    // in program memory), we create an empty 8KB buffer.
    pub fn new(cartridge_chr_data: Vec<u8>, screen_mirroring: Mirroring) -> Self {
        let chr_data = if cartridge_chr_data.is_empty() {
            // No graphics data provided; allocate a blank 8KB pattern table.
            vec![0u8; 8192]
        } else {
            cartridge_chr_data
        };

        Self {
            name_table_vram: [0; 4096],
            palette_ram: [0; 32],
            sprite_oam: [0; 256],
            current_vram_address: 0,
            temporary_vram_address: 0,
            fine_x_scroll: 0,
            address_latch_second_write: false,
            control_register: 0,
            mask_register: 0,
            status_register: 0,
            oam_address_register: 0,
            vram_read_buffer: 0,
            cartridge_chr_data: chr_data,
            screen_mirroring,
            current_scanline: 261,          // Start at the pre-render scanline
            current_cycle: 0,
            nmi_interrupt_pending: false,
            visible_sprites_this_scanline: Vec::new(),
            frame_count: 0,
            screen_pixels: [0; TOTAL_FRAMEBUFFER_BYTES],
            render_vram_address_copy: 0,
            sprite_zero_hit_detected_at_x: None,
        }
    }

    fn vram_address_increment(&self) -> u16 {
        if self.control_register & CONTROL_FLAG_VRAM_INCREMENT_32 == 0 {
            1
        } else {
            32
        }
    }

    // =========================================================================
    // MAIN STEP FUNCTION
    // =========================================================================
    // This function runs once per clock cycle (about 5.4 million times per second).
    // It is the "heartbeat" of the PPU. On each call, it:
    //   1. Updates status flags (VBLANK, etc.)
    //   2. Updates scrolling registers
    //   3. Evaluates which sprites are visible
    //   4. Draws one pixel (if we are on a visible part of the screen)
    //   5. Moves to the next cycle
    pub fn step(&mut self) {
        // Check if the screen is actually being drawn right now.
        let rendering_is_turned_on = self.is_rendering_enabled();

        // -------------------------------------------------------------------
        // STEP 1: UPDATE STATUS FLAGS ON CYCLE 1
        // -------------------------------------------------------------------
        // On the very first cycle of certain scanlines, the PPU flips
        // status flags to tell the CPU what is happening.
        if self.current_cycle == 1 {
            self.update_status_flags_at_cycle_one();
        }

        // -------------------------------------------------------------------
        // STEP 2: SAVE THE VRAM ADDRESS FOR RENDERING
        // -------------------------------------------------------------------
        // At cycle 258 of every rendering scanline, the PPU has just finished
        // updating its internal address. We take a "snapshot" of that address
        // so we can use it to draw the entire scanline consistently.
        let is_rendering_scanline = self.current_scanline < 240 || self.current_scanline == 261;
        if is_rendering_scanline && self.current_cycle == 258 {
            self.render_vram_address_copy = self.current_vram_address;
        }

        // -------------------------------------------------------------------
        // STEP 3: UPDATE SCROLLING AND MEMORY ADDRESS REGISTERS
        // -------------------------------------------------------------------
        // The PPU automatically moves its internal memory pointer as it draws,
        // fetching the next tiles and shifting to the next row. This only
        // happens when rendering is enabled.
        if rendering_is_turned_on {
            self.update_scroll_registers_automatically();
        }

        // -------------------------------------------------------------------
        // STEP 4: FIND WHICH SPRITES ARE VISIBLE ON THIS SCANLINE
        // -------------------------------------------------------------------
        // At cycle 257, the PPU looks through all 64 sprites and checks
        // which ones overlap the current scanline. It can only remember 8.
        if rendering_is_turned_on && self.current_scanline < 240 && self.current_cycle == 257 {
            self.find_visible_sprites_for_this_scanline();
        }

        // -------------------------------------------------------------------
        // STEP 5: DRAW ONE PIXEL
        // -------------------------------------------------------------------
        // If we are on a visible scanline (0-239) and a visible cycle (1-256),
        // we calculate the color of one pixel and write it to the screen buffer.
        let on_visible_scanline = self.current_scanline < SCREEN_HEIGHT_IN_PIXELS as u16;
        let on_visible_cycle = self.current_cycle > 0 && self.current_cycle <= SCREEN_WIDTH_IN_PIXELS as u16;

        if on_visible_scanline && on_visible_cycle {
            self.draw_one_pixel();
        }

        // -------------------------------------------------------------------
        // STEP 6: ADVANCE TO THE NEXT CYCLE
        // -------------------------------------------------------------------
        self.advance_clock();
    }

    // =========================================================================
    // RENDERING STATE CHECKS
    // =========================================================================
    // Returns true if either the background OR sprites are being drawn.
    // When this is false, the PPU is basically idle and does not update
    // its internal scrolling registers automatically.
    fn is_rendering_enabled(&self) -> bool {
        let background_turned_on = self.mask_register & MASK_FLAG_BACKGROUND_ENABLED != 0;
        let sprites_turned_on = self.mask_register & MASK_FLAG_SPRITES_ENABLED != 0;
        background_turned_on || sprites_turned_on
    }

    // Returns true only if the background layer is being drawn.
    fn is_background_enabled(&self) -> bool {
        self.mask_register & MASK_FLAG_BACKGROUND_ENABLED != 0
    }

    // Returns true only if the sprite layer is being drawn.
    fn is_sprites_enabled(&self) -> bool {
        self.mask_register & MASK_FLAG_SPRITES_ENABLED != 0
    }

    // =========================================================================
    // STATUS FLAG UPDATES (CYCLE 1)
    // =========================================================================
    // The PPU has three important status flags in register $2002:
    //   - Vertical Blank (bit 7): set during the resting period between frames
    //   - Sprite 0 Hit (bit 6): set when the first sprite touches the background
    //   - Sprite Overflow (bit 5): set when >8 sprites are on one line
    //
    // These flags are updated on cycle 1 of specific scanlines.
    fn update_status_flags_at_cycle_one(&mut self) {
        match self.current_scanline {
            // Scanline 241, cycle 1: VBLANK begins!
            // The picture is finished. The PPU rests for 20 scanlines.
            // If the game asked for NMI interrupts, we signal the CPU now.
            241 => {
                self.status_register |= STATUS_FLAG_VERTICAL_BLANK;
                if self.control_register & CONTROL_FLAG_NMI_ENABLE != 0 {
                    self.nmi_interrupt_pending = true;
                }
            }

            // Scanline 261, cycle 1: Pre-render scanline begins!
            // Clear the VBLANK flag, the sprite 0 hit flag, and the overflow flag.
            // We are getting ready to draw the next frame.
            261 => {
                self.status_register &= !STATUS_FLAG_VERTICAL_BLANK;
                self.status_register &= !STATUS_FLAG_SPRITE_ZERO_HIT;
                self.status_register &= !STATUS_FLAG_SPRITE_OVERFLOW;
            }

            // On all other scanlines, nothing special happens on cycle 1.
            _ => {}
        }
    }

    // =========================================================================
    // AUTOMATIC SCROLL REGISTER UPDATES
    // =========================================================================
    // While the PPU is drawing, it automatically moves from one tile to the
    // next. This is how horizontal and vertical scrolling work! The PPU does
    // NOT have special "scroll counters"; instead, it simply increments the
    // memory address it is reading from.
    fn update_scroll_registers_automatically(&mut self) {
        // Only run this logic on scanlines where the PPU is actively fetching
        // graphics data (visible scanlines 0-239, and the pre-render scanline 261).
        let active_scanline = self.current_scanline < 240 || self.current_scanline == 261;
        if !active_scanline {
            return;
        }

        // -------------------------------------------------------------------
        // HORIZONTAL MOVEMENT: move to the next tile every 8 cycles
        // -------------------------------------------------------------------
        // The PPU fetches one full tile every 8 clock cycles.
        // This happens during the visible part of the scanline (cycles 1-256)
        // and again during the hidden fetch period (cycles 321-336).
        let in_visible_fetch = self.current_cycle > 0 && self.current_cycle <= 256 && self.current_cycle % 8 == 0;
        let in_hidden_fetch = self.current_cycle >= 321 && self.current_cycle <= 336 && self.current_cycle % 8 == 0;

        if in_visible_fetch || in_hidden_fetch {
            self.increment_horizontal_tile_position();
        }

        // -------------------------------------------------------------------
        // VERTICAL MOVEMENT: move to the next row of tiles at cycle 256
        // -------------------------------------------------------------------
        // At the end of each scanline, the PPU advances to the next row.
        // It first tries to add 1 to the "fine Y" scroll (the sub-tile row).
        // If fine Y was already 7 (the last row of the tile), it resets to 0
        // and moves to the next tile row (coarse Y).
        if self.current_cycle == 256 {
            self.increment_vertical_tile_position();
        }

        // -------------------------------------------------------------------
        // HORIZONTAL COPY: reset X position at cycle 257
        // -------------------------------------------------------------------
        // At the very end of the visible scanline, the PPU copies the
        // horizontal scroll bits from the temporary address (t) back into
        // the live address (v). This is why changing the horizontal scroll
        // mid-frame works the way it does on the NES.
        if self.current_cycle == 257 {
            self.copy_horizontal_scroll_from_temp_to_live();
        }

        // -------------------------------------------------------------------
        // VERTICAL COPY: reset Y position during pre-render scanline
        // -------------------------------------------------------------------
        // During cycles 280-304 of the pre-render scanline (scanline 261),
        // the PPU copies the VERTICAL scroll bits from (t) to (v).
        // This prepares the live address for the very first visible scanline.
        let in_vertical_copy_period = self.current_scanline == 261
            && self.current_cycle >= 280
            && self.current_cycle <= 304;

        if in_vertical_copy_period {
            self.copy_vertical_scroll_from_temp_to_live();
        }
    }

    // Moves one tile to the right. If we go past the last tile of the name
    // table, we wrap around to the next name table horizontally.
    fn increment_horizontal_tile_position(&mut self) {
        // The coarse X position is stored in the lowest 5 bits of the address.
        // There are 32 tiles horizontally in a name table.
        let coarse_x = self.current_vram_address & 0x001F;

        if coarse_x == 31 {
            // We were at the last tile. Wrap back to 0 and flip to the
            // other name table horizontally (toggle bit 10).
            self.current_vram_address &= !0x001F; // Clear coarse X back to 0
            self.current_vram_address ^= 0x0400;   // Toggle horizontal name table
        } else {
            // Simply move to the next tile.
            self.current_vram_address += 1;
        }
    }

    // Moves one pixel row down. Handles wrapping from fine Y to coarse Y,
    // and wrapping from the bottom of the name table back to the top.
    fn increment_vertical_tile_position(&mut self) {
        // "Fine Y" is the sub-row inside an 8x8 tile. It lives in bits 12-14.
        let fine_y = (self.current_vram_address >> 12) & 0x07;

        if fine_y < 7 {
            // We are not at the bottom of the tile yet. Just add 1 to fine Y.
            self.current_vram_address += 0x1000;
        } else {
            // We finished this tile. Reset fine Y to 0 and move to the next tile row.
            self.current_vram_address &= !0x7000; // Clear fine Y

            // "Coarse Y" is the tile row within the name table. It lives in bits 5-9.
            let coarse_y = (self.current_vram_address >> 5) & 0x1F;

            if coarse_y == 29 {
                // We reached the bottom of the name table. Wrap to the top
                // and flip to the other name table vertically (toggle bit 11).
                self.current_vram_address &= !0x03E0; // Clear coarse Y
                self.current_vram_address ^= 0x0800;   // Toggle vertical name table
            } else if coarse_y == 31 {
                // Some games use this as a "negative" row. Just wrap without flipping.
                self.current_vram_address &= !0x03E0;
            } else {
                // Simply move to the next tile row.
                self.current_vram_address += 0x0020;
            }
        }
    }

    // Copies the horizontal scroll bits (coarse X and name table X)
    // from the temporary address to the live address.
    fn copy_horizontal_scroll_from_temp_to_live(&mut self) {
        // Bits 0-4 (coarse X) and bit 10 (name table X) are copied.
        // We keep everything else from the live address.
        let horizontal_bits_from_temp = self.temporary_vram_address & 0x041F;
        let preserved_vertical_bits = self.current_vram_address & !0x041F;
        self.current_vram_address = preserved_vertical_bits | horizontal_bits_from_temp;
    }

    // Copies the vertical scroll bits (fine Y, coarse Y, and name table Y)
    // from the temporary address to the live address.
    fn copy_vertical_scroll_from_temp_to_live(&mut self) {
        // Bits 5-9 (coarse Y), bits 11-12 (name table Y and part of fine Y),
        // and bits 12-14 (fine Y) are copied. Actually bits 5-14 except bit 10.
        let vertical_bits_from_temp = self.temporary_vram_address & 0x7BE0;
        let preserved_horizontal_bits = self.current_vram_address & !0x7BE0;
        self.current_vram_address = preserved_horizontal_bits | vertical_bits_from_temp;
    }

    // =========================================================================
    // SPRITE EVALUATION
    // =========================================================================
    // The PPU looks at all 64 sprites and checks which ones touch the current
    // scanline. It can only display 8 sprites per scanline. If more than 8
    // sprites are found, it sets the "sprite overflow" flag.
    fn find_visible_sprites_for_this_scanline(&mut self) {
        // Start with an empty list for this scanline.
        self.visible_sprites_this_scanline.clear();

        let mut too_many_sprites = false;

        // Check every sprite (64 total, 4 bytes each in OAM).
        for sprite_index in 0..64 {
            // Byte 0 of each sprite is the Y position.
            let sprite_y = self.sprite_oam[sprite_index * 4];

            // A Y value of 0xEF or higher means the sprite is off-screen.
            // The NES uses this to hide sprites.
            if sprite_y >= 0xEF {
                continue;
            }

            // On the NES, a sprite with Y = 0 appears at screen Y = 1.
            // So the visible range starts one pixel lower than the stored value.
            let sprite_top_edge = (sprite_y as u16).wrapping_add(1);
            let sprite_bottom_edge = sprite_top_edge.wrapping_add(8);

            // Check if the current scanline falls within the sprite's vertical range.
            let scanline_covers_sprite = self.current_scanline >= sprite_top_edge
                && self.current_scanline < sprite_bottom_edge;

            if scanline_covers_sprite {
                if self.visible_sprites_this_scanline.len() < 8 {
                    // We have room! Add this sprite to our list.
                    let base_address = sprite_index * 4;
                    self.visible_sprites_this_scanline.push(Sprite {
                        index_in_oam: sprite_index as u8,
                        y_position_on_screen: sprite_y,
                        tile_number: self.sprite_oam[base_address + 1],
                        attribute_byte: self.sprite_oam[base_address + 2],
                        x_position_on_screen: self.sprite_oam[base_address + 3],
                    });
                } else {
                    // More than 8 sprites on this line! Set the overflow flag.
                    too_many_sprites = true;
                }
            }
        }

        if too_many_sprites {
            self.status_register |= STATUS_FLAG_SPRITE_OVERFLOW;
        }
    }

    // =========================================================================
    // PIXEL RENDERING
    // =========================================================================
    // This is where the magic happens! We figure out the color of ONE pixel
    // and write it into the screen buffer.
    fn draw_one_pixel(&mut self) {
        // Convert our cycle/scanline counters into screen coordinates.
        // Cycle 1 maps to screen X = 0, cycle 2 to X = 1, etc.
        let screen_x = (self.current_cycle - 1) as usize;
        let screen_y = self.current_scanline as usize;

        // Calculate where this pixel lives in our flat screen buffer array.
        // Each pixel uses 3 consecutive bytes: [R, G, B].
        let pixel_index_in_buffer = (screen_y * SCREEN_WIDTH_IN_PIXELS + screen_x) * BYTES_PER_PIXEL;

        // -------------------------------------------------------------------
        // PART A: DRAW THE BACKGROUND
        // -------------------------------------------------------------------
        let (background_color_index, background_pixel_value) =
            self.compute_background_pixel(screen_x, screen_y);

        // -------------------------------------------------------------------
        // PART B: SPRITE 0 HIT DETECTION
        // -------------------------------------------------------------------
        // Sprite 0 hit is a special signal used by games for split-screen effects.
        // It triggers when the FIRST sprite (sprite 0) touches a non-transparent
        // background pixel. We handle a 1-pixel delay in detection.
        self.apply_delayed_sprite_zero_hit();
        self.detect_sprite_zero_hit(screen_x, screen_y, background_pixel_value);

        // -------------------------------------------------------------------
        // PART C: DRAW THE SPRITES
        // -------------------------------------------------------------------
        let (sprite_color_index, sprite_pixel_value, sprite_goes_in_front) =
            self.compute_sprite_pixel(screen_x, screen_y);

        // -------------------------------------------------------------------
        // PART D: MIX BACKGROUND AND SPRITE COLORS
        // -------------------------------------------------------------------
        let final_color_index = self.mix_background_and_sprite(
            background_color_index,
            background_pixel_value,
            sprite_color_index,
            sprite_pixel_value,
            sprite_goes_in_front,
        );

        // Look up the actual RGB color and write it to the screen buffer.
        let (red, green, blue) = NES_COLOR_PALETTE[final_color_index as usize];
        self.screen_pixels[pixel_index_in_buffer] = red;
        self.screen_pixels[pixel_index_in_buffer + 1] = green;
        self.screen_pixels[pixel_index_in_buffer + 2] = blue;
    }

    // -------------------------------------------------------------------------
    // BACKGROUND PIXEL CALCULATION
    // -------------------------------------------------------------------------
    // The background is made of a grid of 8x8 pixel tiles.
    // We need to figure out which tile we are in, which pixel inside that tile,
    // and what color that pixel should be.
    fn compute_background_pixel(&self, screen_x: usize, _screen_y: usize) -> (u8, u8) {
        // If the background layer is turned off, return transparent (color 0).
        if !self.is_background_enabled() {
            let backdrop_color = self.read_video_memory(0x3F00);
            return (backdrop_color, 0);
        }

        // We use the SNAPSHOT of the VRAM address taken at cycle 258.
        // This ensures the entire scanline uses a consistent scroll position.
        let vram_address_for_this_scanline = self.render_vram_address_copy;

        // Extract the different parts of the address:
        //   - fine_y: which pixel row INSIDE the 8x8 tile (0-7)
        //   - coarse_x: which tile column in the name table (0-31)
        //   - coarse_y: which tile row in the name table (0-31)
        //   - name_table_select: which of the 4 name tables (0-3)
        let fine_y = ((vram_address_for_this_scanline >> 12) & 0x07) as u8;
        let coarse_x = (vram_address_for_this_scanline & 0x1F) as u16;
        let coarse_y = ((vram_address_for_this_scanline >> 5) & 0x1F) as u16;
        let name_table_select = ((vram_address_for_this_scanline >> 10) & 0x03) as u16;

        // The "fine X scroll" can shift the background up to 7 pixels horizontally.
        // This means the pixel we are drawing might actually come from a different
        // tile than the one the VRAM address points to.
        let effective_screen_x = screen_x as u16 + self.fine_x_scroll as u16;
        let tile_column_with_scroll = coarse_x + (effective_screen_x / 8);
        let pixel_x_inside_tile = (effective_screen_x % 8) as u8;
        let pixel_y_inside_tile = fine_y;

        // If we scrolled past the right edge of the name table, we need to
        // wrap around to the next name table horizontally.
        let mut active_name_table = name_table_select;
        let mut final_tile_column = tile_column_with_scroll;

        if tile_column_with_scroll >= 32 {
            active_name_table ^= 1; // Flip to the adjacent name table
            final_tile_column = tile_column_with_scroll & 0x1F; // Keep only the lower 5 bits
        }

        // -------------------------------------------------------------------
        // STEP 1: READ THE NAME TABLE TO FIND WHICH TILE TO DRAW
        // -------------------------------------------------------------------
        // Name tables start at memory address $2000. Each name table is 1KB.
        // The byte at each position tells us which tile number to look up
        // in the pattern tables (the CHR data from the cartridge).
        let name_table_base_address = 0x2000 | (active_name_table << 10);
        let tile_entry_address = name_table_base_address + (coarse_y << 5) + final_tile_column;
        let tile_number = self.read_video_memory(tile_entry_address) as u16;

        // -------------------------------------------------------------------
        // STEP 2: READ THE ACTUAL PIXEL FROM THE PATTERN TABLE
        // -------------------------------------------------------------------
        // The control register bit 4 selects which pattern table holds the
        // background tiles: 0 = first table ($0000), 1 = second table ($1000).
        let background_pattern_table = if self.control_register & CONTROL_FLAG_BACKGROUND_PATTERN_TABLE == 0 {
            0
        } else {
            1
        };

        let pixel_value = self.read_pixel_from_pattern_table(
            tile_number,
            background_pattern_table,
            pixel_x_inside_tile,
            pixel_y_inside_tile,
        );

        // -------------------------------------------------------------------
        // STEP 3: READ THE ATTRIBUTE TABLE TO FIND THE PALETTE
        // -------------------------------------------------------------------
        // The attribute table tells us which color palette to use for each
        // 16x16 pixel area (a group of 2x2 tiles). This is how the NES creates
        // color variation in the background without using too much memory.
        let attribute_block_x = final_tile_column >> 2; // Divide by 4 (each attribute covers 4 tiles wide)
        let attribute_block_y = coarse_y >> 2;          // Divide by 4 (each attribute covers 4 tiles tall)
        let attribute_table_address = (name_table_base_address + 0x03C0) + (attribute_block_y << 3) + attribute_block_x;
        let attribute_byte = self.read_video_memory(attribute_table_address);

        // Each attribute byte contains 4 palette choices (2 bits each).
        // We need to pick the correct 2-bit group based on which quadrant
        // of the 16x16 block our tile is in.
        let quadrant_shift = ((coarse_y & 0x02) << 1) | (final_tile_column & 0x02);
        let palette_number = (attribute_byte >> quadrant_shift) & 0x03;

        // -------------------------------------------------------------------
        // STEP 4: LOOK UP THE COLOR IN THE PALETTE RAM
        // -------------------------------------------------------------------
        // Palette 0, pixel 0 is always the "backdrop" color (universal background).
        // For any other pixel value, we calculate the address in palette RAM.
        let palette_ram_address = if pixel_value == 0 {
            0x3F00 // Universal background color
        } else {
            0x3F00 + (palette_number as u16) * 4 + (pixel_value as u16)
        };

        let color_index = self.read_video_memory(palette_ram_address);
        (color_index, pixel_value)
    }

    // -------------------------------------------------------------------------
    // SPRITE 0 HIT LOGIC
    // -------------------------------------------------------------------------
    // Sprite 0 hit is delayed by 1 pixel in this implementation.
    // This method applies the hit that was detected on the PREVIOUS pixel.
    fn apply_delayed_sprite_zero_hit(&mut self) {
        if self.sprite_zero_hit_detected_at_x == Some(self.current_cycle - 1) {
            self.status_register |= STATUS_FLAG_SPRITE_ZERO_HIT;
            self.sprite_zero_hit_detected_at_x = None;
        }
    }

    // Checks if sprite 0 (the very first sprite in OAM) is touching the
    // background at the current pixel position. If so, we schedule the hit
    // to be applied one cycle later.
    fn detect_sprite_zero_hit(&mut self, screen_x: usize, screen_y: usize, background_pixel_value: u8) {
        // Conditions that must ALL be true for sprite 0 hit to be possible:
        // 1. Sprites must be enabled
        // 2. Background must be enabled
        // 3. We must not already have a hit pending
        // 4. The sprite 0 hit flag must not already be set
        let sprites_turned_on = self.is_sprites_enabled();
        let background_turned_on = self.is_background_enabled();
        let no_hit_already_pending = self.sprite_zero_hit_detected_at_x.is_none();
        let hit_flag_not_yet_set = self.status_register & STATUS_FLAG_SPRITE_ZERO_HIT == 0;

        if !(sprites_turned_on && background_turned_on && no_hit_already_pending && hit_flag_not_yet_set) {
            return;
        }

        // Look for sprite 0 in the visible sprite list.
        for sprite in &self.visible_sprites_this_scanline {
            // We only care about sprite 0.
            if sprite.index_in_oam != 0 {
                continue;
            }

            let sprite_left_edge = sprite.x_position_on_screen as usize;
            let sprite_right_edge = sprite_left_edge + 8;

            // Check if the current screen X position falls within the sprite.
            if screen_x < sprite_left_edge || screen_x >= sprite_right_edge {
                break; // Sprite 0 is on this scanline but not at this X position
            }

            // Calculate which pixel of the sprite tile we are looking at.
            let mut pixel_column = screen_x - sprite_left_edge;
            let mut pixel_row = screen_y.wrapping_sub((sprite.y_position_on_screen as usize).wrapping_add(1));

            // Apply horizontal flip if the sprite asks for it.
            if sprite.attribute_byte & 0x40 != 0 {
                pixel_column = 7 - pixel_column;
            }
            // Apply vertical flip if the sprite asks for it.
            if sprite.attribute_byte & 0x80 != 0 {
                pixel_row = 7 - pixel_row;
            }

            // Read the pixel from the sprite's pattern table.
            let sprite_pattern_table = if self.control_register & CONTROL_FLAG_SPRITE_PATTERN_TABLE == 0 {
                0
            } else {
                1
            };

            let sprite_pixel_value = self.read_pixel_from_pattern_table(
                sprite.tile_number as u16,
                sprite_pattern_table,
                pixel_column as u8,
                pixel_row as u8,
            );

            // Sprite 0 hit requires BOTH the sprite pixel AND the background pixel
            // to be non-transparent (not 0).
            let sprite_is_visible = sprite_pixel_value != 0;
            let background_is_visible = background_pixel_value != 0;

            if sprite_is_visible && background_is_visible {
                // Check the "left clipping" rules. The NES can hide the leftmost
                // 8 pixels of the screen. If either background or sprites are clipped
                // on the left, the hit does not happen in that zone.
                let left_background_clipped = self.mask_register & MASK_FLAG_LEFT_CLIP_BACKGROUND != 0;
                let left_sprites_clipped = self.mask_register & MASK_FLAG_LEFT_CLIP_SPRITES != 0;
                let in_left_clip_zone = screen_x < 8;
                let clipped_by_left_edge = in_left_clip_zone && (left_background_clipped || left_sprites_clipped);

                // The hit also cannot happen on the very last pixel of the scanline (X=255).
                let is_last_pixel = screen_x == 255;

                if !is_last_pixel && !clipped_by_left_edge {
                    // Schedule the hit to be applied on the NEXT cycle.
                    self.sprite_zero_hit_detected_at_x = Some(self.current_cycle);
                }
            }

            // We only check sprite 0, so stop after finding it.
            break;
        }
    }

    // -------------------------------------------------------------------------
    // SPRITE PIXEL CALCULATION
    // -------------------------------------------------------------------------
    // Figures out which sprite (if any) covers the current pixel and what
    // color that sprite pixel should be.
    // Returns: (color_index, pixel_value, goes_in_front)
    fn compute_sprite_pixel(&self, screen_x: usize, screen_y: usize) -> (u8, u8, bool) {
        // If sprites are turned off, return transparent.
        if !self.is_sprites_enabled() {
            return (0, 0, true);
        }

        // The NES draws sprites in order from index 0 to 63.
        // Lower index sprites appear BEHIND higher index sprites.
        // To find the FRONT-most sprite at this pixel, we iterate BACKWARDS
        // through our visible sprite list. The first match wins.
        for sprite in self.visible_sprites_this_scanline.iter().rev() {
            let sprite_left_edge = sprite.x_position_on_screen as usize;
            let sprite_right_edge = sprite_left_edge + 8;

            // Check if the current pixel falls within this sprite's horizontal range.
            if screen_x < sprite_left_edge || screen_x >= sprite_right_edge {
                continue; // This sprite does not cover this X position; check the next one.
            }

            // Calculate which pixel of the sprite tile we are looking at.
            let mut pixel_column = screen_x - sprite_left_edge;
            let mut pixel_row = screen_y.wrapping_sub((sprite.y_position_on_screen as usize).wrapping_add(2));

            // If the pixel is outside the 8x8 tile area, skip this sprite.
            // (This check is mostly defensive; the math above should keep it in range.)
            if pixel_column >= 8 || pixel_row >= 8 {
                continue;
            }

            // Apply flip flags from the attribute byte.
            if sprite.attribute_byte & 0x40 != 0 {
                pixel_column = 7 - pixel_column; // Flip horizontally
            }
            if sprite.attribute_byte & 0x80 != 0 {
                pixel_row = 7 - pixel_row; // Flip vertically
            }

            // Determine which pattern table to read from.
            let sprite_pattern_table = if self.control_register & CONTROL_FLAG_SPRITE_PATTERN_TABLE == 0 {
                0
            } else {
                1
            };

            let pixel_value = self.read_pixel_from_pattern_table(
                sprite.tile_number as u16,
                sprite_pattern_table,
                pixel_column as u8,
                pixel_row as u8,
            );

            // If the pixel value is 0, this part of the sprite is transparent.
            // We keep checking behind it (lower index sprites).
            if pixel_value == 0 {
                continue;
            }

            // We found a visible sprite pixel!
            // Extract the palette number (bits 0-1 of the attribute byte).
            let sprite_palette_number = sprite.attribute_byte & 0x03;

            // Look up the color in the sprite palette area of palette RAM.
            // Sprite palettes start at $3F10, $3F14, $3F18, $3F1C.
            let palette_ram_address = 0x3F10 + (sprite_palette_number as u16) * 4 + (pixel_value as u16);
            let color_index = self.read_video_memory(palette_ram_address);

            // The "priority" bit (bit 5) tells us if the sprite should appear
            // IN FRONT OF or BEHIND the background.
            //   0 = in front of background
            //   1 = behind background
            let sprite_goes_in_front = sprite.attribute_byte & 0x20 == 0;

            return (color_index, pixel_value, sprite_goes_in_front);
        }

        // No visible sprite covers this pixel.
        (0, 0, true)
    }

    // -------------------------------------------------------------------------
    // BACKGROUND + SPRITE MIXING
    // -------------------------------------------------------------------------
    // Combines the background color and sprite color according to NES rules:
    //   - If the sprite is transparent, show the background.
    //   - If the sprite is in front, show the sprite.
    //   - If the sprite is behind but the background is transparent, show the sprite.
    //   - Otherwise, show the background.
    fn mix_background_and_sprite(
        &self,
        background_color_index: u8,
        background_pixel_value: u8,
        sprite_color_index: u8,
        sprite_pixel_value: u8,
        sprite_goes_in_front: bool,
    ) -> u8 {
        // If the sprite pixel is transparent (value 0), we always show the background.
        if sprite_pixel_value == 0 {
            return background_color_index;
        }

        // If the sprite wants to be in front, it wins over the background.
        if sprite_goes_in_front {
            return sprite_color_index;
        }

        // If the sprite wants to be behind, but the background is transparent,
        // we show the sprite anyway (you can see through the background).
        if background_pixel_value == 0 {
            return sprite_color_index;
        }

        // Otherwise, the background covers the sprite.
        background_color_index
    }

    // =========================================================================
    // PATTERN TABLE READING
    // =========================================================================
    // Pattern tables store the actual graphics. Each tile is 16 bytes:
    //   - bytes 0-7: the LOW bits of each pixel row
    //   - bytes 8-15: the HIGH bits of each pixel row
    //
    // By combining the low and high bit, we get a 2-bit pixel value (0-3).
    // Value 0 means transparent. Values 1-3 are colors from the palette.
    fn read_pixel_from_pattern_table(
        &self,
        tile_number: u16,
        pattern_table_select: u16,
        pixel_x: u8,
        pixel_y: u8,
    ) -> u8 {
        // Each pattern table is 4KB. Select the correct one.
        let pattern_table_start = (pattern_table_select as usize) * 4096;

        // Each tile is 16 bytes, so multiply the tile number by 16.
        let tile_start_address = pattern_table_start + (tile_number as usize) * 16;

        // Read the low bit plane and high bit plane for this row.
        let low_bits_byte = self.cartridge_chr_data[tile_start_address + (pixel_y as usize)];
        let high_bits_byte = self.cartridge_chr_data[tile_start_address + (pixel_y as usize) + 8];

        // The NES stores pixels in reverse order within each byte.
        // Pixel 0 is bit 7, pixel 1 is bit 6, etc.
        let bit_position = 7 - pixel_x;
        let low_bit = (low_bits_byte >> bit_position) & 0x01;
        let high_bit = (high_bits_byte >> bit_position) & 0x01;

        // Combine the two bits to get the final 2-bit pixel value.
        ((high_bit << 1) | low_bit) as u8
    }

    // =========================================================================
    // CPU INTERFACE: READING FROM PPU REGISTERS
    // =========================================================================
    // The CPU reads from these addresses to get information from the PPU:
    //   $2002 = Status Register
    //   $2004 = OAM Data (sprite memory)
    //   $2007 = PPU Data (video memory)
    pub fn read_cpu(&mut self, cpu_address: u16) -> u8 {
        match cpu_address {
            // ----------------------------------------------------------------
            // $2002 - Status Register
            // ----------------------------------------------------------------
            // The CPU reads this to know if VBLANK happened or if sprite 0 hit.
            // Reading this register has two side effects:
            //   1. It clears the "vertical blank" flag (bit 7).
            //   2. It resets the address latch (the first/second write toggle).
            0x2002 => {
                let value_to_return = self.status_register;
                self.status_register &= !STATUS_FLAG_VERTICAL_BLANK;
                self.address_latch_second_write = false;
                value_to_return
            }

            // ----------------------------------------------------------------
            // $2004 - OAM Data Register
            // ----------------------------------------------------------------
            // The CPU can read the sprite memory directly through this port.
            0x2004 => self.sprite_oam[self.oam_address_register as usize],

            // ----------------------------------------------------------------
            // $2007 - PPU Data Register
            // ----------------------------------------------------------------
            // The CPU reads video memory through this port. Because the PPU
            // is slow, there is a one-read delay for most addresses.
            // Palette addresses ($3F00-$3FFF) are NOT delayed.
            0x2007 => {
                let value = if self.current_vram_address < 0x3F00 {
                    // For normal VRAM, return the PREVIOUSLY buffered value
                    // and store the current value for next time.
                    let previously_buffered_value = self.vram_read_buffer;
                    self.vram_read_buffer = self.read_video_memory(self.current_vram_address);
                    previously_buffered_value
                } else {
                    // For palette RAM, there is no delay. Return the color immediately.
                    // However, the buffer still gets filled with the mirrored VRAM value.
                    let palette_color = self.read_video_memory(self.current_vram_address);
                    self.vram_read_buffer = self.read_video_memory(self.current_vram_address - 0x1000);
                    palette_color
                };

                // After every read, the VRAM address auto-increments by 1 or 32,
                // depending on the control register setting.
                let increment = self.vram_address_increment();
                self.current_vram_address = self.current_vram_address.wrapping_add(increment);

                value
            }

            // Any other address is not used for reading from the PPU.
            _ => 0,
        }
    }

    // =========================================================================
    // CPU INTERFACE: WRITING TO PPU REGISTERS
    // =========================================================================
    // The CPU writes to these addresses to control the PPU:
    //   $2000 = Control Register
    //   $2001 = Mask Register
    //   $2003 = OAM Address
    //   $2004 = OAM Data
    //   $2005 = Scroll Register
    //   $2006 = PPU Address
    //   $2007 = PPU Data
    pub fn write_cpu(&mut self, cpu_address: u16, value: u8) {
        match cpu_address {
            // ----------------------------------------------------------------
            // $2000 - Control Register
            // ----------------------------------------------------------------
            // This sets basic PPU options. The bottom 2 bits also update the
            // "temporary VRAM address" (t) to select the starting name table.
            0x2000 => {
                self.control_register = value;
                // Update the name table selection in the temporary address.
                // Bits 0-1 of the control register become bits 10-11 of t.
                let name_table_bits = (value as u16) & 0x03;
                self.temporary_vram_address = (self.temporary_vram_address & !0x0C00) | (name_table_bits << 10);
            }

            // ----------------------------------------------------------------
            // $2001 - Mask Register
            // ----------------------------------------------------------------
            // This turns rendering on/off and controls color emphasis.
            0x2001 => {
                self.mask_register = value;
            }

            // ----------------------------------------------------------------
            // $2003 - OAM Address Register
            // ----------------------------------------------------------------
            // Sets which byte of sprite memory (OAM) the CPU wants to access.
            0x2003 => {
                self.oam_address_register = value;
            }

            // ----------------------------------------------------------------
            // $2004 - OAM Data Register
            // ----------------------------------------------------------------
            // Writes one byte to sprite memory and automatically moves to the
            // next byte. This is how games load all their sprite data.
            0x2004 => {
                self.sprite_oam[self.oam_address_register as usize] = value;
                self.oam_address_register = self.oam_address_register.wrapping_add(1);
            }

            // ----------------------------------------------------------------
            // $2005 - Scroll Register
            // ----------------------------------------------------------------
            // This is the trickiest register! The CPU writes the horizontal
            // and vertical scroll values here, one byte at a time.
            // The first write sets the horizontal scroll (and fine X).
            // The second write sets the vertical scroll.
            0x2005 => {
                if !self.address_latch_second_write {
                    // FIRST WRITE: horizontal scroll
                    // The bottom 3 bits become the fine X scroll.
                    self.fine_x_scroll = value & 0x07;
                    // The top 5 bits become the coarse X position in the temporary address.
                    self.temporary_vram_address = (self.temporary_vram_address & 0xFFE0) | ((value as u16) >> 3);
                    self.address_latch_second_write = true;
                } else {
                    // SECOND WRITE: vertical scroll
                    // The bottom 3 bits become the fine Y scroll (bits 12-14 of t).
                    // The top 5 bits become the coarse Y position (bits 5-9 of t).
                    let fine_y = ((value as u16) & 0x07) << 12;
                    let coarse_y = ((value as u16) & 0xF8) << 2;
                    self.temporary_vram_address = (self.temporary_vram_address & 0x8C1F) | fine_y | coarse_y;
                    self.address_latch_second_write = false;
                }
            }

            // ----------------------------------------------------------------
            // $2006 - PPU Address Register
            // ----------------------------------------------------------------
            // The CPU writes the 16-bit VRAM address here, one byte at a time.
            // The first write sets the high byte, the second sets the low byte.
            // After the second write, the address is copied to the live address (v).
            0x2006 => {
                if !self.address_latch_second_write {
                    // FIRST WRITE: high byte of the address
                    // Only the bottom 6 bits are used (the PPU address space is 14 bits).
                    self.temporary_vram_address = (self.temporary_vram_address & 0x00FF) | (((value as u16) & 0x3F) << 8);
                    self.address_latch_second_write = true;
                } else {
                    // SECOND WRITE: low byte of the address
                    self.temporary_vram_address = (self.temporary_vram_address & 0xFF00) | (value as u16);
                    // Copy the complete address to the live address.
                    self.current_vram_address = self.temporary_vram_address;
                    self.address_latch_second_write = false;
                }
            }

            // ----------------------------------------------------------------
            // $2007 - PPU Data Register
            // ----------------------------------------------------------------
            // The CPU writes data directly to video memory at the current address.
            // After writing, the address auto-increments by 1 or 32.
            0x2007 => {
                self.write_video_memory(self.current_vram_address, value);
                let increment = self.vram_address_increment();
                self.current_vram_address = self.current_vram_address.wrapping_add(increment);
            }

            // Any other address is not used for writing to the PPU.
            _ => {}
        }
    }

    // =========================================================================
    // VIDEO MEMORY READ
    // =========================================================================
    // Reads a byte from the PPU's internal memory map. The address space is:
    //   $0000-$1FFF = Pattern tables (graphics tiles from cartridge)
    //   $2000-$2FFF = Name tables (background layout)
    //   $3000-$3EFF = Mirror of name tables
    //   $3F00-$3FFF = Palette RAM (colors)
    fn read_video_memory(&self, address: u16) -> u8 {
        // The PPU only has a 14-bit address bus, so we mask to $3FFF.
        let address = address & 0x3FFF;

        match address {
            0..=0x1fff=> {
                // Pattern table data comes from the cartridge CHR ROM/RAM.
                let index = address as usize;
                if index < self.cartridge_chr_data.len() {
                    self.cartridge_chr_data[index]
                } else {
                    // If the cartridge data is smaller than expected, return 0.
                    0
                }
            }

           0x2000..=0x2fff => {
                // Name table data comes from internal VRAM, but the 4 name tables
                // may be mirrored depending on the cartridge's mirroring setting.
                let mirrored_address = self.apply_mirroring(address);
                self.name_table_vram[(mirrored_address - 0x2000) as usize]
           },

           0x3000..=0x3eff => panic!("addr space 0x3000..0x3eff is not expected to be used, requested = {} ", address),
           0x3f00..=0x3fff => {
                // Palette RAM. Only 32 bytes are actually used.
                let mut palette_index = address & 0x1F;

                // The NES has a quirk: every 4th color in the sprite palettes
                // mirrors the corresponding background palette color.
                // So $3F10 mirrors $3F00, $3F14 mirrors $3F04, etc.
                if palette_index >= 0x10 && (palette_index & 0x03) == 0 {
                    palette_index -= 0x10;
                }

                self.palette_ram[palette_index as usize]
           },
           _ => 0 
        }
    }

    // =========================================================================
    // VIDEO MEMORY WRITE
    // =========================================================================
    // Writes a byte to the PPU's internal memory map.
    fn write_video_memory(&mut self, address: u16, value: u8) {
        let address = address & 0x3FFF;

        if address < 0x2000 {
            // Some cartridges use CHR RAM instead of ROM, allowing the CPU
            // to write graphics data directly.
            if self.cartridge_chr_data.len() > address as usize {
                self.cartridge_chr_data[address as usize] = value;
            }
        } else if address < 0x3F00 {
            // Name table writes go through the mirroring logic.
            let mirrored_address = self.apply_mirroring(address);
            self.name_table_vram[(mirrored_address - 0x2000) as usize] = value;
        } else {
            // Palette RAM write.
            let mut palette_index = address & 0x1F;

            // Same mirroring quirk as in read_video_memory.
            if palette_index >= 0x10 && (palette_index & 0x03) == 0 {
                palette_index -= 0x10;
            }

            self.palette_ram[palette_index as usize] = value;
        }
    }

    // =========================================================================
    // NAME TABLE MIRRORING
    // =========================================================================
    // The NES has 4 name table slots in memory ($2000, $2400, $2800, $2C00),
    // but only 2KB of actual RAM. Cartridges decide how to map the 4 slots
    // onto the 2 physical tables:
    //
    //   Vertical mirroring:   the two tables are arranged side-by-side.
    //                         Good for horizontal scrolling.
    //
    //   Horizontal mirroring: the two tables are arranged one above the other.
    //                         Good for vertical scrolling.
    //
    //   Four-screen:          the cartridge provides all 4 tables (rare).
    fn apply_mirroring(&self, address: u16) -> u16 {
        // The offset within a single name table (0 to 1023).
        let offset_within_table = address & 0x03FF;

        // Which of the 4 name table slots the address is asking for (0-3).
        let requested_table = (address >> 10) & 0x03;

        // Map the requested table to an actual physical table (0 or 1).
        let physical_table = match self.screen_mirroring {
            Mirroring::Vertical => requested_table & 0x01,
            Mirroring::Horizontal => (requested_table & 0x02) >> 1,
            Mirroring::FourScreen => requested_table,
            _ => 0,
        };

        // Reconstruct the final mirrored address.
        0x2000 | (physical_table << 10) | offset_within_table
    }

    // =========================================================================
    // CLOCK ADVANCE
    // =========================================================================
    // Moves the PPU forward by one clock cycle. When we reach the end of a
    // scanline, we move to the next one. When we finish the last scanline,
    // we start a new frame.
    fn advance_clock(&mut self) {
        self.current_cycle += 1;

        // Each scanline has 341 cycles (numbered 0 to 340).
        if self.current_cycle > 340 {
            self.current_cycle = 0;
            self.current_scanline += 1;

            // There are 262 scanlines per frame (numbered 0 to 261).
            if self.current_scanline > 261 {
                self.current_scanline = 0;
                self.frame_count += 1;
            }
        }
    }
}
