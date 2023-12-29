use super::colour::Colour;

pub fn white() -> Colour {
    Colour::new(0xDD, 0xDD, 0xDD)
}
pub fn light_grey() -> Colour {
    Colour::new(0xAA, 0xAA, 0xAA)
}
pub fn dark_grey() -> Colour {
    Colour::new(0x88, 0x88, 0x88)
}
pub fn black() -> Colour {
    Colour::new(0x55, 0x55, 0x55)
}

pub fn colour_from_grey_shade_id(id: u8) -> Colour {
    match id {
        0 => white(),
        1 => light_grey(),
        2 => dark_grey(),
        3 => black(),
        _ => panic!("Invalid grey shade id {}", id)
    }
}
