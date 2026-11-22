#![no_std]
#![no_main]

use embedded_graphics::{
    image::{Image, ImageRaw},
    pixelcolor::Rgb565,
    prelude::*,
};

pub fn show_frame<D>(display: &mut D, frame: &[u8])
where
    D: DrawTarget<Color = Rgb565>,
{
    let raw = ImageRaw::<Rgb565>::new(frame, 64);

    Image::new(&raw, Point::new(110, 6))
        .draw(display)
        .ok();
}
