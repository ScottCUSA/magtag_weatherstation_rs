use embedded_graphics::{
    image::{Image, ImageRaw},
    pixelcolor::{BinaryColor, Gray2},
    prelude::*,
    primitives::Rectangle,
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

use crate::{error::AppError, weather::model::ApiResponse};

// load img data at compile time into static storage
static WEATHER_BG: Lazy<ImageRaw<'static, BinaryColor>> = Lazy::new(|| {
    ImageRaw::<BinaryColor>::new(
        include_bytes!("../resources/weather_bg_296x128_1b.raw"),
        296,
    )
});

static WEATHER_ICONS_20PX: Lazy<ImageRaw<'static, Gray2>> = Lazy::new(|| {
    ImageRaw::<Gray2>::new(
        include_bytes!("../resources/weather_icons_20px_60x60_2b.raw"),
        60,
    )
});

static WEATHER_ICONS_70PX: Lazy<ImageRaw<'static, Gray2>> = Lazy::new(|| {
    ImageRaw::<Gray2>::new(
        include_bytes!("../resources/weather_icons_70px_210x210_2b.raw"),
        210,
    )
});

// TODO: consider moving to ssd1680 library
/// Adapter to convert BinaryColor drawings to Gray2
struct BinaryToGray2Adapter<'a, T>(&'a mut T);

impl<'a, T> DrawTarget for BinaryToGray2Adapter<'a, T>
where
    T: DrawTarget<Color = Gray2> + OriginDimensions,
{
    type Color = BinaryColor;
    type Error = T::Error;

    fn draw_iter<I>(&mut self, pixels: I) -> Result<(), Self::Error>
    where
        I: IntoIterator<Item = Pixel<Self::Color>>,
    {
        self.0
            .draw_iter(pixels.into_iter().map(|Pixel(point, color)| {
                let gray2_color = if color.is_off() {
                    Gray2::BLACK
                } else {
                    Gray2::WHITE
                };
                Pixel(point, gray2_color)
            }))
    }
}

impl<'a, T> OriginDimensions for BinaryToGray2Adapter<'a, T>
where
    T: OriginDimensions,
{
    fn size(&self) -> Size {
        self.0.size()
    }
}

/// Display a BMP background image on the e-paper display
pub fn display_graphical_weather(
    weather_data: ApiResponse,
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

    draw_background_image(&mut display_gray).map_err(|_| {
        log::error!("Failed to draw background image to display buffer");
        AppError::DisplayError
    })?;

    draw_today_weather_view(&weather_data, &mut display_gray).map_err(|_| {
        log::error!("Failed to draw today weather view to display buffer");
        AppError::DisplayError
    })?;

    draw_future_weather_view(&weather_data, &mut display_gray).map_err(|_| {
        log::error!("Failed to draw future weather view to display buffer");
        AppError::DisplayError
    })?;

    log::info!("Displaying buffer");
    // Transfer and display the buffer on the display
    if let Err(e) = epd.update_gray2_and_display(
        display_gray.high_buffer(),
        display_gray.low_buffer(),
        &mut Delay::new(),
    ) {
        log::error!("Failed to update e-paper display: {:?}", e);
        return Err(AppError::DisplayError);
    }

    log::info!("Updated display successfully");
    Ok(())
}

/// Draw the background image onto the buffer
pub fn draw_background_image<D>(display: &mut D) -> Result<(), AppError>
where
    D: DrawTarget<Color = Gray2> + OriginDimensions,
{
    log::info!("Drawing background image");

    // Convert and draw BinaryColor image to Gray2 buffer
    let image = Image::new(&*WEATHER_BG, Point::zero());
    image
        .draw(&mut BinaryToGray2Adapter(display))
        .map_err(|_| {
            log::error!("Failed to draw image to display buffer");
            AppError::DisplayError
        })?;

    log::info!("Background image drawn successfully");
    Ok(())
}

/// Draw the today weather view onto the display buffer
fn draw_today_weather_view<D>(weather_data: &ApiResponse, display: &mut D) -> Result<(), AppError>
where
    D: DrawTarget<Color = Gray2> + OriginDimensions,
{
    let today_weather_code =
        weather_code_to_icon_index(*weather_data.daily.weather_code.first().unwrap());
    draw_weather_icon(display, today_weather_code, Point::new(10, 40), 70).map_err(|_| {
        log::error!("Failed to draw image to display buffer");
        AppError::DisplayError
    })?;
    Ok(())
}

/// Draw the future weather view onto the display buffer
fn draw_future_weather_view<D>(weather_data: &ApiResponse, display: &mut D) -> Result<(), AppError>
where
    D: DrawTarget<Color = Gray2> + OriginDimensions,
{
    let days = weather_data.daily.time.len();
    // DAY OF WEEK, WEATHER ICON, MIN(F), MAX(F)
    for day in 1..days {
        let dow = *weather_data.daily.time.get(day).unwrap();
        let icon = weather_code_to_icon_index(*weather_data.daily.weather_code.get(day).unwrap());
        let min = *weather_data.daily.temperature_2m_min.get(day).unwrap();
        let max = *weather_data.daily.temperature_2m_max.get(day).unwrap();
        draw_weather_icon(
            display,
            icon,
            Point::new(220, 15 + ((day as i32 - 1) * 18)),
            20,
        )
        .map_err(|_| {
            log::error!("Failed to draw image to display buffer");
            AppError::DisplayError
        })?;
    }
    Ok(())
}

/// Draw a weather icon from the sprite sheet onto the display
///
/// # Arguments
/// * `display` - The display buffer to draw onto
/// * `icon_index` - Index of the icon in the 3x3 sprite sheet (0-8)
/// * `position` - Where to draw the icon on the display
/// * `size` - Either 20 or 70 for the icon size
pub fn draw_weather_icon<D>(
    display: &mut D,
    icon_index: i32,
    position: Point,
    size: u32,
) -> Result<(), D::Error>
where
    D: DrawTarget<Color = Gray2>,
{
    let (sprite_sheet, icon_size) = match size {
        20 => (&*WEATHER_ICONS_20PX, 20u32),
        70 => (&*WEATHER_ICONS_70PX, 70u32),
        _ => return Ok(()),
    };

    let row = icon_index / 3;
    let col = icon_index % 3;
    let x = (col as u32) * icon_size;
    let y = (row as u32) * icon_size;

    log::info!(
        "{}px sprite sheet row: {} col: {} x: {} y: {}",
        size,
        row,
        col,
        x,
        y
    );

    let rect = Rectangle::new(
        Point::new(x as i32, y as i32),
        Size::new(icon_size, icon_size),
    );
    let sub_image = sprite_sheet.sub_image(&rect);
    log::trace!("{:?}", sub_image);
    Image::new(&sub_image, position).draw(display)
}

/// Map weather codes to icon indices in the sprite sheet (3x3 grid, row-major order)
fn weather_code_to_icon_index(code: i32) -> i32 {
    match code {
        0 => 0,                                               // sunny
        1 => 1,                                               // partly sunny/cloudy
        2 => 2,                                               // cloudy
        3 => 3,                                               // very cloudy
        61 | 63 | 65 => 4,                                    // rain
        51 | 53 | 55 | 80 | 81 | 82 => 5,                     // showers
        95 | 96 | 99 => 6,                                    // storms
        56 | 57 | 66 | 67 | 71 | 73 | 75 | 77 | 85 | 86 => 7, // snow
        45 | 48 => 8,                                         // fog
        _ => 0,                                               // default to sunny
    }
}
