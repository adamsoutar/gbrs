use crate::colour::colour::Colour;
use crate::colour::grey_shades;
use crate::colour::grey_shades::colour_from_grey_shade_id;
use crate::combine_u8;
use crate::constants::*;
use crate::lcd::*;
use crate::memory::ram::Ram;
use crate::memory::memory::Memory;
use crate::interrupts::*;
use crate::log;

use smallvec::SmallVec;

#[derive(Clone)]
pub struct Sprite {
    pub y_pos: i32,
    pub x_pos: i32,
    pub pattern_id: u8,

    pub above_bg: bool,
    pub y_flip: bool,
    pub x_flip: bool,
    pub use_palette_0: bool,

    // CGB-specific attributes
    pub use_upper_vram_bank: bool,
    pub cgb_palette: u8
}

pub struct Gpu {
    cgb_features: bool,
    // This is the WIP frame that the GPU draws to
    frame: [Colour; SCREEN_BUFFER_SIZE],
    // This is the last rendered frame displayed on the LCD, only updated
    // in VBlank. GUI implementations can read it to show the display.
    pub finished_frame: [Colour; SCREEN_BUFFER_SIZE],

    // X and Y of background position
    scy: u8,
    scx: u8,

    // X and Y of the Window
    wy: u8,
    wx: u8,

    // The scan-line Y co-ordinate
    ly: u8,
    // If ly is lyc ("compare") and the interrupt is enabled,
    // an LCD Status interrupt is flagged
    lyc: u8,
    // The "Window internal line counter" - relied upon by a handful of
    // unusual games and DMG-ACID2.
    window_line_counter: u8,

    // Scan-line X co-ordinate
    // This isn't a real readable Gameboy address, it's just for internal tracking
    lx: u16,

    bg_pallette: u8,
    sprite_pallete_1: u8,
    sprite_pallete_2: u8,

    status: LcdStatus,
    control: LcdControl,

    // "Object Attribute Memory" - Sprite properties
    oam: Ram,

    dma_source: u8,
    dma_cycles: u8,

    cgb_dma_source: u16,
    cgb_dma_dest: u16,
    cgb_dma_cycles: u16,

    // The global 40-sprite OAM cache
    // SmallVec doesn't do blocks of 40 so we leave 24 empty slots, it's still
    // more performant than allocating.
    sprite_cache: SmallVec<[Sprite; 64]>,
    // The per-scanline 10-sprite cache
    // TODO: These come straight from sprite_cache. Maybe they can be &Sprite?
    //   Would that be faster?
    sprites_on_line: SmallVec<[Sprite; 10]>
}

impl Gpu {
    pub fn raw_write (&mut self, raw_address: u16, value: u8, ints: &mut Interrupts) {
        match raw_address {
            OAM_START ..= OAM_END => self.oam.write(raw_address - OAM_START, value),

            0xFF40 => {
                let original_display_enable = self.control.display_enable;
                self.control = LcdControl::from(value);

                if original_display_enable && !self.control.display_enable {
                    // The LCD has just been turned off
                    self.ly = 0;
                    self.status.set_mode(LcdMode::HBlank);
                    if self.status.hblank_interrupt {
                        ints.raise_interrupt(InterruptReason::LCDStat)
                    }
                    self.lyc = 0;
                }
                if !original_display_enable && self.control.display_enable {
                    // The LCD has just been turned on
                    self.status.set_mode(LcdMode::OAMSearch);
                    if self.status.oam_interrupt {
                        ints.raise_interrupt(InterruptReason::LCDStat);
                    }
                    self.cache_all_sprites();
                }
            },
            0xFF41 => self.status.set_data(value, ints),
            0xFF42 => self.scy = value,
            0xFF43 => self.scx = value,
            0xFF45 => self.lyc = value,

            0xFF46 => self.begin_dma(value),

            0xFF47 => self.bg_pallette = value,
            0xFF48 => self.sprite_pallete_1 = value,
            0xFF49 => self.sprite_pallete_2 = value,

            0xFF4A => self.wy = value,
            0xFF4B => self.wx = value,

            // TODO: 0xFF4D "CGB Prepare Speed Switch" is in this range.
            0xFF4C ..= 0xFF4E => log!("[WARN] Unknown LCD register write at {:#06x} (value: {:#04x})", raw_address, value),

            // The Y Scanline is read only.
            // Space Invaders writes here. As a bug?
            0xFF44 => {},

            0xFF51 => self.cgb_dma_source = (self.cgb_dma_source & 0x0F) | ((value as u16) << 8),
            0xFF52 => self.cgb_dma_source = (self.cgb_dma_source & 0xF0) | (value as u16),
            0xFF53 => self.cgb_dma_dest = (self.cgb_dma_dest & 0x0F) | ((value as u16) << 8),
            0xFF54 => self.cgb_dma_dest = (self.cgb_dma_dest & 0xF0) | (value as u16),

            _ => panic!("Unsupported GPU write at {:#06x} (value: {:#04x})", raw_address, value)
        }
    }
    pub fn raw_read (&self, raw_address: u16) -> u8 {
        match raw_address {
            OAM_START ..= OAM_END => self.oam.read(raw_address - OAM_START),

            0xFF40 => u8::from(self.control),
            0xFF41 => u8::from(self.status),
            0xFF42 => self.scy,
            0xFF43 => self.scx,
            0xFF44 => self.ly,
            0xFF45 => self.lyc,

            0xFF46 => self.dma_source,

            0xFF4A => self.wy,
            0xFF4B => self.wx,

            0xFF47 => self.bg_pallette,
            0xFF48 => self.sprite_pallete_1,
            0xFF49 => self.sprite_pallete_2,

            // High and low bits of a 16-bit register
            0xFF51 => (self.cgb_dma_source >> 8) as u8,
            0xFF52 => (self.cgb_dma_source & 0xFF) as u8,
            0xFF53 => (self.cgb_dma_dest >> 8) as u8,
            0xFF54 => (self.cgb_dma_dest & 0xFF) as u8,

            _ => { log!("Unsupported GPU read at {:#06x}", raw_address); 0xFF }
        }
    }

    fn cache_all_sprites (&mut self) {
        // There's room for 40 sprites in the OAM table
        let mut i = 0;
        while i < 40 {
            let address: u16 = i as u16 * 4;

            let y_pos = self.oam.read(address) as i32 - 16;
            let x_pos = self.oam.read(address + 1) as i32 - 8;
            let pattern_id = self.oam.read(address + 2);
            let attribs = self.oam.read(address + 3);

            let above_bg = (attribs & 0b1000_0000) == 0;
            let y_flip = (attribs & 0b0100_0000) > 0;
            let x_flip = (attribs & 0b0010_0000) > 0;
            let use_palette_0 = (attribs & 0b0001_0000) == 0;
            let use_upper_vram_bank = (attribs & 0b0000_1000) > 0;
            let cgb_palette = attribs & 0b0000_0111;

            if self.sprite_cache.len() > i {
                self.sprite_cache[i] = Sprite {
                    y_pos, x_pos, pattern_id,
                    above_bg, y_flip, x_flip, use_palette_0,
                    use_upper_vram_bank, cgb_palette
                };
            } else {
                self.sprite_cache.push(Sprite {
                    y_pos, x_pos, pattern_id,
                    above_bg, y_flip, x_flip, use_palette_0,
                    use_upper_vram_bank, cgb_palette
                });
            }

            i += 1;
        }

        self.sprite_cache.truncate(i);
    }

    fn begin_dma(&mut self, source: u8) {
        // Really, we should be disabling access to anything but HRAM now,
        // but if the rom is nice then there shouldn't be an issue.
        if self.dma_cycles != 0 {
            log!("INTERRUPTING DMA!")
        }
        self.dma_source = source;
        self.dma_cycles = gpu_timing::DMA_CYCLES;
    }

    fn update_dma (&mut self, ints: &mut Interrupts, mem: &mut Memory) {
        // There isn't one pending
        if self.dma_cycles == 0 { return; }

        self.dma_cycles -= 1;
        // Ready to actually perform DMA?
        if self.dma_cycles == 0 {
            let source = (self.dma_source as u16) * 0x100;

            for i in 0x00..=0x9F {
                let data = mem.read(ints, self, source + i);
                self.oam.write(i, data);
            }
        }
    }

    fn enter_vblank (&mut self, ints: &mut Interrupts) {
        ints.raise_interrupt(InterruptReason::VBlank);

        // TODO: This seems like odd behaviour to me.
        if self.status.vblank_interrupt {
            ints.raise_interrupt(InterruptReason::LCDStat);
        }

        self.finished_frame = self.frame.clone();
    }

    fn run_ly_compare (&mut self, ints: &mut Interrupts) {
        if self.ly == self.lyc {
            self.status.coincidence_flag = true;

            if self.status.lyc {
                ints.raise_interrupt(InterruptReason::LCDStat);
            }
        }
    }

    pub fn step(&mut self, ints: &mut Interrupts, mem: &mut Memory) {
        // TODO: Check that a DMA is performed even with display off
        self.update_dma(ints, mem);

        if !self.control.display_enable {
            return;
        }

        self.lx += 1;
        if self.lx == gpu_timing::HTOTAL {
            self.lx = 0;
        }

        let mode = self.status.get_mode();

        if mode == LcdMode::VBlank {
            if self.lx == 0 {
                self.ly += 1;
                if self.ly == gpu_timing::VTOTAL {
                    self.ly = 0;
                }

                self.run_ly_compare(ints);

                if self.ly == 0 {
                    self.window_line_counter = 0;
                    if self.status.oam_interrupt {
                        ints.raise_interrupt(InterruptReason::LCDStat);
                    }
                    self.status.set_mode(LcdMode::OAMSearch);
                    self.cache_all_sprites();
                    self.draw_line_if_necessary(ints, mem);
                }
            }
            return;
        }

        if self.lx == 0 {
            // Unusual GPU implementation detail. This is only
            // incremented when the Window was drawn on this scanline.
            // TODO: Relate these magic numbers to constants.
            if self.control.window_enable &&
                self.wx < 166 &&
                self.wy < 143 &&
                self.ly >= self.wy
            {
                self.window_line_counter += 1;
            }

            self.ly += 1;

            self.run_ly_compare(ints);
            // Done with frame, enter VBlank
            if self.ly == gpu_timing::VBLANK_ON {
                self.enter_vblank(ints);
                self.status.set_mode(LcdMode::VBlank);
            } else {
                if mode != LcdMode::OAMSearch {
                    self.status.set_mode(LcdMode::OAMSearch);
                    self.cache_all_sprites();
                    self.draw_line_if_necessary(ints, mem);
                }
            }
            return;
        }

        if self.lx == gpu_timing::HTRANSFER_ON {
            self.status.set_mode(LcdMode::Transfer);
            self.draw_line_if_necessary(ints, mem);
            return;
        }

        if self.lx == gpu_timing::HBLANK_ON {
            if self.status.hblank_interrupt {
                ints.raise_interrupt(InterruptReason::LCDStat)
            }
            self.status.set_mode(LcdMode::HBlank);
            return;
        }

        self.draw_line_if_necessary(ints, mem);
    }

    #[inline(always)]
    fn draw_line_if_necessary (&mut self, ints: &mut Interrupts, mem: &mut Memory) {
        let line_start = gpu_timing::HTRANSFER_ON +
            if self.ly == 0 { 160 } else { 48 };

        if self.lx == line_start {
            // Draw the current line
            // TODO: Move these draw_pixel calls into the mode switch
            //       to allow mid-scanline visual effects
            self.cache_sprites_on_line(self.ly);
            for x in 0..(SCREEN_WIDTH as u8) {
                self.draw_pixel(ints, mem, x, self.ly);
            }
        }
    }

    fn draw_pixel (&mut self, ints: &Interrupts, mem: &Memory, x: u8, y: u8) {
        let ux = x as usize; let uy = y as usize;
        let idx = uy * SCREEN_WIDTH + ux;

        let bg_col: Colour;
        let bg_col_id = if self.cgb_features || self.control.bg_display {
            let (new_col, id) = self.get_background_colour_at(ints, mem, x, y);
            bg_col = new_col;
            id
        } else {
            bg_col = grey_shades::white();
            0
        };

        // If there's a non-transparent sprite here, use its colour
        let s_col = self.get_sprite_colour_at(mem, bg_col, bg_col_id, x, y);

        self.frame[idx] = s_col;
    }

    fn get_colour_id_in_line (&self, tile_line: u16, subx: u8) -> u16 {
        let lower = tile_line & 0xFF;
        let upper = (tile_line & 0xFF00) >> 8;

        let shift_amnt = 7 - subx;
        let mask = 1 << shift_amnt;
        let u = (upper & mask) >> shift_amnt;
        let l = (lower & mask) >> shift_amnt;
        let pixel_colour_id = (u << 1) | l;

        pixel_colour_id
    }

    fn get_shade_from_colour_id (&self, pixel_colour_id: u16, palette: u8) -> Colour {
        let shift_2 = pixel_colour_id * 2;
        let shade = (palette & (0b11 << shift_2)) >> shift_2;

        colour_from_grey_shade_id(shade)
    }

    fn get_background_colour_at (&self, ints: &Interrupts, mem: &Memory, x: u8, y: u8) -> (Colour, u16) {
        let is_window = self.control.window_enable &&
            x as isize > self.wx as isize - 8 && y >= self.wy;

        let tilemap_select = if is_window {
            self.control.window_tile_map_display_select
        } else {
            self.control.bg_tile_map_display_select
        };

        let tilemap_base = if tilemap_select {
            0x9C00
        } else { 0x9800 };

        // This is which tile ID our pixel is in
        let x16: u16;
        let y16: u16;

        if is_window {
            // TODO: Check this saturating_sub, it's a guess.
            //   Super Mario Bros Deluxe pause menu crashes without it
            x16 = x.wrapping_sub(self.wx.saturating_sub(7)) as u16;
            y16 = self.window_line_counter as u16;
        } else {
            x16 = x.wrapping_add(self.scx) as u16;
            y16 = y.wrapping_add(self.scy) as u16;
        }

        let tx = x16 / 8; let ty = y16 / 8;
        // NOTE: Things like y16 % 8 is equivalent to y16 - ty * 8
        //   However, this is not more performant. I think the compiler
        //   is smart enough to recognise that.
        let mut subx = (x16 % 8) as u8; let mut suby = y16 % 8;

        let byte_offset = ty * 32 + tx;
        let tilemap_address = tilemap_base + byte_offset;
        let tile_metadata = mem.vram.bg_map_attributes.get_entry(byte_offset);

        let tile_id_raw = mem.read(ints, self, tilemap_address);
        let tile_id: u16;

        if self.control.bg_and_window_data_select {
            // 0x8000 addressing mode
            tile_id = tile_id_raw as u16;
        } else {
            // 0x8800 addressing mode
            if tile_id_raw < 128 {
                tile_id = (tile_id_raw as u16) + 256;
            } else { tile_id = tile_id_raw as u16 }
        }

        // BG tile flipping is a CGB-exclusive feature
        if self.cgb_features {
            if tile_metadata.x_flip {
                subx = 7 - subx;
            }
            if tile_metadata.y_flip {
                suby = 7 - suby;
            }
        }

        let tile_byte_offset = tile_id * 16;
        let tile_line_offset = tile_byte_offset + (suby * 2);

        // This is the line of the tile data that out pixel resides on
        let tiledata_base = 0x8000;
        let tile_address = tiledata_base + tile_line_offset;

        let bank = if self.cgb_features { tile_metadata.vram_bank as u16 } else { 0 };
        let tile_line0 = mem.vram.read_arbitrary_bank(bank, tile_address);
        let tile_line1 = mem.vram.read_arbitrary_bank(bank, tile_address + 1);
        let tile_line = combine_u8!(tile_line1, tile_line0);

        let col_id = self.get_colour_id_in_line(tile_line, subx);

        if self.cgb_features {
            let colour = mem.palette_ram.get_bg_palette_colour(tile_metadata.palette as u16, col_id);
            (colour, col_id)
        } else {
            (self.get_shade_from_colour_id(col_id, self.bg_pallette), col_id)
        }
    }

    fn get_sprite_colour_at (&self, mem: &Memory, bg_col: Colour, bg_col_id: u16, x: u8, y: u8) -> Colour {
        // Sprites are hidden for this scanline
        if !self.control.obj_enable {
            return bg_col
        }

        let sprite_height = if self.control.obj_size { 16 } else { 8 };

        let ix = x as i32; let iy = y as i32;

        let mut maybe_colour: Option<Colour> = None;
        let mut min_x: i32 = SCREEN_WIDTH as i32 + 8;
        for sprite in &self.sprites_on_line {
            let mut above_bg = sprite.above_bg;
            // In CGB mode, bg_display off means sprites always get priority.
            if self.cgb_features && !self.control.bg_display {
                above_bg = true;
            }

            if sprite.x_pos <= ix && (sprite.x_pos + 8) > ix && sprite.x_pos < min_x {
                if !above_bg && bg_col_id != 0 {
                    continue;
                }

                let mut subx = (ix - sprite.x_pos) as u8;
                let mut suby = iy - sprite.y_pos;

                // Tile address for 8x8 mode
                let mut pattern = sprite.pattern_id;

                if sprite_height == 16 {
                    if suby > 7 {
                        suby -= 8;

                        if sprite.y_flip {
                            pattern = sprite.pattern_id & 0xFE;
                        } else {
                            pattern = sprite.pattern_id | 0x01;
                        }
                    } else {
                        if sprite.y_flip {
                            pattern = sprite.pattern_id | 0x01;
                        } else {
                            pattern = sprite.pattern_id & 0xFE;
                        }
                    }
                }

                if sprite.x_flip { subx = 7 - subx }
                // TODO: Not sure if this applies to vertically flipped 8x16 mode sprites
                if sprite.y_flip { suby = 7 - suby }

                let tile_address = 0x8000 + (pattern as u16) * 16;
                let line_we_need = suby as u16 * 2;
                let bank = if self.cgb_features && sprite.use_upper_vram_bank { 1 } else { 0 };
                let tile_address = tile_address + line_we_need;
                // log!("Sprite at [{},{}] is using upper VRAM bank? {:?}, pattern_id {}, address: {:#06x}", sprite.x_pos, sprite.y_pos, sprite.use_upper_vram_bank, sprite.pattern_id, tile_address);

                let tile_line0 = mem.vram.read_arbitrary_bank(bank, tile_address);
                let tile_line1 = mem.vram.read_arbitrary_bank(bank, tile_address + 1);
                let tile_line = combine_u8!(tile_line1, tile_line0);

                let col_id = self.get_colour_id_in_line(tile_line, subx);

                if col_id == 0 {
                    // This pixel is transparent
                    continue
                } else {
                    if self.cgb_features {
                        let colour = mem.palette_ram.get_obj_palette_colour(sprite.cgb_palette as u16, col_id);
                        maybe_colour = Some(colour)
                    } else {
                        let palette = if sprite.use_palette_0
                            { self.sprite_pallete_1 } else { self.sprite_pallete_2 };

                        min_x = sprite.x_pos;
                        maybe_colour = Some(self.get_shade_from_colour_id(col_id, palette))
                    }
                }
            }
        }

        match maybe_colour {
            Some(col) => col,
            None => bg_col
        }
    }

    // Will be used later for get_sprite_pixel
    fn cache_sprites_on_line (&mut self, y: u8) {
        let sprite_height = if self.control.obj_size { 16 } else { 8 };

        let iy = y as i32;
        self.sprites_on_line.truncate(0);
        for s in &self.sprite_cache {
            if s.y_pos <= iy && (s.y_pos + sprite_height) > iy {
                self.sprites_on_line.push(s.clone());
            }
            if self.sprites_on_line.len() == 10 { break }
        }
    }

    pub fn get_rgba_frame (&self) -> [u8; SCREEN_RGBA_SLICE_SIZE] {
        let mut out_array = [0; SCREEN_RGBA_SLICE_SIZE];
        for i in 0..SCREEN_BUFFER_SIZE {
            let start = i * 4;
            out_array[start] = self.finished_frame[i].red;
            out_array[start + 1] = self.finished_frame[i].green;
            out_array[start + 2] = self.finished_frame[i].blue;
            out_array[start + 3] = 0xFF;
        }
        out_array
    }

    pub fn new (cgb_features: bool) -> Gpu {
        let empty_frame = [grey_shades::white(); SCREEN_BUFFER_SIZE];
        Gpu {
            cgb_features,
            frame: empty_frame,
            finished_frame: empty_frame.clone(),
            window_line_counter: 0,
            scy: 0, scx: 0, ly: 0, lx: 0, lyc:0, wy: 0, wx: 0,
            bg_pallette: 0, sprite_pallete_1: 0, sprite_pallete_2: 0,
            status: LcdStatus::new(),
            control: LcdControl::new(),
            oam: Ram::new(OAM_SIZE),
            dma_source: 0, dma_cycles: 0,
            cgb_dma_source: 0, cgb_dma_dest: 0, cgb_dma_cycles: 0,
            sprite_cache: SmallVec::with_capacity(40),
            sprites_on_line: SmallVec::with_capacity(10)
        }
    }
}
