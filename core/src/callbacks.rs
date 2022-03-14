// This allows ports to register functions for things like logging as well as
// saving/loading battery-backed RAM.
pub type LogCallback = fn(&str);

#[derive(Clone)]
pub struct Callbacks {
    pub log: LogCallback
}

#[cfg(feature = "std")]
pub static mut CALLBACKS: Callbacks = Callbacks {
    log: |log_str| println!("{}", log_str)
};

#[cfg(not(feature = "std"))]
pub static mut CALLBACKS: Callbacks = Callbacks {
    log: |_log_str| {}
};

pub unsafe fn set_callbacks (cbs: Callbacks) {
    CALLBACKS = cbs;
}