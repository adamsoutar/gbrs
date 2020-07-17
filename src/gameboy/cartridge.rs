// Parses the cartridge header
#[derive(Clone)]
pub struct Cartridge {
    pub title: String,
    pub cart_type: u8,
    pub rom_size: u8,
    pub ram_size: u8
}

impl Cartridge {
    pub fn parse(buffer: &Vec<u8>) -> Cartridge {
        let title = get_title(buffer);
        let cart_type = buffer[0x0147];
        let rom_size = buffer[0x0148];
        let ram_size = buffer[0x0149];

        Cartridge {
            title, cart_type, rom_size, ram_size
        }
    }
}

fn get_title (buffer: &Vec<u8>) -> String {
    let mut out_buff = vec![];
    for i in 0x0134..=0x0143 {
        // A null byte terminates the title string
        if buffer[i] == 0 { break; }
        out_buff.push(buffer[i]);
    }
    String::from_utf8(out_buff)
        .expect("ROM title isn't valid UTF-8")
}
