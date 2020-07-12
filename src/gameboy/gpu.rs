use crate::gameboy::constants::*;

pub struct Gpu {
    pub frame: Vec<u8>
}

impl Gpu {
    pub fn get_sfml_frame (&self) -> [u8; SCREEN_RGBA_SLICE_SIZE] {
        [255; SCREEN_RGBA_SLICE_SIZE]
    }

    pub fn new () -> Gpu {
        Gpu {
            frame: vec![0; SCREEN_VEC_SIZE]
        }
    }
}
