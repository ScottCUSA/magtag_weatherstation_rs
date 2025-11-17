use embedded_graphics::{
    pixelcolor::Gray2,
    prelude::*,
    primitives::{PrimitiveStyle, Rectangle},
};
use embedded_hal_bus::spi::ExclusiveDevice;
use esp_hal::{
    delay::Delay,
    gpio::{Input, Output},
    spi::master::Spi,
};
use ssd1680::displays::adafruit_thinkink_2in9::{Display2in9Gray2, ThinkInk2in9Gray2};
use ssd1680::prelude::*;

use embedded_text::{
    TextBox,
    alignment::HorizontalAlignment,
    style::{HeightMode, TextBoxStyleBuilder},
};

use crate::error::AppError;

pub fn show_on_display(
    text: &str,
    spi_device: &mut ExclusiveDevice<Spi<'static, esp_hal::Blocking>, Output<'static>, Delay>,
    busy: Input<'static>,
    dc: Output<'static>,
    rst: Output<'static>,
) -> Result<(), AppError> {
    log::info!("Show on display: \n{}", text);
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

    // text style: monospace 6x10 as used previously
    let character_style = embedded_graphics::mono_font::MonoTextStyle::new(
        &embedded_graphics::mono_font::ascii::FONT_6X10,
        Gray2::BLACK,
    );

    let textbox_style = TextBoxStyleBuilder::new()
        .height_mode(HeightMode::FitToText)
        .alignment(HorizontalAlignment::Left)
        .paragraph_spacing(2)
        .build();

    // prefer display width 296 for 2.9" ThinkInk; height 0 lets FitToText compute required height
    let bounds = embedded_graphics::primitives::Rectangle::new(Point::zero(), Size::new(296, 0));
    let text_box = TextBox::with_textbox_style(text, bounds, character_style, textbox_style);

    log::info!("Clearing display");
    // clear display first (fill white)
    if let Err(e) = Rectangle::new(Point::new(0, 0), Size::new(296, 128))
        .into_styled(PrimitiveStyle::with_fill(Gray2::WHITE))
        .draw(&mut display_gray)
    {
        log::error!("Failed to clear display: {:?}", e);
        return Err(AppError::DisplayError);
    }

    if let Err(e) = text_box.draw(&mut display_gray) {
        log::error!("Failed to draw text to display buffer: {:?}", e);
        return Err(AppError::DisplayError);
    }

    log::info!("Drawing text to display");
    // Transfer and display the buffer on the display
    if let Err(e) = epd.update_gray2_and_display(
        display_gray.high_buffer(),
        display_gray.low_buffer(),
        &mut Delay::new(),
    ) {
        log::error!("Failed to update e-paper display: {:?}", e);
        return Err(AppError::DisplayError);
    }
    Ok(())
}

/// Show an `AppError` on the display. Returns the same `AppError` so callers
/// can forward it to a sleep helper without needing to map types.
pub fn show_app_error(
    msg: &str,
    spi_device: &mut ExclusiveDevice<Spi<'static, esp_hal::Blocking>, Output<'static>, Delay>,
    busy: Input<'static>,
    dc: Output<'static>,
    rst: Output<'static>,
) {
    let _ = show_on_display(msg, spi_device, busy, dc, rst);
}
