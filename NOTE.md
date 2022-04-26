# Notes

If you lock a Mac (CMD+CTRL+Q) while gbrs is running from `cargo run --release`,
`cargo` will segfault.

Dr. Mario locks up, seemingly looking for the GPU to enter OAMSearch status,
but when it checks we're always reporting that we're in VBlank

F-1 Race seems to run too slow - maybe the timers aren't right?

Donkey Kong's audio is waaaaay too slow - again a timer issue?

Space Invaders and Zelda seem to have the same issue where they can only make
certain APU Channel 4 sounds once - Space Invaders only makes one shot fire
noise, and Zelda only makes one sword slash noise. I think it might be because
I haven't implemented _reading_ from APU channel addresses.

## Optimisation ideas

In Memory::Step, which is 10% of runtime according to `cargo-flamegraph`, can
probably have its loops unrolled slightly. Instead of running per-cycle, we
could probably step timers and the like by doing addition rather than repeated
increments. ✅

MBCs probably do not need to be stepped per-cycle either. They can likely be
stepped per frame, or _even per second_ or something, and still be fine.
This step is only for save files and real-time clocks (not implemented).
MBC::Step _may_ currently be slow due to MBCs being allocated on the heap and
using indirection due to traits. - This turned out not to be very important.
Even not stepping an MBC at all barely impacts performance.

There may be optimisation to be found in the fact that, if we have a sprite
pixel, there is no need to go and calculate a background pixel colour. - This
isn't terribly useful as sprites don't cover tonnes of the screen.

The screen buffer likely does not need to be fully copied every frame.
Since we're not at all multi-threaded (yet?), frame data will not be modified
during rendering. - This also doesn't seem to be much quicker.
