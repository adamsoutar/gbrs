#[derive(Clone, Copy)]
pub enum GreyShade {
    White,
    LightGrey,
    DarkGrey,
    Black
}

pub struct LcdControl {
    display_enable: bool,
    tile_map_display_select: bool,
    window_enable: bool,
    bg_and_window_data_select: bool,
    obj_size: bool,
    obj_enable: bool,
    bg_display: bool
}
impl From<u8> for LcdControl {
    fn from(n: u8) -> LcdControl {
        LcdControl {
            bg_display: (n & 1) == 1,
            obj_enable: (n & 0b10) == 0b10,
            obj_size: (n & 0b100) == 0b100,
            bg_and_window_data_select: (n & 0b1000) == 0b1000,
            window_enable: (n & 0b10000) == 0b10000,
            tile_map_display_select: (n & 0b100000) == 0b100000,
            display_enable: (n & 0b1000000) == 0b1000000
        }
    }
}
impl From<LcdControl> for u8 {
    fn from (lcd: LcdControl) -> u8 {
        lcd.bg_display as u8 |
        (lcd.obj_enable as u8) << 1 |
        (lcd.obj_size as u8) << 2 |
        (lcd.bg_and_window_data_select as u8) << 3 |
        (lcd.window_enable as u8) << 4 |
        (lcd.tile_map_display_select as u8) << 5 |
        (lcd.display_enable as u8) << 6
    }
}

#[derive(Clone, Copy)]
pub struct LcdStatus {
    lyc: bool,
    oam_interrupt: bool,
    vblank_interrupt: bool,
    hblank_interrupt: bool,
    coincidence_flag: bool,
    mode_flag: u8
}
impl LcdStatus {
    pub fn new () -> LcdStatus {
        LcdStatus::from(0)
    }
}
impl From<u8> for LcdStatus {
    fn from(n: u8) -> LcdStatus {
        LcdStatus {
            lyc: (n & 0b1000000) == 0b1000000,
            oam_interrupt: (n & 0b100000) == 0b100000,
            vblank_interrupt: (n & 0b10000) == 0b10000,
            hblank_interrupt: (n & 0b1000) == 0b1000,
            coincidence_flag: (n & 0b100) == 0b100,
            mode_flag: n & 0b11
        }
    }
}
impl From<LcdStatus> for u8 {
    fn from(lcd: LcdStatus) -> u8 {
        lcd.mode_flag |
        (lcd.coincidence_flag as u8) << 2 |
        (lcd.hblank_interrupt as u8) << 3 |
        (lcd.vblank_interrupt as u8) << 4 |
        (lcd.oam_interrupt as u8) << 5 |
        (lcd.lyc as u8) << 6
    }
}
