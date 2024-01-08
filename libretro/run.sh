cargo build &&
RUST_BACKTRACE=1 /Applications/RetroArch.app/Contents/MacOS/RetroArch --verbose -L ./target/debug/libgbrs_libretro.dylib ../roms/$1.gb
