use embedded_graphics::{
    image::{Image, ImageRaw},
    pixelcolor::{BinaryColor, Gray2},
    prelude::*,
    primitives::Rectangle,
};
use embedded_text::{
    TextBox,
    alignment::HorizontalAlignment,
    style::{HeightMode, TextBoxStyleBuilder},
};

use core::fmt::Write;
use heapless::String;
use once_cell::sync::Lazy;

use crate::{
    display::CHARACTER_STYLE,
    error::AppError,
    time::{format_date, get_iso_8601_hh_mm},
    weather::model::OpenMeteoResponse,
};

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

/// Draw the background image onto the buffer
pub fn draw_background_image<D>(buffer: &mut D) -> Result<(), AppError>
where
    D: DrawTarget<Color = Gray2> + OriginDimensions,
{
    // Convert and draw BinaryColor image to Gray2 buffer
    let image = Image::new(&*WEATHER_BG, Point::zero());
    image.draw(&mut BinaryToGray2Adapter(buffer)).map_err(|_| {
        log::error!("Failed to draw image to display buffer");
        AppError::DisplayError
    })?;

    log::info!("Background image drawn successfully");
    Ok(())
}

/// Draw the today weather view onto the display buffer
pub fn draw_today_date<D>(date: &str, buffer: &mut D) -> Result<(), AppError>
where
    D: DrawTarget<Color = Gray2> + OriginDimensions,
    <D as DrawTarget>::Error: core::fmt::Debug,
{
    let textbox_style = TextBoxStyleBuilder::new()
        .height_mode(HeightMode::FitToText)
        .alignment(HorizontalAlignment::Left)
        .paragraph_spacing(2)
        .build();

    // Draw Today's Date
    // need to convert the ISO 8601 time stamp to a nice string
    let date = format_date(date).unwrap();
    let bounds =
        embedded_graphics::primitives::Rectangle::new(Point::new(8, 16), Size::new(296, 0));
    let text_box = TextBox::with_textbox_style(&date, bounds, *CHARACTER_STYLE, textbox_style);
    if let Err(e) = text_box.draw(buffer) {
        log::error!("Failed to draw text to display buffer: {:?}", e);
        return Err(AppError::DisplayError);
    }

    log::info!("Today's date drawn successfully");
    Ok(())
}

/// Draw the today weather view onto the display buffer
pub fn draw_today_lat_long<D>(lat: f32, long: f32, buffer: &mut D) -> Result<(), AppError>
where
    D: DrawTarget<Color = Gray2> + OriginDimensions,
    <D as DrawTarget>::Error: core::fmt::Debug,
{
    let textbox_style = TextBoxStyleBuilder::new()
        .height_mode(HeightMode::FitToText)
        .alignment(HorizontalAlignment::Left)
        .paragraph_spacing(2)
        .build();

    // Draw the Latitute and Longitude
    let mut lat_long_buf: String<24> = String::new();
    write!(&mut lat_long_buf, "({:.4}, {:.4})", lat, long).unwrap();

    let bounds =
        embedded_graphics::primitives::Rectangle::new(Point::new(8, 27), Size::new(296, 0));

    let text_box =
        TextBox::with_textbox_style(&lat_long_buf, bounds, *CHARACTER_STYLE, textbox_style);
    if let Err(e) = text_box.draw(buffer) {
        log::error!("Failed to draw text to display buffer: {:?}", e);
        return Err(AppError::DisplayError);
    }

    log::info!("lat, long drawn successfully");
    Ok(())
}

pub fn draw_today_high_low<D>(
    high: f32,
    low: f32,
    temp_unit: &char,
    buffer: &mut D,
) -> Result<(), AppError>
where
    D: DrawTarget<Color = Gray2> + OriginDimensions,
    <D as DrawTarget>::Error: core::fmt::Debug,
{
    let textbox_style = TextBoxStyleBuilder::new()
        .height_mode(HeightMode::FitToText)
        .alignment(HorizontalAlignment::Left)
        .paragraph_spacing(2)
        .build();

    let mut temp_buf: String<8> = String::new();

    // Draw the low temperatures
    temp_buf.clear();
    write!(&mut temp_buf, "{:.0}{}", low, temp_unit).unwrap();
    let bounds =
        embedded_graphics::primitives::Rectangle::new(Point::new(100, 60), Size::new(80, 0));
    let text_box = TextBox::with_textbox_style(&temp_buf, bounds, *CHARACTER_STYLE, textbox_style);
    if let Err(e) = text_box.draw(buffer) {
        log::error!("Failed to draw text to display buffer: {:?}", e);
        return Err(AppError::DisplayError);
    }
    log::info!("low temp drawn successfully");

    // Draw the high temperature
    temp_buf.clear();
    write!(&mut temp_buf, "{:.0}{}", high, temp_unit).unwrap();
    let bounds =
        embedded_graphics::primitives::Rectangle::new(Point::new(140, 60), Size::new(80, 0));
    let text_box = TextBox::with_textbox_style(&temp_buf, bounds, *CHARACTER_STYLE, textbox_style);
    if let Err(e) = text_box.draw(buffer) {
        log::error!("Failed to draw low_temp to display buffer: {:?}", e);
        return Err(AppError::DisplayError);
    }
    log::info!("high temp drawn successfully");

    Ok(())
}

pub fn draw_today_wind<D>(
    wind_speed: f32,
    wind_dir: i32,
    wind_unit: &str,
    buffer: &mut D,
) -> Result<(), AppError>
where
    D: DrawTarget<Color = Gray2> + OriginDimensions,
    <D as DrawTarget>::Error: core::fmt::Debug,
{
    let textbox_style = TextBoxStyleBuilder::new()
        .height_mode(HeightMode::FitToText)
        .alignment(HorizontalAlignment::Left)
        .paragraph_spacing(2)
        .build();

    let mut wind_buf: String<24> = String::new();

    // Draw the wind speed + direction
    let wind_dir = wind_dir_text(wind_dir);
    wind_buf.clear();
    write!(&mut wind_buf, "{}{} {}", wind_speed, wind_unit, wind_dir).unwrap();

    let bounds =
        embedded_graphics::primitives::Rectangle::new(Point::new(95, 90), Size::new(80, 0));
    let text_box = TextBox::with_textbox_style(&wind_buf, bounds, *CHARACTER_STYLE, textbox_style);
    if let Err(e) = text_box.draw(buffer) {
        log::error!("Failed to draw windspeed to display buffer: {:?}", e);
        return Err(AppError::DisplayError);
    }

    log::info!("windspeed drawn successfully");

    Ok(())
}

pub fn draw_today_weather_icon<D>(weather_code: i32, buffer: &mut D) -> Result<(), AppError>
where
    D: DrawTarget<Color = Gray2> + OriginDimensions,
    <D as DrawTarget>::Error: core::fmt::Debug,
{
    let icon = weather_code_to_icon_index(weather_code);
    draw_weather_icon(icon, Point::new(6, 40), 70, buffer).map_err(|_| {
        log::error!("Failed to draw image to display buffer");
        AppError::DisplayError
    })?;
    log::info!("today weather icon drawn successfully");
    Ok(())
}

pub fn draw_today_sunrise_sunset<D>(
    sunrise: &str,
    sunset: &str,
    buffer: &mut D,
) -> Result<(), AppError>
where
    D: DrawTarget<Color = Gray2> + OriginDimensions,
    <D as DrawTarget>::Error: core::fmt::Debug,
{
    let textbox_style = TextBoxStyleBuilder::new()
        .height_mode(HeightMode::FitToText)
        .alignment(HorizontalAlignment::Left)
        .paragraph_spacing(2)
        .build();
    // Draw sunrise
    let time = get_iso_8601_hh_mm(sunrise).unwrap();
    let bounds =
        embedded_graphics::primitives::Rectangle::new(Point::new(30, 113), Size::new(296, 0));
    let text_box = TextBox::with_textbox_style(time, bounds, *CHARACTER_STYLE, textbox_style);
    if let Err(e) = text_box.draw(buffer) {
        log::error!("Failed to draw text to display buffer: {:?}", e);
        return Err(AppError::DisplayError);
    }
    log::info!("sunrise drawn successfully");

    // Draw sunset
    let time = get_iso_8601_hh_mm(sunset).unwrap();
    let bounds =
        embedded_graphics::primitives::Rectangle::new(Point::new(115, 113), Size::new(296, 0));
    let text_box = TextBox::with_textbox_style(time, bounds, *CHARACTER_STYLE, textbox_style);
    if let Err(e) = text_box.draw(buffer) {
        log::error!("Failed to draw text to display buffer: {:?}", e);
        return Err(AppError::DisplayError);
    }
    log::info!("sunset drawn successfully");
    Ok(())
}

/// Draw the future weather view onto the display buffer
pub fn draw_future_weather_view<D>(
    weather_data: &OpenMeteoResponse,
    buffer: &mut D,
) -> Result<(), AppError>
where
    D: DrawTarget<Color = Gray2> + OriginDimensions,
    <D as DrawTarget>::Error: core::fmt::Debug,
{
    let textbox_style = TextBoxStyleBuilder::new()
        .height_mode(HeightMode::FitToText)
        .alignment(HorizontalAlignment::Left)
        .paragraph_spacing(2)
        .build();

    let days = weather_data.daily.time.len();
    let temp_unit = &weather_data
        .daily_units
        .temperature_2m_max
        .chars()
        .last()
        .unwrap();

    let mut min_buf: String<8> = String::new();
    let mut max_buf: String<8> = String::new();

    // Draw the day of week, weather icon, the min and max temp for each future day
    for i in 1..days {
        min_buf.clear();
        max_buf.clear();
        let start_point = Point::new(191, 15 + ((i as i32 - 1) * 18));

        // day of week
        let date = weather_data.daily.time.get(i).unwrap();
        let y = date[0..4].parse().unwrap();
        let m = date[5..7].parse().unwrap();
        let d = date[8..10].parse().unwrap();
        let dow = day_of_week_sakamoto(y, m, d);

        let bounds = embedded_graphics::primitives::Rectangle::new(
            start_point + Point::new(0, 5),
            Size::new(20, 0),
        );
        let text_box = TextBox::with_textbox_style(dow, bounds, *CHARACTER_STYLE, textbox_style);
        if let Err(e) = text_box.draw(buffer) {
            log::error!("Failed to draw text to display buffer: {:?}", e);
            return Err(AppError::DisplayError);
        }

        // weather icon
        let icon = weather_code_to_icon_index(*weather_data.daily.weather_code.get(i).unwrap());
        draw_weather_icon(icon, start_point + Point::new(20, 0), 20, buffer).map_err(|_| {
            log::error!("Failed to draw image to display buffer");
            AppError::DisplayError
        })?;

        // minimum temperature
        write!(
            &mut min_buf,
            "{:.0}{}",
            weather_data.daily.temperature_2m_min[i], temp_unit
        )
        .unwrap();

        let bounds = embedded_graphics::primitives::Rectangle::new(
            start_point + Point::new(45, 5),
            Size::new(30, 0),
        );
        let text_box =
            TextBox::with_textbox_style(&min_buf, bounds, *CHARACTER_STYLE, textbox_style);
        if let Err(e) = text_box.draw(buffer) {
            log::error!("Failed to draw text to display buffer: {:?}", e);
            return Err(AppError::DisplayError);
        }

        // maximum temperature
        write!(
            &mut max_buf,
            "{:.0}{}",
            weather_data.daily.temperature_2m_max[i], temp_unit
        )
        .unwrap();

        let bounds = embedded_graphics::primitives::Rectangle::new(
            start_point + Point::new(75, 5),
            Size::new(30, 0),
        );
        let text_box =
            TextBox::with_textbox_style(&max_buf, bounds, *CHARACTER_STYLE, textbox_style);
        if let Err(e) = text_box.draw(buffer) {
            log::error!("Failed to draw text to display buffer: {:?}", e);
            return Err(AppError::DisplayError);
        }
        log::info!("future day {} drawn successfully", i);
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
    icon_index: i32,
    position: Point,
    size: u32,
    buffer: &mut D,
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
    Image::new(&sub_image, position).draw(buffer)
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

/// Get the day of the week using the Sakamoto algorithm
fn day_of_week_sakamoto(year: i32, month: i32, day: i32) -> &'static str {
    let mut y = year;
    let t = [0, 3, 2, 5, 0, 3, 5, 1, 4, 6, 2, 4];
    if month < 3 {
        y -= 1;
    }
    let dow = (y + y / 4 - y / 100 + y / 400 + t[(month - 1) as usize] + day) % 7;
    match dow {
        0 => "SUN",
        1 => "MON",
        2 => "TUE",
        3 => "WED",
        4 => "THU",
        5 => "FRI",
        _ => "SAT",
    }
}

fn wind_dir_text(direction: i32) -> &'static str {
    match direction {
        0..22 => "N",
        22..67 => "NE",
        67..122 => "E",
        122..157 => "SE",
        157..202 => "S",
        202..247 => "SW",
        247..293 => "W",
        293..337 => "NW",
        _ => "N",
    }
}
