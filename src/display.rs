use embedded_graphics::{
    mono_font::MonoTextStyle,
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
use once_cell::sync::Lazy;
use ssd1680::displays::adafruit_thinkink_2in9::{Display2in9Gray2, ThinkInk2in9Gray2};
use ssd1680::prelude::*;

use embedded_text::{
    TextBox,
    alignment::HorizontalAlignment,
    style::{HeightMode, TextBoxStyleBuilder},
};

use crate::{error::AppError, weather::model::OpenMeteoResponse};

// text style: monospace 6x10 as used previously
pub static CHARACTER_STYLE: Lazy<MonoTextStyle<Gray2>> = Lazy::new(|| {
    MonoTextStyle::new(
        &embedded_graphics::mono_font::ascii::FONT_6X10,
        Gray2::BLACK,
    )
});

pub fn text_display(
    text: &str,
    spi_device: &mut ExclusiveDevice<Spi<'static, esp_hal::Blocking>, Output<'static>, Delay>,
    busy: Input<'static>,
    dc: Output<'static>,
    rst: Output<'static>,
) -> Result<(), AppError> {
    log::info!("Show on display: \n{}", text);

    // Create display with SPI interface
    let mut epd = ThinkInk2in9Gray2::new(spi_device, busy, dc, rst).map_err(|e| {
        log::error!("Failed to create e-paper display: {:?}", e);
        AppError::DisplayError
    })?;
    let mut display = Display2in9Gray2::new();

    // Initialize the display
    epd.begin(&mut Delay::new()).map_err(|e| {
        log::error!("Failed to initialize e-paper display: {:?}", e);
        AppError::DisplayError
    })?;
    log::info!("E-paper display initialized");

    let textbox_style = TextBoxStyleBuilder::new()
        .height_mode(HeightMode::FitToText)
        .alignment(HorizontalAlignment::Left)
        .paragraph_spacing(2)
        .build();

    // prefer display width 296 for 2.9" ThinkInk; height 0 lets FitToText compute required height
    let bounds = embedded_graphics::primitives::Rectangle::new(Point::zero(), Size::new(296, 0));
    let text_box = TextBox::with_textbox_style(text, bounds, *CHARACTER_STYLE, textbox_style);

    log::info!("Clearing display");
    // clear display first (fill white)
    Rectangle::new(Point::new(0, 0), Size::new(296, 128))
        .into_styled(PrimitiveStyle::with_fill(Gray2::WHITE))
        .draw(&mut display)
        .map_err(|e| {
            log::error!("Failed to clear display: {:?}", e);
            AppError::DisplayError
        })?;

    text_box.draw(&mut display).map_err(|e| {
        log::error!("Failed to draw text to display buffer: {:?}", e);
        AppError::DisplayError
    })?;

    log::info!("Drawing text to display");
    // Transfer and display the buffer on the display
    epd.update_gray2_and_display(
        display.high_buffer(),
        display.low_buffer(),
        &mut Delay::new(),
    )
    .map_err(|e| {
        log::error!("Failed to update e-paper display: {:?}", e);
        AppError::DisplayError
    })?;
    Ok(())
}

#[cfg(feature = "graphical")]
pub fn graphical_display(
    weather_data: OpenMeteoResponse,
    spi_device: &mut ExclusiveDevice<Spi<'static, esp_hal::Blocking>, Output<'static>, Delay>,
    busy: Input<'static>,
    dc: Output<'static>,
    rst: Output<'static>,
) -> Result<(), AppError> {
    use crate::graphics::{
        draw_background_image, draw_future_weather_view, draw_today_date, draw_today_high_low,
        draw_today_lat_long, draw_today_sunrise_sunset, draw_today_weather_icon, draw_today_wind,
    };

    let stats: esp_alloc::HeapStats = esp_alloc::HEAP.stats();
    log::info!("{}", stats);

    log::info!("Drawing graphical display");

    // Create display with SPI interface
    let mut epd = ThinkInk2in9Gray2::new(spi_device, busy, dc, rst).map_err(|e| {
        log::error!("Failed to create e-paper display: {:?}", e);
        AppError::DisplayError
    })?;
    let mut display = Display2in9Gray2::new();

    // Initialize the display
    epd.begin(&mut Delay::new()).map_err(|e| {
        log::error!("Failed to initialize e-paper display: {:?}", e);
        AppError::DisplayError
    })?;
    log::info!("E-paper display initialized");

    draw_background_image(&mut display)?;
    draw_today_weather_icon(
        *weather_data.daily.weather_code.first().unwrap(),
        &mut display,
    )?;
    draw_today_date(weather_data.daily.time.first().unwrap(), &mut display)?;
    draw_today_lat_long(weather_data.latitude, weather_data.longitude, &mut display)?;
    draw_today_high_low(
        *weather_data.daily.temperature_2m_max.first().unwrap(),
        *weather_data.daily.temperature_2m_min.first().unwrap(),
        &weather_data
            .daily_units
            .temperature_2m_max
            .chars()
            .last()
            .unwrap(),
        &mut display,
    )?;
    draw_today_wind(
        *weather_data.daily.wind_speed_10m_max.first().unwrap(),
        *weather_data
            .daily
            .wind_direction_10m_dominant
            .first()
            .unwrap(),
        weather_data.daily_units.wind_speed_10m_max,
        &mut display,
    )?;
    draw_today_sunrise_sunset(
        weather_data.daily.sunrise.first().unwrap(),
        weather_data.daily.sunset.first().unwrap(),
        &mut display,
    )?;
    draw_future_weather_view(&weather_data, &mut display)?;

    log::info!("displaying buffer");
    // Transfer and display the buffer on the display
    epd.update_gray2_and_display(
        display.high_buffer(),
        display.low_buffer(),
        &mut Delay::new(),
    )
    .map_err(|e| {
        log::error!("Failed to update e-paper display: {:?}", e);
        AppError::DisplayError
    })?;

    log::info!("updated display successfully");
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
    let _ = text_display(msg, spi_device, busy, dc, rst);
}
