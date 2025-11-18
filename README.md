# magtag_weatherstation

Compact no_std Rust firmware that fetches weather data and renders it on an Adafruit MagTag e-paper device (ESP32‑S2, esp-hal ecosystem).

The firmware:

- Initializes the SSD1680 e-paper display over SPI
- Brings up Wi‑Fi using `esp-radio` and `embassy-net`
- Fetches weather data and renders it with `embedded-graphics`/`embedded-text`
- Enters deep sleep to conserve battery between updates

This project was heavily inspired by Adafruit's MagTag weather example:

- https://learn.adafruit.com/magtag-weather
- https://github.com/adafruit/Adafruit_Learning_System_Guides/blob/main/MagTag/MagTag_Weather/openmeteo/code.py

## Prerequisites

- Rust with the Espressif toolchain for Xtensa (see esp-rs getting started):
	https://docs.espressif.com/projects/rust/book/getting-started/toolchain.html
- A flasher tool for the xtensa-esp32s2 target: 
	- `espflash`
    
- To build the project and connect to wifi, you need to set the environment variables SSID, and PASSWORD. The project uses the env! macro to set these static values at compile time in the src/network.rs file.

## Notable dependencies

All dependencies are declared in `Cargo.toml`. Highlights:

- `esp-hal`, `esp-rtos`, `esp-radio` — ESP32‑S2 HAL and runtime
- `embassy-net`, `embassy-executor` — async network stack and executor
- `ssd1680` — SSD1680 e-paper driver for Adafruit MagTag ePaper display (EPD) (git dependency)
- `embedded-graphics`, `embedded-text` — drawing and text layout

## Build

Build the release binary for the target from the repository root:

```bash
# build optimized firmware
cargo build --release
```


## Flashing

By default this project is wired to use the espflash runner.

```bash
# cargo will build the firmware and call espflash to flash the firmware
cargo run --release
```

## Logs / Serial

Firmware emits logs via the `log` facade over serial. To follow runtime output, open a serial terminal at 115200 baud (or the configured baud rate).
**Important Note: This project does not setup a USB serial device on the esp32s2 to write serial output over USB. To monitor the serial output, you will need to connect a USB-serial device to the UART RX/TX pinouts on the back of the MagTag.**

## Project layout

- `src/bin/main.rs` — application entry point: hardware init, network stack, and high-level flow
- `src/lib.rs` — crate exports and shared types
- `src/display.rs` — display initialization and drawing helpers
- `src/network.rs` — Wi‑Fi and networking helpers
- `src/sleep.rs` — deep sleep helper
- `src/weather/` — weather model, API client and parsing logic

## Contributing

Contributions are welcome, but please do not change build targets or hardware assumptions.

## License

MIT
