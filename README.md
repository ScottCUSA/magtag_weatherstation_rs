# MagTag Weather Station

A `no_std` Rust firmware for the 2025 revision of the [Adafruit MagTag](https://www.adafruit.com/product/4800) with the SSD1680 controller. It displays weather information on the e-paper display. Built with the `esp-hal` ecosystem for ESP32-S2, this project demonstrates async/await patterns with Embassy, network connectivity with `esp-radio`, and e-paper graphics rendering.

## Features

- **E-Paper Display**: Drives the SSD1680 2.9" grayscale e-paper display (296x128 pixels) over SPI
- **WiFi Connectivity**: Connects to WiFi using `esp-radio` and `embassy-net` with async networking
- **Weather Data**: Fetches weather forecasts from the [Open-Meteo API](https://open-meteo.com/)
- **Graphical UI**: Renders weather data with icons, text, and formatting using `embedded-graphics` and `embedded-text`
- **Low Power**: Enters deep sleep between updates to conserve battery (24-hour update cycle by default)
- **Error Handling**: Displays error messages on the e-paper screen when issues occur
- **No Standard Library**: Runs entirely in `no_std` environment with custom allocator

## Hardware Requirements

- [Adafruit MagTag](https://www.adafruit.com/product/4800) - 2025 Edition with SSD1680 (ESP32-S2 based e-paper display) 
- USB cable for programming and power
- Optional: USB-to-serial adapter for debugging (see Serial Logging section)

## Software Prerequisites

1. **Rust Toolchain**: Install Rust with the Espressif Xtensa toolchain
   - Follow the [esp-rs Getting Started guide](https://docs.esp-rs.org/book/installation/index.html)
   - Requires the `xtensa-esp32s2-none-elf` target

2. **Flashing Tool**: Install `espflash`
   ```bash
   cargo install espflash
   ```

3. **Environment Variables**: Set WiFi credentials as environment variables
   ```bash
   export SSID="YourNetworkName"
   export PASSWORD="YourNetworkPassword"
   ```

## Configuration

Edit [src/config.rs](src/config.rs) to customize:

- `OPENMETEO_LATITUDE` / `OPENMETEO_LONGITUDE` — Your location coordinates
- `OPENMETEO_TIMEZONE` — Your timezone (e.g., "America/Denver")
- `TEMPERATURE_UNIT` — "fahrenheit" or "celsius"
- `WIND_SPEED_UNIT` — "mph" or "kmh"

WiFi credentials are read from environment variables at compile time:
- `WIFI_SSID` from `$SSID`
- `WIFI_PASSWORD` from `$PASSWORD`

## Project Structure

```
src/
├── bin/
|   ├── tasks/
|   |   ├── display.rs   # Display Embassy tasks
|   |   ├── network.rs   # Network Embassy tasks
|   |   ├── mod.rs       # tasks module exports
|   |   └── weather.rs   # Weather Embassy tasks
│   └── main.rs          # Application entry point and main loop
├── config.rs            # Configuration constants (WiFi, location, units)
├── display.rs           # E-paper display initialization and rendering
├── error.rs             # Application error types
├── graphics.rs          # Drawing helpers and text rendering
├── http.rs              # HTTP client for API requests
├── lib.rs               # Library root and module exports
├── network.rs           # WiFi and network stack setup
├── sleep.rs             # Deep sleep functionality
├── time.rs              # Date/time formatting utilities
└── weather/
    ├── api.rs           # Open-Meteo API client
    ├── mod.rs           # Weather module exports
    ├── model.rs         # Serde data models for API responses
    └── ui.rs            # Weather UI layout and rendering

resources/               # Image assets (bitmaps and raw format)
scripts/                 # Python scripts to convert images to raw format
```

## Building

```bash
# Build release firmware (optimized for size and speed)
cargo build --release

# Build debug firmware (faster compilation, larger binary)
cargo build
```

The project uses LTO and size optimization (`opt-level = 's'`) for release builds.

## Flashing

The project is configured to use `espflash` as the default runner:

```bash
# Build and flash in one command
cargo run --release

# Or flash a pre-built binary
espflash flash --monitor --chip esp32s2 target/xtensa-esp32s2-none-elf/release/magtag_weatherstation
```

## Runtime Behavior

1. **Startup**: Initializes peripherals, display, and WiFi
2. **Network**: Connects to WiFi and obtains IP via DHCP
3. **Fetch**: Retrieves weather data from Open-Meteo API
4. **Display**: Renders weather information on e-paper screen
5. **Sleep**: Enters deep sleep for 24 hours (or 5 minutes on error)
6. **Repeat**: Wakes up and repeats the cycle

## Serial Logging

The firmware outputs log messages via UART at 115200 baud using the `log` facade and `esp-println`.

**Important**: The MagTag's USB port does **not** expose serial output. To view logs, you must:

1. Connect a USB-to-serial adapter to the UART pins on the back of the MagTag
2. Open a serial terminal at 115200 baud
3. Logs are controlled by the `ESP_LOG` environment variable (set in `.cargo/config.toml`)

## Dependencies

Key dependencies include:

- **esp-hal** (1.0.0) — Hardware abstraction layer for ESP32-S2
- **esp-rtos** (0.2.0) — RTOS integration with Embassy executor
- **esp-radio** (0.17.0) — WiFi radio driver
- **embassy-net** — Async TCP/IP networking stack
- **ssd1680** — E-paper display driver (custom fork)
- **embedded-graphics** — 2D graphics library
- **serde** / **serde-json-core** — JSON parsing in `no_std`
- **heapless** — Stack-allocated collections

## Heap Configuration

The firmware uses a custom heap allocator with 64KB allocated from reclaimed RAM. This is sufficient for network buffers, HTTP responses, and display rendering.

## Troubleshooting

### Build Errors

- Ensure `SSID` and `PASSWORD` environment variables are set
- Verify Xtensa Rust toolchain is installed: `rustup target list | grep xtensa`
- Check that `espflash` is in your PATH: `espflash --version`

### Network Issues

- Verify WiFi credentials in environment variables
- Check that your router supports 2.4GHz (ESP32-S2 doesn't support 5GHz)
- Monitor serial output to see connection status

### Display Issues

- Ensure SPI pins are correctly connected
- Check that the display driver is compatible with your MagTag hardware revision
- Look for error messages on the display itself

## Inspiration

This project was inspired by Adafruit's [MagTag Weather Example](https://learn.adafruit.com/magtag-weather) and demonstrates how to build similar functionality in pure Rust with `no_std`.

## Architecture
These diagrams are derived from `src/bin/main.rs` and the tasks in `src/bin/tasks/`.

This diagram shows what `main` spawns during initialization (no runtime messaging).

```mermaid
sequenceDiagram
   participant Main as `main` (bin/main.rs)
   participant Executor as Embassy Executor / Spawner

   Main->>Executor: spawner.spawn(network::wifi_task(controller))
   Main->>Executor: spawner.spawn(network::net_runner_task(runner))
   Main->>Executor: spawner.spawn(network::net_validator_task(stack))
   Main->>Executor: spawner.spawn(weather::weather_fetcher_task(stack))
   Main->>Executor: spawner.spawn(display::display_task(..., rtc))

   Note over Main,Executor: After spawning, `main` yields to the executor loop
```


The diagram below shows the main tasks and channels and signals used to communicate between them.

```mermaid
sequenceDiagram
   participant NetValidator as `network::net_validator_task`
   participant Weather as `weather::weather_fetcher_task`
   participant Display as `display::display_task`
   participant Sleep as `sleep::enter_deep_sleep_secs`
   
   NetValidator->>NetValidator: wait for link and IP
   alt IP acquired
      NetValidator->>Weather: signal `NETWORK_READY`
   else  Link or IP timeout or failure
      NetValidator->>Display: send `NETWORK_ERROR` (Channel<String>)
   end

   alt Fetch success
      Weather->>Display: send `WEATHER_CHANNEL` (Channel<OpenMeteoResponse>)
   else Fetch failure (after retries)
      Weather->>Display: send `NETWORK_ERROR` (Channel<String>)
   end

   Display->>Display: select(NETWORK_ERROR.receive(), WEATHER_CHANNEL.receive())
   alt Received `NETWORK_ERROR`
      Display->>Display: `display_error_text(...)`
      Display->>Sleep: `enter_deep_sleep_secs(rtc, SLEEP_ON_ERROR_SECS)`
   else Received `WEATHER_CHANNEL` data
      Display->>Display: `display_weather(...)`
      Display->>Sleep: `enter_deep_sleep_secs(rtc, SLEEP_ON_SUCCESS_SECS)`
   end

```

**Legend**
- `NETWORK_READY`: `Signal<()>` used by `net_validator_task` to notify `weather_fetcher_task`.
- `WEATHER_CHANNEL`: `Channel<OpenMeteoResponse, 1>` used by `weather_fetcher_task` -> `display_task`.
- `NETWORK_ERROR`: `Channel<heapless::String<128>, 1>` used to report link/IP/fetch errors to `display_task`.

This diagram mirrors the code in `src/bin/main.rs` and the tasks in `src/bin/tasks/`.

## Contributing

Contributions are welcome! Please:

- Keep changes focused on specific features or fixes
- Maintain `no_std` compatibility
- Test on actual MagTag hardware when possible
- Follow existing code style and patterns

## License

MIT