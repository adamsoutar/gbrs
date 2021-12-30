# Notes

If you lock a Mac (CMD+CTRL+Q) while gbrs is running from `cargo run --release`,
`cargo` will segfault.

Dr. Mario locks up, seemingly looking for the GPU to enter OAMSearch status,
but when it checks we're always reporting that we're in VBlank