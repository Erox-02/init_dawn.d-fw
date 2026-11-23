#![no_std]
#![no_main]

use embedded_graphics::{
    image::{Image, ImageRaw},
    pixelcolor::Rgb565,
    prelude::*,
};
use esp_hal::delay::Delay;

pub fn show_frame<D>(display: &mut D, frames: &[&[u8]], delay: &mut Delay)
where
    D: DrawTarget<Color = Rgb565>,
{
    loop {
        for frame in frames{
        let raw = ImageRaw::<Rgb565>::new(frame, 64);

    Image::new(&raw, Point::new(110, 6)).draw(display).ok();

    delay.delay_millis(66);
        }
    }
}
