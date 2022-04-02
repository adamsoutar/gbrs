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
