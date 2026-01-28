use embedded_graphics::{image::Image, pixelcolor::Gray2, prelude::*};
use embedded_hal_bus::spi::ExclusiveDevice;
use esp_hal::{
    delay::Delay,
    gpio::{Input, Output},
    spi::master::Spi,
};
use heapless::LinearMap;
use ssd1680::displays::adafruit_thinkink_2in9::{Display2in9Gray2, ThinkInk2in9Gray2};
use ssd1680::prelude::*;
use tinybmp::Bmp;

use crate::error::AppError;

/// Display a BMP background image on the e-paper display
pub fn show_background_image(
    spi_device: &mut ExclusiveDevice<Spi<'static, esp_hal::Blocking>, Output<'static>, Delay>,
    busy: Input<'static>,
    dc: Output<'static>,
    rst: Output<'static>,
) -> Result<(), AppError> {
    log::info!("Displaying background image");

    // Create display with SPI interface
    let mut epd = match ThinkInk2in9Gray2::new(spi_device, busy, dc, rst) {
        Ok(display) => display,
        Err(e) => {
            log::error!("Failed to create e-paper display: {:?}", e);
            return Err(AppError::DisplayError);
        }
    };
    let mut display_gray = Display2in9Gray2::new();

    // Initialize the display
    if let Err(e) = epd.begin(&mut Delay::new()) {
        log::error!("Failed to initialize e-paper display: {:?}", e);
        return Err(AppError::DisplayError);
    }
    log::info!("E-paper display initialized");

    // Load the background BMP image
    let bmp_data = include_bytes!("../resources/weather_bg.bmp");
    let bmp = match Bmp::<Gray2>::from_slice(bmp_data) {
        Ok(bmp) => bmp,
        Err(e) => {
            log::error!("Failed to parse BMP image: {:?}", e);
            return Err(AppError::DisplayError);
        }
    };

    log::info!(
        "BMP image loaded: {}x{}",
        bmp.size().width,
        bmp.size().height
    );

    // Draw the BMP image to the display buffer
    let image = Image::new(&bmp, Point::zero());
    if let Err(e) = image.draw(&mut display_gray) {
        log::error!("Failed to draw BMP to display buffer: {:?}", e);
        return Err(AppError::DisplayError);
    }

    log::info!("Updating display with background image");
    // Transfer and display the buffer on the display
    if let Err(e) = epd.update_gray2_and_display(
        display_gray.high_buffer(),
        display_gray.low_buffer(),
        &mut Delay::new(),
    ) {
        log::error!("Failed to update e-paper display: {:?}", e);
        return Err(AppError::DisplayError);
    }

    log::info!("Background image displayed successfully");
    Ok(())
}
