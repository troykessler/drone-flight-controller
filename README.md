# Drone Flight Controller

Requires Rust via [rustup](https://rustup.rs/) and [picotool](https://github.com/raspberrypi/picotool) on your PATH.

Build for RP2040:

```sh
cargo build --release
```

Hold **BOOTSEL** while connecting the board via USB, then release it. Flash and start:

```sh
cargo run --release
```

The Rust target and picotool runner are configured in this project.
