use embedded_graphics::{
    image::Image,
    mono_font::MonoTextStyle,
    pixelcolor::{BinaryColor, Gray2},
    prelude::*,
    primitives::Rectangle,
};
use embedded_text::{
    TextBox,
    alignment::HorizontalAlignment,
    style::{HeightMode, TextBoxStyleBuilder},
};

use once_cell::sync::Lazy;

use crate::error::{AppError, Result};

// text style: monospace 6x10 as used previously
pub static CHARACTER_STYLE: Lazy<MonoTextStyle<Gray2>> = Lazy::new(|| {
    MonoTextStyle::new(
        &embedded_graphics::mono_font::ascii::FONT_6X10,
        Gray2::BLACK,
    )
});

/// Draw `text` inside a rectangle at `(x,y)` with width `w` and height `h` on `buffer` using the module text style.
///
/// Returns `Ok(())` on success or `AppError::GraphicsError` on failure.
pub fn draw_text<D>(text: &str, x: i32, y: i32, w: u32, h: u32, buffer: &mut D) -> Result<()>
where
    D: DrawTarget<Color = Gray2> + OriginDimensions,
    <D as DrawTarget>::Error: core::fmt::Debug,
{
    draw_text_at_point(text, Point::new(x, y), Size::new(w, h), buffer)
}

/// Draw `text` inside a rectangle at `top_left` with `size` on `buffer` using the module text style.
///
/// Returns `Ok(())` on success or `AppError::GraphicsError` on failure.
pub fn draw_text_at_point<D>(text: &str, top_left: Point, size: Size, buffer: &mut D) -> Result<()>
where
    D: DrawTarget<Color = Gray2> + OriginDimensions,
    <D as DrawTarget>::Error: core::fmt::Debug,
{
    let textbox_style = TextBoxStyleBuilder::new()
        .height_mode(HeightMode::FitToText)
        .alignment(HorizontalAlignment::Left)
        .paragraph_spacing(2)
        .build();

    let bounds = Rectangle::new(top_left, size);
    let text_box = TextBox::with_textbox_style(text, bounds, *CHARACTER_STYLE, textbox_style);
    text_box.draw(buffer).map_err(|e| {
        log::error!("Failed to draw text to display buffer: {:?}", e);
        AppError::GraphicsError
    })?;

    Ok(())
}

/// Draw a Gray2 `image` at `position` onto `buffer`.
///
/// Returns `Ok(())` on success or `AppError::GraphicsError` on failure.
pub fn draw_image<T, D>(image: &T, position: Point, buffer: &mut D) -> Result<()>
where
    T: ImageDrawable<Color = Gray2>,
    D: DrawTarget<Color = Gray2>,
    <D as DrawTarget>::Error: core::fmt::Debug,
{
    Image::new(image, position).draw(buffer).map_err(|e| {
        log::error!("Failed to draw weather icon to display buffer: {:?}", e);
        AppError::GraphicsError
    })
}

/// Draw a `BinaryColor` image at `position` onto `buffer`, converting to `Gray2`.
///
/// Useful for drawing 1-bit images on a Gray2 target. Returns `Ok(())` or `AppError::GraphicsError`.
pub fn draw_binary_color_image<T, D>(image: &T, position: Point, buffer: &mut D) -> Result<()>
where
    T: ImageDrawable<Color = BinaryColor>,
    D: DrawTarget<Color = Gray2> + OriginDimensions,
    <D as DrawTarget>::Error: core::fmt::Debug,
{
    Image::new(image, position)
        .draw(&mut BinaryToGray2Adapter(buffer))
        .map_err(|e| {
            log::error!("Failed to draw weather icon to display buffer: {:?}", e);
            AppError::GraphicsError
        })
}

/// Adapter to convert BinaryColor drawings to Gray2
struct BinaryToGray2Adapter<'a, T>(&'a mut T);

impl<'a, T> DrawTarget for BinaryToGray2Adapter<'a, T>
where
    T: DrawTarget<Color = Gray2> + OriginDimensions,
{
    type Color = BinaryColor;
    type Error = T::Error;

    fn draw_iter<I>(&mut self, pixels: I) -> core::result::Result<(), Self::Error>
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
