#[macro_export]
macro_rules! combine_u8(
    ($x:expr, $y:expr) => (
        (($x as u16) << 8) | $y as u16
    )
);
#[macro_export]
macro_rules! split_u16(
    ($n:expr) => ({
        let b1 = ($n & 0x00FF) as u8;
        let b2 = (($n & 0xFF00) >> 8) as u8;
        (b1, b2)
    })
);
#[macro_export]
macro_rules! set_bit(
    ($number:expr, $bit_index:expr, $bit:expr) => (
        $number = ($number & !(1 << $bit_index)) | $bit << $bit_index;
    )
);

#[macro_export]
macro_rules! log {
    ($($a:expr),*) => {
        {
            #[cfg(not(feature = "std"))]
            use alloc::format;
            ($crate::callbacks::CALLBACKS.lock().log)(&format!($($a,)*)[..])
        }
    };
}

// Macro for bit-matching
// https://www.reddit.com/r/rust/comments/2d7rrj/comment/cjo2c7t/?context=3
#[macro_export]
macro_rules! compute_mask {
    (0) => {
        1
    };
    (1) => {
        1
    };
    (_) => {
        0
    };
}
#[macro_export]
macro_rules! compute_equal {
    (0) => {
        0
    };
    (1) => {
        1
    };
    (_) => {
        0
    };
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
