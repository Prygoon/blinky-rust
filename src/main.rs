// #![deny(unsafe_code)]
#![no_std]
#![no_main]

use core::borrow::BorrowMut;
// Includes.
use panic_halt as _;

use stm32f1xx_hal as hal;

use crate::hal::{
    gpio::{gpiob, Output, PushPull},
    pac::{interrupt, Interrupt, Peripherals, TIM2},
    prelude::*,
    timer::{CountDownTimer, Event},
};

use core::cell::RefCell;
use cortex_m::{asm::wfi, interrupt::Mutex};
use cortex_m_rt::entry;
use embedded_hal::digital::v2::OutputPin;
use stm32f1xx_hal::timer::Timer;

// NOTE You can uncomment 'hprintln' here and in the code below for a bit more
// verbosity at runtime, at the cost of throwing off the timing of the blink
// (using 'semihosting' for printing debug info anywhere slows program
// execution down)
//use cortex_m_semihosting::hprintln;

// A type definition for the GPIO pin to be used for our LED
type LedPin = gpiob::PB1<Output<PushPull>>;

// Make LED pin globally available
static G_LED: Mutex<RefCell<Option<LedPin>>> = Mutex::new(RefCell::new(None));

// Make timer interrupt registers globally available
static G_TIM: Mutex<RefCell<Option<CountDownTimer<TIM2>>>> = Mutex::new(RefCell::new(None));

// Define an interrupt handler, i.e. function to call when interrupt occurs.
// This specific interrupt will "trip" when the timer tim2 times out
#[interrupt]
#[allow(non_snake_case)]
unsafe fn TIM2() {
    static mut LED: Option<LedPin> = None;
    static mut TIM: Option<CountDownTimer<TIM2>> = None;

    let led = LED.borrow_mut().get_or_insert_with(|| {
        cortex_m::interrupt::free(|cs|
            // Move LED pin here, leaving a None in its place
            {
                G_LED.borrow(cs).replace(None).unwrap()
            })
    });

    let tim = TIM.borrow_mut().get_or_insert_with(|| {
        cortex_m::interrupt::free(|cs| {
            // Move LED pin here, leaving a None in its place
            G_TIM.borrow(cs).replace(None).unwrap()
        })
    });

    let _ = led.toggle();
    let _ = tim.wait();
}
#[deny(non_snake_case)]
#[entry]
fn main() -> ! {
    let device_periph = Peripherals::take().unwrap();

    let mut rcc = device_periph.RCC.constrain();
    let mut flash = device_periph.FLASH.constrain();
    let clocks = rcc
        .cfgr
        .sysclk(8.mhz())
        .pclk1(8.mhz())
        .freeze(&mut flash.acr);

    // Configure PC13 pin to blink LED
    let mut gpiob = device_periph.GPIOB.split(&mut rcc.apb2);
    let mut led = gpiob.pb1.into_push_pull_output(&mut gpiob.crl);
    let _ = led.set_high(); // Turn off

    // Move the pin into our global storage
    cortex_m::interrupt::free(|cs| *G_LED.borrow(cs).borrow_mut() = Some(led));

    // Set up a timer expiring after 1s
    let mut timer =
        Timer::tim2(device_periph.TIM2, &clocks, &mut rcc.apb1).start_count_down(1.hz());
    // timer.start(1.secs()).unwrap();

    // Generate an interrupt when the timer expires
    timer.listen(Event::Update);

    // Move the timer into our global storage
    cortex_m::interrupt::free(|cs| *G_TIM.borrow(cs).borrow_mut() = Some(timer));

    unsafe {
        cortex_m::peripheral::NVIC::unmask(Interrupt::TIM2);
    }

    loop {
        wfi();
    }
}
