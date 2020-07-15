use crate::gameboy::constants::*;
use crate::gameboy::lcd::*;
use crate::gameboy::memory::ram::Ram;
use crate::gameboy::memory::memory::Memory;
use crate::gameboy::interrupts::*;

pub struct Sprite {
    pub y_pos: u8,
    pub x_pos: u8,
    pub pattern_id: u8,

    pub above_bg: bool,
    pub y_flip: bool,
    pub x_flip: bool,
    pub use_palette_0: bool
}

fn get_all_sprites (gpu: &Gpu, ints: &Interrupts, mem: &Memory) -> Vec<Sprite> {
    let mut out = vec![];

    // There's room for 40 sprites in the OAM table
    for i in 0..40 {
        let address: u16 = 0xFE00 + i * 4;

        let y_pos = mem.read(ints, gpu, address);
        let x_pos = mem.read(ints, gpu, address + 1);
        let pattern_id = mem.read(ints, gpu, address + 2);
        let attribs = mem.read(ints, gpu, address + 3);

        let above_bg = (attribs & 0b1000_0000) != 0b1000_0000;
        let y_flip = (attribs & 0b0100_0000) == 0b0100_0000;
        let x_flip = (attribs & 0b0010_0000) == 0b0010_0000;
        let use_palette_0 = (attribs & 0b0001_0000) != 0b0001_0000;

        out.push(Sprite {
            y_pos, x_pos, pattern_id,
            above_bg, y_flip, x_flip, use_palette_0
        })
    }

    out
}

pub struct Gpu {
    // This is the WIP frame that the GPU draws to
    frame: [GreyShade; SCREEN_BUFFER_SIZE],
    // This is the frame read by the GUI,
    // it's only updated in VBlank
    finished_frame: [GreyShade; SCREEN_BUFFER_SIZE],

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
    dma_cycles: u8
}

impl Gpu {
    pub fn raw_write (&mut self, raw_address: u16, value: u8) {
        match raw_address {
            OAM_START ..= OAM_END => self.oam.write(raw_address - OAM_START, value),

            0xFF40 => self.control = LcdControl::from(value),
            0xFF41 => self.status.set_data(value),
            0xFF42 => self.scy = value,
            0xFF43 => self.scx = value,
            0xFF45 => self.lyc = value,

            0xFF46 => self.begin_dma(value),

            0xFF4A => self.wy = value,
            0xFF4B => self.wx = value,

            0xFF47 => self.bg_pallette = value,
            0xFF48 => self.sprite_pallete_1 = value,
            0xFF49 => self.sprite_pallete_2 = value,

            // CGB only
            0xFF4F => {},
            _ => panic!("Unsupported GPU write at {:#06x}", raw_address)
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
            _ => panic!("Unsupported GPU read at {:#06x}", raw_address)
        }
    }

    fn begin_dma(&mut self, source: u8) {
        // Really, we should be disabling access to anything but HRAM now,
        // but if the rom is nice then there shouldn't be an issue.
        self.dma_source = source;
        self.dma_cycles = gpu_timing::DMA_CYCLES;
    }

    fn update_dma (&mut self, ints: &mut Interrupts, mem: &mut Memory) {
        // There isn't one pending
        if self.dma_cycles == 0 { return; }

        self.dma_cycles -= 1;
        // Ready to actually perform DMA?
        if self.dma_cycles == 0 {
            let source = (self.dma_source as u16) << 4;

            for i in 0x00..=0x9F {
                let data = mem.read(ints, self, source + i);
                mem.write(ints, self, 0xFE00 + i, data);
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

        self.lx = (self.lx + 1) % gpu_timing::HTOTAL;

        let mode = self.status.get_mode();

        let new_mode = match mode {
            LcdMode::VBlank => {
                if self.lx == 0 {
                    self.ly = (self.ly + 1) % gpu_timing::VTOTAL;
                    self.run_ly_compare(ints);

                    if self.ly == 0 {
                        if self.status.oam_interrupt {
                            ints.raise_interrupt(InterruptReason::LCDStat);
                        }
                        LcdMode::OAMSearch
                    } else { mode }
                } else { mode }
            },
            _ => {
                match self.lx {
                    0 => {
                        self.ly += 1;
                        self.run_ly_compare(ints);
                        // Done with frame, enter VBlank
                        if self.ly == gpu_timing::VBLANK_ON {
                            self.enter_vblank(ints);
                            LcdMode::VBlank
                        } else { LcdMode::OAMSearch }
                    }
                    gpu_timing::HTRANSFER_ON => LcdMode::Transfer,
                    gpu_timing::HBLANK_ON => {
                        if self.status.hblank_interrupt {
                            ints.raise_interrupt(InterruptReason::LCDStat)
                        }
                        LcdMode::HBlank
                    },
                    _ => mode
                }
            }
        };
        self.status.set_mode(new_mode);

        // The first line takes longer to draw
        let line_start = gpu_timing::HTRANSFER_ON +
            if self.ly == 0 { 160 } else { 48 };

        if self.lx == line_start && self.status.get_mode() != LcdMode::VBlank {
            // Draw the current line
            // TODO: Move these draw_pixel calls into the mode switch above
            //       to allow mid-scanline visual effects
            for x in 0..(SCREEN_WIDTH as u8) {
                self.draw_pixel(ints, mem, x, self.ly);
            }
        }
    }

    fn draw_pixel (&mut self, ints: &Interrupts, mem: &Memory, x: u8, y: u8) {
        let ux = x as usize; let uy = y as usize;
        let idx = uy * SCREEN_WIDTH + ux;

        // We just always assume the background's enabled,
        // and there are no sprites or window
        let bg = self.get_background_colour_at(ints, mem, x, y);
        self.frame[idx] = bg;
    }

    fn get_background_colour_at (&self, ints: &Interrupts, mem: &Memory, x: u8, y: u8) -> GreyShade {
        let tilemap_base = if self.control.bg_tile_map_display_select {
            0x9C00
        } else { 0x9800 };

        // This is which tile ID our pixel is in
        let y16 = y.wrapping_add(self.scy) as u16; let x16 = x.wrapping_add(self.scx) as u16;
        let tx = x16 / 8; let ty = y16 / 8;
        let subx = (x16 % 8) as u8; let suby = y16 % 8;

        let byte_offset = ty * 32 + tx;

        let tile_id_raw = mem.read(ints, self, tilemap_base + byte_offset);
        let tile_id: u16;

        if self.control.bg_and_window_data_select {
            // 8000 addressing mode
            tile_id = tile_id_raw as u16;
        } else {
            // 8800 addressing mode
            // TODO: This might not be right
            if tile_id_raw < 128 {
                tile_id = (tile_id_raw as u16) + 255;
            } else { tile_id = tile_id_raw as u16 }
        }

        let tile_byte_offset = tile_id * 16;
        let tile_line_offset = tile_byte_offset + (suby * 2);

        // This is the line of the tile data that out pixel resides on
        let tiledata_base = 0x8000;

        let lower = mem.read(ints, self, tiledata_base + tile_line_offset);
        let upper = mem.read(ints, self, tiledata_base + tile_line_offset + 1);

        let shift_amnt = 7 - subx;
        let mask = 1 << shift_amnt;
        let u = (upper & mask) >> shift_amnt;
        let l = (lower & mask) >> shift_amnt;
        let pixel_colour_id = (u << 1) | l;

        let shift_2 = pixel_colour_id * 2;
        let shade = (self.bg_pallette & (0b11 << shift_2)) >> shift_2;

        GreyShade::from(shade)
    }

    // Will be used later for get_sprite_pixel
    fn get_sprites_on_line (&self, ints: &Interrupts, mem: &Memory, y: u8) -> Vec<Sprite> {
        let sprites = get_all_sprites(self, ints, mem);

        if self.control.obj_size {
            panic!("8x16 sprites are not supported!")
        }

        let on_line = vec![];
        for s in &sprites {
            
        }

        sprites
    }

    pub fn get_sfml_frame (&self) -> [u8; SCREEN_RGBA_SLICE_SIZE] {
        let mut out_array = [0; SCREEN_RGBA_SLICE_SIZE];
        for i in 0..SCREEN_BUFFER_SIZE {
            let start = i * 4;
            match &self.finished_frame[i] {
                GreyShade::White => {
                    out_array[start] = 0xDD;
                    out_array[start + 1] = 0xDD;
                    out_array[start + 2] = 0xDD;
                    out_array[start + 3] = 0xFF;
                },
                GreyShade::LightGrey => {
                    out_array[start] = 0xAA;
                    out_array[start + 1] = 0xAA;
                    out_array[start + 2] = 0xAA;
                    out_array[start + 3] = 0xFF;
                },
                GreyShade::DarkGrey => {
                    out_array[start] = 0x88;
                    out_array[start + 1] = 0x88;
                    out_array[start + 2] = 0x88;
                    out_array[start + 3] = 0xFF;
                },
                GreyShade::Black => {
                    out_array[start] = 0x55;
                    out_array[start + 1] = 0x55;
                    out_array[start + 2] = 0x55;
                    out_array[start + 3] = 0xFF;
                }
            }
        }
        out_array
    }

    pub fn new () -> Gpu {
        let empty_frame = [GreyShade::White; SCREEN_BUFFER_SIZE];
        Gpu {
            frame: empty_frame,
            finished_frame: empty_frame.clone(),
            scy: 0, scx: 0, ly: 0, lx: 0, lyc:0, wy: 0, wx: 0,
            bg_pallette: 0, sprite_pallete_1: 0, sprite_pallete_2: 0,
            status: LcdStatus::new(),
            control: LcdControl::new(),
            oam: Ram::new(OAM_SIZE),
            dma_source: 0, dma_cycles: 0
        }
    }
}
