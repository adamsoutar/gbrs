#[derive(Clone)]
pub struct StereoPanning {
    pub channel1_left: bool,
    pub channel1_right: bool,
    pub channel2_left: bool,
    pub channel2_right: bool,
    pub channel3_left: bool,
    pub channel3_right: bool,
    pub channel4_left: bool,
    pub channel4_right: bool,
}

impl From<u8> for StereoPanning {
    fn from(n: u8) -> StereoPanning {
        StereoPanning {
            channel4_left: (n & 0b1000_0000) > 0,
            channel3_left: (n & 0b0100_0000) > 0,
            channel2_left: (n & 0b0010_0000) > 0,
            channel1_left: (n & 0b0001_0000) > 0,
            channel4_right: (n & 0b0000_1000) > 0,
            channel3_right: (n & 0b0000_0100) > 0,
            channel2_right: (n & 0b0000_0010) > 0,
            channel1_right: (n & 0b0000_0001) > 0,
        }
    }
}

impl From<StereoPanning> for u8 {
    fn from(stereo: StereoPanning) -> u8 {
        (stereo.channel1_right as u8)
            | ((stereo.channel2_right as u8) << 1)
            | ((stereo.channel3_right as u8) << 2)
            | ((stereo.channel4_right as u8) << 3)
            | ((stereo.channel1_left as u8) << 4)
            | ((stereo.channel2_left as u8) << 5)
            | ((stereo.channel3_left as u8) << 6)
            | ((stereo.channel4_left as u8) << 7)
    }
}
