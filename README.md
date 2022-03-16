# gbrs

A Rust GameBoy emulator!

<table>
  <tr>
    <th>Tetris</th><th>Zelda: Link's Awakening</th>
  </tr>
  <tr>
    <td><img src="assets/tetris.gif" /></td>
    <td><img src="assets/zelda.gif" /></td>
  </tr>
  <tr>
    <th>Super Mario Land</th><th>Super Mario Land 2</th>
  </tr>
  <tr>
    <td><img src="assets/mario.gif" /></td>
    <td><img src="assets/mario2.gif" /></td>
  </tr>
  <tr>
    <th>Galaga & Galaxian</th><th>Mortal Kombat</th>
  </tr>
  <tr>
    <td><img src="assets/galaga.gif" /></td>
    <td><img src="assets/mortalkombat.gif" /></td>
  </tr>
  <tr>
    <th>Pac-Man</th><th>Alleyway</th>
  </tr>
  <tr>
    <td><img src="assets/pacman.gif" /></td>
    <td><img src="assets/alleyway.gif" /></td>
  </tr>
  <tr>
    <th>Space Invaders</th><th>Road Rash</th>
  </tr>
  <tr>
    <td><img src="assets/spaceinvaders.gif" /></td>
    <td><img src="assets/roadrash.gif" /></td>
  </tr>
  <tr>
    <th>Donkey Kong</th><th>Frogger</th>
  </tr>
  <tr>
    <td><img src="assets/donkeykong.gif" /></td>
    <td><img src="assets/frogger.gif" /></td>
  </tr>
</table>

## Support

gbrs supports:

- Mid-scanline effects (required for games like Road Rash)
- "The Window" - a GPU feature required for Pac Man and Zelda
- Cycle-accurate CPU & counters
- Save files & saved games (Zelda & Super Mario Land 2 use these)
- Memory Board Controller 1 (MBCs are required for some more complex games)
- Memory Board Controller 2

& more!

## Progress so far

I'm still working on gbrs (and having a **_tonne_** of fun doing it!).

Some main things I'm working on

- Pokémon requires Memory Controller Board 3 support
- There's no support for sound right now

## Building from source

gbrs is not yet finished enough to distribute binaries, but if you want to try it out:

You'll need SFML set up, which you can find instructions for [here](https://github.com/jeremyletang/rust-sfml/wiki).

Afterwards, in a terminal, you can execute these commands, assuming you have a
[Rust](https://rustlang.org) toolchain installed.

```
git clone https://github.com/adamsoutar/gbrs
cd gbrs/sfml-gui
cargo run --release ROM_PATH
```

(Replace ROM_PATH with the path to a .gb file)

## Ports

gbrs is written to be ported to other platforms. Its default GUI for Windows,
macOS and Linux is just a module that it doesn't _have_ to use.

You can port [gbrs-core](./core) to almost anything - especially since it
supports running _without_ the Rust StdLib.

All a port needs to do is:

```rust
use gbrs_core::cpu::Cpu;

let mut gameboy = Cpu::from_rom_bytes(
  include_bytes!("./tetris.gb").to_vec()
);

// Each frame:
gameboy.step_one_frame();
draw_screen(&gameboy.gpu.finished_frame);
// (where draw_screen is a platform-specific function left to the reader)
```

---

<h6 align="center">By Adam Soutar</h6>
