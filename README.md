# useless-looper
A toy audio looper written in Rust using [GPUI](https://github.com/zed-industries/zed/tree/main/crates/gpui) for UI and [kittyaudio](https://github.com/zeozeozeo/kittyaudio) for audio. Main purpose is to learn the GPUI library. Very WIP.

## Run
```
cargo run --release
```
## Usage
Scroll left/right to change the position of the playhead.
CTRL + Scroll to change the size of the loop.
ALT + up/down scroll to change the pitch of the loop.
You can drop audio files directly into the window to load them.
Currently, waveforms are shown only for the WAV format.
