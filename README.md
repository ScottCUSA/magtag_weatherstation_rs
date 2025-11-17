# magtag_weatherstation

Compact Rust firmware to fetch weather data and display it on an Adafruit MagTag e-paper device using the ESP32-S2 family and `esp-hal` ecosystem.

This repository contains a small no_std async application that:
- initializes the display via SPI (SSD1680 driver),
- brings up Wi-Fi using `esp-radio`/`embassy-net`,
- fetches weather information and renders it with `embedded-graphics`,
- then puts the device into deep sleep to conserve power.

### Prerequisites

- Rust with the `esp` toolchain installed. https://docs.espressif.com/projects/rust/book/getting-started/toolchain.html
- Recommended tools for flashing: `cargo-espflash` or `espflash` (or any other flasher supporting the xtensa-esp32s2 toolchain).

### Dependencies

All runtime dependencies are tracked in `Cargo.toml`. Notable crates:

- `esp-hal`, `esp-rtos`, `esp-radio` — low-level ESP32-S2 HAL and runtime
- `embassy-net`, `embassy-executor` — async network stack and executor
- `ssd1680` (custom git dependency) — e-paper display driver
- `embedded-graphics`, `embedded-text` — rendering text/graphics for the display

### Building

This project builds for the ESP32-S2 target. From the repository root:

```bash
# Build in release with optimizations appropriate for embedded
cargo build --release
```

### Flashing

The project is configured to use espflash as the runner

```bash
# flash the release binary (adjust port and target as needed)
cargo run --release
```

Alternatively use `espflash` or the vendor tooling you prefer. See esp-rs/espflash docs for details on partitions, bootloader, and arguments.

### Running / Logs

The firmware uses `log` to output informational messages on serial. Open a serial terminal at 115200 baud (or the rate configured in your environment) to read logs while the device boots.

### Device behaviour

- On startup the firmware initializes peripherals (SPI, display), configures Wi-Fi and the network stack, then fetches weather data and draws it to the e-paper display.
- On success the device enters deep sleep for ~24 hours (see `SLEEP_ON_SUCCESS_SECS` in `src/bin/main.rs`). On failure it sleeps for a shorter retry interval.

### Project layout

- `src/bin/main.rs` - application entrypoint: hardware init, network stack, spawn tasks and drive high level flow
- `src/lib.rs` - no_std crate module exports
- `src/display.rs` - display initialization and drawing helpers
- `src/network.rs` - Wi-Fi/network tasks and helpers
- `src/sleep.rs` - deep sleep helper
- `src/weather/` - weather model and fetch logic

### Contributing

If you want to contribute, please:

1. Fork the repo and open a PR.
2. Keep changes small and focused.
3. If adding features that change behavior or build targets, include build/test instructions.

### License

MIT

