pub fn combine_u8 (b1: u8, b2: u8) -> u16 {
    let bu1 = b1 as u16;
    let bu2 = b2 as u16;
    (bu1 << 8) | bu2
}
pub fn split_u16 (v: u16) -> (u8, u8) {
    let b1 = (v & 0x00FF) as u8;
    let b2 = ((v & 0xFF00) >> 8) as u8;
    (b1, b2)
}
pub fn set_bit (number: &mut u8, bit_index: u8, bit: u8) {
    // Clear the bit
    *number &= !(1 << bit_index);
    // Set it
    *number |= bit << bit_index;
}

// Frontends can give us their own logging method
// This is especially handy when using no_std due to being without println
pub type LogCallback = fn(&str);

#[cfg(feature = "std")]
fn default_log_callback (log_string: &str) {
    println!("{}", log_string);
}
// NOTE: By default we do not actually log in no_std mode. The consumer may
//   modify the LOG_CALLBACK lazy static to change this fact
#[cfg(not(feature = "std"))]
fn default_log_callback (_log_string: &str) {

}

pub static mut LOG_CALLBACK: LogCallback = default_log_callback;

// This is by no means thread-safe, but gbrs does not multithread,
// so it only ever talks to the log callback from the main thread.
// In addition, the callback is likely only modified once, at the start
// of the program, if at all.
// Also, the main gbrs-core crate never modifies this. A port MAY choose to.
pub unsafe fn set_log_callback (new_cb: LogCallback) {
    LOG_CALLBACK = new_cb;
}

#[macro_export]
macro_rules! log {
    ($($a:expr),*) => {
        {
            #[cfg(not(feature = "std"))]
            use alloc::format;
            unsafe { crate::helpers::LOG_CALLBACK(&format!($($a,)*)[..]) }
        }
    };  
}

// Macro for bit-matching
// https://www.reddit.com/r/rust/comments/2d7rrj/comment/cjo2c7t/?context=3
#[macro_export]
macro_rules! compute_mask {
    (0) => { 1 };
    (1) => { 1 };
    (_) => { 0 };
}
#[macro_export]
macro_rules! compute_equal {
    (0) => { 0 };
    (1) => { 1 };
    (_) => { 0 };
}
#[macro_export]
macro_rules! bitmatch(
    ($x: expr, ($($b: tt),*)) => ({
        let mut mask = 0;
        let mut val = 0;
        $(
            mask = (mask << 1) | compute_mask!($b);
            val = (val << 1) | compute_equal!($b);
        )*
        ($x & mask) == val
    });
);
