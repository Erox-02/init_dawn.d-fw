#![no_std]
#![no_main]
use embedded_graphics::{
    image::{Image, ImageRaw},
    pixelcolor::Rgb565,
    prelude::*,
};
use esp_hal::delay::Delay;

pub fn disp_gif<D>(display: &mut D, frames: &[&[u8]], delay: &mut Delay, l: &mut bool )
where
    D: DrawTarget<Color = Rgb565>,
{
    loop {
        for frame in frames{
        let raw = ImageRaw::<Rgb565>::new(frame, 64);
        let x = raw.clone(); // idk why but compiler threw an err on same stance , so i cloned lol its funny 

    Image::new(&raw, Point::new(10, 6)).draw(display).ok();
    Image::new(&x, Point::new(200, 6)).draw(display).ok();
    
    delay.delay_millis(66);
        }
    }
    return *l = true;
}
