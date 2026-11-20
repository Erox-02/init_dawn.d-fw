// on this build i had a lot of probs , like while building , u needa use `cargo build --target riscv32imc-unknown-none-elf` instead of normal cargo build bec it throws some omnious atomic err 

#![no_std]
#![no_main]

use esp_hal::clock::CpuClock;
use esp_hal::main;
use esp_hal::time::{Duration, Instant};
use esp_hal::gpio::{Input, InputConfig, Pull};

use esp_println::println;

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


    let sw1 = esp_hal::gpio::Input::new(peripherals.GPIO3, cfg);
    let sw2 = esp_hal::gpio::Input::new(peripherals.GPIO4, cfg);
    let sw3 = esp_hal::gpio::Input::new(peripherals.GPIO5, cfg);
    let sw4 = esp_hal::gpio::Input::new(peripherals.GPIO6, cfg);
    let sw5 = esp_hal::gpio::Input::new(peripherals.GPIO7, cfg);                          // js to test the switches 

    let mut previous = [false; 5];

    loop {
        let current = [sw1.is_high(), sw2.is_high(), sw3.is_high(), sw4.is_high(), sw5.is_high(),];
        
        for i in 0..5 {
            if current[i] && !previous[i] {
                esp_println::println!("sw{} has been clicked beepboop", i + 1);   // beepboop , prints when a switch is pressed 
            }
        } 

        previous = current;
        let start = Instant::now();
        while start.elapsed() < Duration::from_millis(20) {}
    }
}