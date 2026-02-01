use alloc::string::ToString;
use embedded_hal_bus::spi::ExclusiveDevice;
use esp_hal::{
    delay::Delay,
    gpio::{Input, Output},
    spi::master::Spi,
};
use ssd1680::displays::adafruit_thinkink_2in9::{Display2in9Gray2, ThinkInk2in9Gray2};
use ssd1680::prelude::*;

use crate::{
    error::{AppError, Result},
    weather::{
        graphics::{draw_text, draw_weather_station_view},
        model::OpenMeteoResponse,
    },
};

pub fn display_weather(
    weather_data: OpenMeteoResponse,
    spi_device: &mut ExclusiveDevice<Spi<'static, esp_hal::Blocking>, Output<'static>, Delay>,
    busy: Input<'static>,
    dc: Output<'static>,
    rst: Output<'static>,
) -> Result<()> {
    log::info!("Drawing graphical display");
    let mut display = Display2in9Gray2::new();
    match draw_weather_station_view(&weather_data, &mut display) {
        Ok(_) => (),
        Err(e) => {
            log::error!("Failed to draw weather station view: {:?}", e);
            return Err(e);
        }
    }
    display_buffer(&display, spi_device, busy, dc, rst)
}

pub fn display_buffer(
    buffer: &Display2in9Gray2,
    spi_device: &mut ExclusiveDevice<Spi<'static, esp_hal::Blocking>, Output<'static>, Delay>,
    busy: Input<'static>,
    dc: Output<'static>,
    rst: Output<'static>,
) -> Result<()> {
    // Create display with SPI interface
    let mut epd = ThinkInk2in9Gray2::new(spi_device, busy, dc, rst).map_err(|e| {
        log::error!("Failed to create e-paper display: {:?}", e);
        AppError::DisplayError
    })?;

    // Initialize the display
    epd.begin(&mut Delay::new()).map_err(|e| {
        log::error!("Failed to initialize e-paper display: {:?}", e);
        AppError::DisplayError
    })?;
    log::info!("E-paper display initialized");

    log::info!("Drawing to display");
    // Transfer and display the buffer on the display
    epd.update_gray2_and_display(buffer.high_buffer(), buffer.low_buffer(), &mut Delay::new())
        .map_err(|e| {
            log::error!("Failed to update e-paper display: {:?}", e);
            AppError::DisplayError
        })?;
    log::info!("updated display successfully");
    Ok(())
}

pub fn display_text(
    text: &str,
    spi_device: &mut ExclusiveDevice<Spi<'static, esp_hal::Blocking>, Output<'static>, Delay>,
    busy: Input<'static>,
    dc: Output<'static>,
    rst: Output<'static>,
) -> Result<()> {
    log::info!("Showing text on display: \n{}", text);
    let mut buffer = Display2in9Gray2::new();
    draw_text(text, 0, 0, 296, 0, &mut buffer)?;
    display_buffer(&buffer, spi_device, busy, dc, rst)
}

/// Show an error message on the display.
pub fn display_error_text(
    msg: &str,
    spi_device: &mut ExclusiveDevice<Spi<'static, esp_hal::Blocking>, Output<'static>, Delay>,
    busy: Input<'static>,
    dc: Output<'static>,
    rst: Output<'static>,
) {
    let _ = display_text(msg, spi_device, busy, dc, rst);
}

/// Show an `AppError` message on the display.
pub fn display_app_error(
    err: &AppError,
    spi_device: &mut ExclusiveDevice<Spi<'static, esp_hal::Blocking>, Output<'static>, Delay>,
    busy: Input<'static>,
    dc: Output<'static>,
    rst: Output<'static>,
) {
    display_error_text(&err.to_string(), spi_device, busy, dc, rst);
}
