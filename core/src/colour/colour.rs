#[derive(Clone, Copy)]
pub struct Colour {
    pub red: u8,
    pub green: u8,
    pub blue: u8
}

impl Colour {
    pub fn from_16_bit_colour (val: u16) -> Colour {
        let red_5bit = (val & 0b0000_0000_0001_1111) as u8;
        let green_5bit = (val & 0b0000_0011_1110_0000 >> 5) as u8;
        let blue_5bit = (val & 0b0111_1100_0000_0000 >> 10) as u8;
        let red = red_5bit << 3 | red_5bit >> 2;
        let green = green_5bit << 3 | green_5bit >> 2;
        let blue = blue_5bit << 3 | blue_5bit >> 2;
        Colour { red, green, blue }
    }

    pub fn new(red: u8, green: u8, blue: u8) -> Colour {
        Colour { red, green, blue }
    }
}
