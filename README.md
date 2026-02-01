
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

## Features

- Graphical output (default): controlled by the `graphical` feature flag (used via `cfg(feature = "graphical")`). 
- Text fallback: when graphical output is disabled the firmware renders a compact text summary.

## Prerequisites

- Rust with the Espressif toolchain for Xtensa (see esp-rs getting started):
	https://docs.espressif.com/projects/rust/book/getting-started/toolchain.html
- A flasher tool for the xtensa-esp32s2 target (e.g. `espflash`).
- The project reads Wi‑Fi credentials at compile time via `env!` in `src/network.rs`; set `SSID` and `PASSWORD` when building if your configuration requires it.
- Make necessary changes in the config.rs file.


## Project layout

- `src/bin/main.rs` — application entry point: hardware init, network stack, and high-level flow
- `src/lib.rs` — crate exports and shared types
- `src/config.rs` — compile-time configuration constants
- `src/error.rs` — application error types
- `src/network.rs` — Wi‑Fi and networking helpers
- `src/time.rs` — date/time helpers and formatters
- `src/sleep.rs` — deep sleep helper
- `src/weather/` — weather subsystem
	- `src/weather/mod.rs` — high-level weather helpers and `fetch_weather`/`draw_weather`
	- `src/weather/api.rs` — HTTP request builder and fetch helper
	- `src/weather/http.rs` — minimal HTTP client helpers used by `api.rs`
	- `src/weather/model.rs` — serde models for the Open-Meteo API (uses `heapless::String`)
	- `src/weather/display.rs` — textual & graphical drawing glue for weather data
	- `src/weather/graphics.rs` — bitmap/icon drawing helpers (graphical feature)
- `resources/` — bitmap images and compiled raw image assets
- `scripts/` — helper scripts used to generate raw image assets

## Build

Build the release firmware:

```bash
# build with default features
cargo build --release

# build text-only firmware (no graphical feature)
cargo build --release --no-default-features
```

## Flashing

By default this project uses the `espflash` runner. From the repo root:

```bash
# cargo will build the firmware and call espflash to flash the firmware
cargo run --release
```

## Logs / Serial

Firmware emits logs via the `log` facade over serial. To follow runtime output, open a serial terminal at 115200 baud (or the configured baud rate).
Important Note: The `log` facade does NOT support USB serial. To monitor serial output, you will need to connect a USB-serial device to the UART RX/TX pinouts on the back of the MagTag.

## Contributions

Contributions are welcome. Please keep changes focused to features or fixes and avoid altering the hardware assumptions unless explicitly discussed.

## License

MIT
