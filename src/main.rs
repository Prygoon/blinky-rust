#![deny(unsafe_code)]
#![no_std]
#![no_main]


// Includes.
use panic_halt as _;

use nb::block;

use cortex_m_rt::entry;
use stm32f1xx_hal::{pac, prelude::*, timer::Timer};


#[entry]
fn main() -> ! {
    // Get access to the core peripherals from the cortex-m crate
    let cortex_periph = cortex_m::Peripherals::take().unwrap();
    // Get access to the device specific peripherals from the peripheral access crate
    let device_periph = pac::Peripherals::take().unwrap();

    // Take ownership over the raw flash and rcc devices and convert them into the corresponding
    // HAL structs
    let mut flash = device_periph.FLASH.constrain();
    let mut rcc = device_periph.RCC.constrain();

    // Freeze the configuration of all the clocks in the system and store the frozen frequencies in
    // `clocks`
    let clocks = rcc.cfgr.freeze(&mut flash.acr);

    // Acquire the GPIOB peripheral
    let mut gpiob = device_periph.GPIOB.split(&mut rcc.apb2);

    // Configure gpio B pin 1 as a push-pull output. The `crl` register is passed to the function
    // in order to configure the port. For pins 0-7, crl should be passed instead.
    let mut led = gpiob.pb1.into_push_pull_output(&mut gpiob.crl);
    // Configure the syst timer to trigger an update every second
    let mut timer = Timer::syst(cortex_periph.SYST, &clocks).start_count_down(1.hz());

    loop {
        let _ = led.toggle();
        block!(timer.wait()).unwrap();
    }
}
