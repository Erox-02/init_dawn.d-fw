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
use embedded_graphics::{pixelcolor::Rgb565, prelude::*, primitives::{PrimitiveStyle, Rectangle}, mono_font::{ascii::{FONT_6X10, FONT_10X20}, MonoTextStyle}, text::{Baseline, Text},image::{Image, ImageRaw}};
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

    let sw1 = esp_hal::gpio::Input::new(peripherals.GPIO3, cfg);
    let sw2 = esp_hal::gpio::Input::new(peripherals.GPIO4, cfg);
    let sw3 = esp_hal::gpio::Input::new(peripherals.GPIO5, cfg);
    let sw4 = esp_hal::gpio::Input::new(peripherals.GPIO6, cfg);
    let sw5 = esp_hal::gpio::Input::new(peripherals.GPIO7, cfg); 

    let mut previous = [false; 5];
    let mut multiplier = 1; 
    

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
    
    // params found frm diff tests
    let mut display = Builder::new(ST7789, interface).display_size(76, 284).display_offset(82, 18).orientation(Orientation::new().rotate(Rotation::Deg90)).reset_pin(rst).init(&mut delay).unwrap();

    buzzer.set_low();

    let white = MonoTextStyle::new(&FONT_6X10, Rgb565::WHITE,);
    let tim = MonoTextStyle::new(&FONT_10X20, Rgb565::WHITE,);
    display.clear(Rgb565::BLACK).unwrap();

    let green = MonoTextStyle::new(&FONT_6X10, Rgb565::GREEN);
    Text::with_baseline("Init_Dawn", Point::new(110, 10), tim, Baseline::Top,).draw(&mut display).unwrap();

    for i in 0..=120 {
        Rectangle::new(Point::new(82, 30),Size::new(i, 2),).into_styled(PrimitiveStyle::with_fill(Rgb565::GREEN)).draw(&mut display).unwrap();
        delay.delay_millis(15); // looks so sickkks
    }
    Text::with_baseline("readyy", Point::new(124, 43), green ,Baseline::Top).draw(&mut display).unwrap();
    delay.delay_millis(700);
        display.clear(Rgb565::BLACK).unwrap();
    Text::with_baseline("1x", Point::new(0, 0), white, Baseline::Top,).draw(&mut display).unwrap();
    Text::with_baseline("Shift : ON", Point::new(220, 0), white, Baseline::Top,).draw(&mut display).unwrap();
    Text::with_baseline(
        "00:00:00",
        Point::new(100, 27),   // it took me 10 turns to js make it in perfect size :sobspin:
        tim,
        Baseline::Top,
    ).draw(&mut display).unwrap();
    let mut shift_on = false;

    // ui archi for the timer and blah blah 
    let mut hrs: u8 = 0;
    let mut mins: u8 = 0;
    let mut secs: u8 = 0; // dont pronounce it wrong :skull:
    let mut cnt_down = false; // it is count bruh
    let mut tick = Instant::now();

    let frames: [&[u8]; 15] = [include_bytes!("../frames/frame_000.raw"),include_bytes!("../frames/frame_001.raw"),include_bytes!("../frames/frame_002.raw"),include_bytes!("../frames/frame_003.raw"),include_bytes!("../frames/frame_004.raw"),
    include_bytes!("../frames/frame_005.raw"),include_bytes!("../frames/frame_006.raw"),include_bytes!("../frames/frame_007.raw"),include_bytes!("../frames/frame_008.raw"),include_bytes!("../frames/frame_009.raw"),include_bytes!("../frames/frame_010.raw"),
    include_bytes!("../frames/frame_011.raw"),include_bytes!("../frames/frame_012.raw"),include_bytes!("../frames/frame_013.raw"),include_bytes!("../frames/frame_014.raw"),];
    let mut alarm: bool = false;    

    loop {
        let current = [
            sw1.is_high(),
            sw2.is_high(),
            sw3.is_high(),
            sw4.is_high(),
            sw5.is_high(),
        ];

        // ok i js came to realize tht m pcb doesnt have any extra button for the secs , so i will js use cur*[3] as min and cur*[2] as sec , you people be like huh wheres the hour? bro are u autistic ? but i am god in resource management
        // i will use a 60x multiplier eazzy , and it will make sec min and min sec
        let enter = current[1] && !previous[1];
        let mut need_rst = false;

        if enter && !cnt_down && (hrs != 0 || mins != 0 || secs != 0) {
            cnt_down = true;
            tick = Instant::now();
            println!("cntdown")
        }

        if cnt_down {
            let now = Instant::now();
            if (now - tick) >= Duration::from_secs(1) { //some old vs new api lol lts vs cargo suggested one 
                tick = tick + Duration::from_secs(1);
            
            if secs > 0 {
                secs -= 1;
            } else if mins > 0 {
                mins -= 1;
                secs = 59;
            } else if hrs > 0 {
                hrs -= 1;
                mins = 59;
                secs = 59;  
            }
            if hrs == 0 && mins == 0 && secs == 0 {
                cnt_down = false;
                alarm = true;
            }
            need_rst = true;
            }
        }

        if current[4] && !previous[4] { // lol i was tryin to fix it by flashing diff comps again and again but then i realized it wont print anything unless you press the button first :skull:
            multiplier = match multiplier {
                1 => 2,
                2 => 5,
                5 => 10,
                10 => 15,
                15 => 60,
                _ => 1,
            };
        Rectangle::new(Point::new(0,0),Size::new(24,10)).into_styled(PrimitiveStyle::with_fill(Rgb565::BLACK)).draw(&mut display).unwrap();

        let val = match multiplier {
                1 => "1x",
                2 => "2x",
                5 => "5x",
                10 => "10x",
                15 => "15x",
                60 => "60x",
                _ => "1x",
            };  // idk why the hell did i write this , i could've made an enum and yeah it wd have worked too

            Text::with_baseline(
                val,
                Point::new(0,0), // corner at the start
                white,
                Baseline::Top,
            ).draw(&mut display).unwrap();
        }

    if current[0] && !previous[0] {
        shift_on = !shift_on;
    Rectangle::new(Point::new(210,0),Size::new(230,10)).into_styled(PrimitiveStyle::with_fill(Rgb565::BLACK)).draw(&mut display).unwrap(); // parameters and points guessed i meant measured through a lot of tests

    // display.clear(Rgb565::BLACK).unwrap();

    let shit_text = if shift_on {
        "Shift : ON"
    } else {
        "Shift : OFF"
    };
        Text::with_baseline(
            shit_text, // a typo tht stayed  as a meme
            Point::new(220,0), // right corner at the start
            white,
            Baseline::Top,
        ).draw(&mut display).unwrap();
        }

    if current[2] && !previous[2] && !cnt_down {
        if multiplier == 60 {
            if shift_on {
                mins = (mins + 59) % 60;
        } else {
                mins = (mins + 1) % 60;
            }
            need_rst = true;
        } else if shift_on {
            secs = (secs + 60 - (multiplier % 60)) % 60;
            need_rst = true;
        } else {
            secs = (secs + multiplier) % 60;
            need_rst = true;
        }
    }

    if current[3] && !previous[3] && !cnt_down {
        if shift_on {
            let total = hrs as i16 * 60 + mins as i16 - multiplier as i16;
            let total = total.rem_euclid(24 * 60);
            hrs = (total / 60) as u8;
            mins = (total % 60) as u8;
            need_rst = true;
        } else {
            let total = hrs as u16 * 60 + mins as u16 + multiplier as u16;
            let total = total % (24 * 60);
            hrs = (total / 60) as u8;
            mins = (total % 60) as u8;
            need_rst = true;  // history repeats hereby proved , i did the same err 20 mins ago lol      
        }
    }

    if need_rst {
        Rectangle::new(Point::new(100,20),Size::new(160,60)).into_styled(PrimitiveStyle::with_fill(Rgb565::BLACK)).draw(&mut display).unwrap(); // parameters and points guessed i meant measured through a lot of tests


    // this logic was ai generated
    let timer_text = [   
        b'0' + hrs / 10,
        b'0' + hrs % 10,
        b':',
        b'0' + mins / 10,
        b'0' + mins % 10,
        b':',
        b'0' + secs / 10,
        b'0' + secs % 10,
        ];

    let timer_str = core::str::from_utf8(&timer_text).unwrap();

    // display.clear(Rgb565::BLACK).unwrap();
    Text::with_baseline(
        timer_str,
        Point::new(100, 27),
        // Point::new(120, 30), // looks pretty odd 
        tim,
        Baseline::Top,
    )
    .draw(&mut display)
    .unwrap();
    }

    if alarm { // another file was suchh a hassle :sob:
        display.clear(Rgb565::BLACK).unwrap();
        for frame in frames{
        let raw = ImageRaw::<Rgb565>::new(frame, 64);
        let x = raw.clone(); // idk why but compiler threw an err on same stance , so i cloned lol its funny 

    Image::new(&raw, Point::new(10, 6)).draw(&mut display).ok();
    Image::new(&x, Point::new(200, 6)).draw(&mut display).ok();
    delay.delay_millis(66);

            }

    buzzer.set_high();
        }

    // end of slop a.k.a <eos>
    
        delay.delay_millis(20);
        previous = current;
    }

// display.clear(Rgb565::BLACK).unwrap();

// let green = MonoTextStyle::new(
//     &FONT_6X10,
//     Rgb565::GREEN,
// );

// // Title
// Text::with_baseline(
//     "INIT_DAWN",
//     Point::new(110, 8),
//     white,
//     Baseline::Top,
// )
// .draw(&mut display)
// .unwrap();

// // Boot message
// Text::with_baseline(
//     "BOOTING SYSTEM...",
//     Point::new(94, 25),
//     green,
//     Baseline::Top,
// )
// .draw(&mut display)
// .unwrap();

// // Progress bar
// for i in 0..=100 {
//     let width = (i * 120 / 100) as u32;

//     Rectangle::new(
//         Point::new(82, 43),
//         Size::new(width, 5),
//     )
//     .into_styled(
//         PrimitiveStyle::with_fill(Rgb565::GREEN)
//     )
//     .draw(&mut display)
//     .unwrap();

//     delay.delay_millis(25);
// }

// // Ready
// Text::with_baseline(
//     "SYSTEM READY",
//     Point::new(103, 57),
//     green,
//     Baseline::Top,
// )
// .draw(&mut display)
// .unwrap();

// loop {} 
//==================================it was slop i used it and now ik the exact cords of my display lol==========================
}
