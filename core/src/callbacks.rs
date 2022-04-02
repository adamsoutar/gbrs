// This allows ports to register functions for things like logging as well as
// saving/loading battery-backed RAM.
use crate::constants::SOUND_BUFFER_SIZE;

#[cfg(not(feature = "std"))]
use alloc::{vec, vec::Vec};
#[cfg(feature = "std")]
use std::{fs, io::Read, path::PathBuf, time::Instant};

pub type LogCallback = fn(log_str: &str);
pub type SaveCallback =
    fn(game_name: &str, rom_path: &str, save_data: &Vec<u8>);
pub type LoadCallback =
    fn(game_name: &str, rom_path: &str, expected_size: usize) -> Vec<u8>;
// Should return any number in milliseconds. Used to wait for save debouncing
pub type TimestampCallback = fn() -> usize;
pub type SoundCallback = fn(sound_buffer: &[i16; SOUND_BUFFER_SIZE]);

#[derive(Clone)]
pub struct Callbacks {
    pub log: LogCallback,
    pub save: SaveCallback,
    pub load: LoadCallback,
    pub get_ms_timestamp: TimestampCallback,
    pub play_sound: SoundCallback
}

#[cfg(feature = "std")]
lazy_static! {
    static ref PROGRAM_START: Instant = Instant::now();
}

#[cfg(feature = "std")]
fn get_save_file_path(rom_path: &str) -> String {
    let mut sav_path = PathBuf::from(rom_path);
    sav_path.set_extension("sav");

    sav_path.to_string_lossy().to_string()
}

#[cfg(feature = "std")]
pub static mut CALLBACKS: Callbacks = Callbacks {
    log: |log_str| println!("{}", log_str),
    save: |_game_name, rom_path, save_data| {
        let save_path = get_save_file_path(rom_path);
        fs::write(&save_path, save_data).expect("Failed to write save file");
    },
    load: |_game_name, rom_path, expected_size| {
        let save_path = get_save_file_path(rom_path);
        let mut buffer = vec![];
        let file_result = fs::File::open(save_path);

        if let Ok(mut file) = file_result {
            file.read_to_end(&mut buffer)
                .expect("Unable to read save file");
            buffer
        } else {
            // The save file likely does not exist
            vec![0; expected_size]
        }
    },
    get_ms_timestamp: || PROGRAM_START.elapsed().as_millis() as usize,
    play_sound: |_sound_buffer| {}
};

#[cfg(not(feature = "std"))]
pub static mut CALLBACKS: Callbacks = Callbacks {
    log: |_log_str| {},
    save: |_game_name, _rom_path, _save_data| {},
    load: |_game_name, _rom_path, expected_size| vec![0; expected_size],
    // Acting like time never advances is fine. We just will never save
    get_ms_timestamp: || 0,
    play_sound: |_sound_buffer| {}
};

pub unsafe fn set_callbacks(cbs: Callbacks) {
    CALLBACKS = cbs;
}
