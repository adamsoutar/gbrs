#[derive(Clone, Copy)]
pub struct Colour {
    pub red: u8,
    pub green: u8,
    pub blue: u8
}

impl Colour {
    // Colour space conversion algo from
    //   https://gamedev.stackexchange.com/a/196834
    pub fn from_16_bit_colour (val: u16) -> Colour {
        let mut red = ((val % 32) * 8) as u8;
        red = red + red / 32;
        let mut green = (((val / 32) % 32) * 8) as u8;
        green = green + green / 32;
        let mut blue = (((val / 1024) % 32) * 8) as u8;
        blue = blue + blue / 32;
        Colour { red, green, blue }
    }

    pub fn new(red: u8, green: u8, blue: u8) -> Colour {
        Colour { red, green, blue }
    }
}
