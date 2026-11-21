// on this build i had a lot of probs , like while building , u needa use `cargo build --target riscv32imc-unknown-none-elf` instead of normal cargo build bec it throws some omnious atomic err 
// my display had bl reverse logic so bl - high then actually off and bl -low then actually on , kind of hell in debugging but i fixed it with a lot of suffering anyways
#![no_std]
#![no_main]

use esp_hal::clock::CpuClock;
use esp_hal::main;
use esp_hal::time::{Duration, Instant, Rate};
use esp_hal::gpio::{Input, InputConfig, Pull, Output, OutputConfig, Level};     // bruh forgot output
use esp_hal::spi::{Mode, master::{Config, Spi}};
use esp_println::println;
use esp_hal::delay::Delay;

use embedded_hal_bus::spi::ExclusiveDevice;
use embedded_graphics::{pixelcolor::Rgb565, prelude:: *, primitives::{PrimitiveStyle, Rectangle}, mono_font::{ascii::FONT_6X10, MonoTextStyle}, text::{Baseline, Text}, image::{Image, ImageRawBE}};
use mipidsi::{Builder, models::ST7789, interface::SpiInterface, options::{Orientation, Rotation}};

#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    loop {}
}

// This creates a default app-descriptor required by the esp-idf bootloader.
// For more information see: <https://docs.espressif.com/projects/esp-idf/en/stable/esp32/api-reference/system/app_image_format.html#application-description>
esp_bootloader_esp_idf::esp_app_desc!();

#[allow(
    clippy::large_stack_frames,
    reason = "it's not unusual to allocate larger buffers etc. in main"
)]

#[main]
fn main() -> ! {
    let peripherals = esp_hal::init(
        esp_hal::Config::default().with_cpu_clock(CpuClock::max())
    );

    let cfg = esp_hal::gpio::InputConfig::default().with_pull(esp_hal::gpio::Pull::Down);
    let spi = Spi::new(peripherals.SPI2, Config::default().with_frequency(Rate::from_mhz(10)).with_mode(Mode::_0),).unwrap().with_sck(peripherals.GPIO10).with_mosi(peripherals.GPIO8);
    const RICK_RAW: &[u8] = include_bytes!("/home/archbtw/init_dawn.d-fw/init/rick.raw");
    const IMG_W: u32 = 320;
    const IMG_H: u32 = 240;

    let sw1 = esp_hal::gpio::Input::new(peripherals.GPIO3, cfg);
    let sw2 = esp_hal::gpio::Input::new(peripherals.GPIO4, cfg);
    let sw3 = esp_hal::gpio::Input::new(peripherals.GPIO5, cfg);
    let sw4 = esp_hal::gpio::Input::new(peripherals.GPIO6, cfg);
    let sw5 = esp_hal::gpio::Input::new(peripherals.GPIO7, cfg);                          // js to test the switches 

    let mut previous = [false; 5];

        let cs = Output::new(
        peripherals.GPIO2,
        Level::High,
        OutputConfig::default(),
    );

    let dc = Output::new(
        peripherals.GPIO20,
        Level::Low,
        OutputConfig::default(),
    );

    let rst = Output::new(
        peripherals.GPIO9,
        Level::High,
        OutputConfig::default(),
    );

    let mut buzzer = Output::new(
      peripherals.GPIO21,
        Level::Low,
        OutputConfig::default(),
    );

    let spidev = ExclusiveDevice::new(
        spi,
        cs,
        Delay::new(),
    ).unwrap();

    let mut buffer = [0u8; 512]; // got the sizes and specs of the display frm here `https://blare.hackclub.com/docs/firmware` the guide is good af , but i am a crab lol

    let interface = SpiInterface::new(
        spidev,
        dc,
        &mut buffer,
    );

    let mut delay = Delay::new();
    
    let mut display = Builder::new(ST7789, interface)
    .display_size(76, 284)
    .display_offset(82, 18)
    .orientation(Orientation::new().rotate(Rotation::Deg90))
    .reset_pin(rst)
    .init(&mut delay)
    .unwrap();

    loop {
        buzzer.set_low();
        let current = [sw1.is_high(), sw2.is_high(), sw3.is_high(), sw4.is_high(), sw5.is_high(),];
        
        for i in 0..5 {
            if current[i] && !previous[i] {
                esp_println::println!("sw{} has been clicked beepboop", i + 1);   // beepboop , prints when a switch is pressed 
            }
        } 


    let raw: ImageRawBE<Rgb565> = ImageRawBE::new(RICK_RAW, IMG_W);  // gemini generated dont think this img processing part is written by me 
    let crop_x = (IMG_W.saturating_sub(284) / 2) as i32;
    let crop_y = (IMG_H.saturating_sub(76) / 2) as i32;
    let cropped = raw.sub_image(&Rectangle::new(
        Point::new(crop_x, crop_y),
        Size::new(284.min(IMG_W), 76.min(IMG_H)),
    ));

    Image::new(&cropped, Point::zero()).draw(&mut display).unwrap();

    } 

}