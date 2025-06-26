#![no_std]
#![no_main]

use defmt::*;
use embassy_executor::Spawner;
use embassy_stm32::gpio::{Level, Output, Speed};
use embassy_time::Timer;
use embassy_sync::signal::Signal;
use embassy_sync::blocking_mutex::raw::ThreadModeRawMutex;
use {defmt_rtt as _, panic_probe as _};


static LED1_SIGNAL: Signal<ThreadModeRawMutex, bool> = Signal::new();
static LED2_SIGNAL: Signal<ThreadModeRawMutex, bool> = Signal::new();
static LED3_SIGNAL: Signal<ThreadModeRawMutex, bool> = Signal::new();
static MAIN_SIGNAL: Signal<ThreadModeRawMutex, bool> = Signal::new();

#[embassy_executor::task(pool_size = 3)]
async fn handle_led(mut led: Output<'static>, signal: &'static Signal<ThreadModeRawMutex, bool>, name: &'static str, next_signal: &'static Signal<ThreadModeRawMutex, bool>, wait: u16) {
    loop {
        let state = signal.wait().await;
        led.set_level(
            match state {
                true => Level::Low,
                false => Level::High
            }
        );
        info!("Set {} to: {}", name, state);
        Timer::after_millis(wait as u64).await;
        next_signal.signal(state);
    }
}

pub fn setup_trace() {
    // clear reset for IO port E
    embassy_stm32::pac::RCC.ahb1rstr().write(|w| w.set_gpioerst(false));
    // switch trace pins to alternate function
    embassy_stm32::pac::GPIOE.moder().modify(|w| w.set_moder(2, embassy_stm32::pac::gpio::vals::Moder::ALTERNATE));
    embassy_stm32::pac::GPIOE.moder().modify(|w| w.set_moder(3, embassy_stm32::pac::gpio::vals::Moder::ALTERNATE));
    embassy_stm32::pac::GPIOE.moder().modify(|w| w.set_moder(4, embassy_stm32::pac::gpio::vals::Moder::ALTERNATE));
    embassy_stm32::pac::GPIOE.moder().modify(|w| w.set_moder(5, embassy_stm32::pac::gpio::vals::Moder::ALTERNATE));
    embassy_stm32::pac::GPIOE.moder().modify(|w| w.set_moder(6, embassy_stm32::pac::gpio::vals::Moder::ALTERNATE));
    // set trace pins to very high speed
    embassy_stm32::pac::GPIOE.ospeedr().modify(|w| w.set_ospeedr(2, embassy_stm32::pac::gpio::vals::Ospeedr::VERY_HIGH_SPEED));
    embassy_stm32::pac::GPIOE.ospeedr().modify(|w| w.set_ospeedr(3, embassy_stm32::pac::gpio::vals::Ospeedr::VERY_HIGH_SPEED));
    embassy_stm32::pac::GPIOE.ospeedr().modify(|w| w.set_ospeedr(4, embassy_stm32::pac::gpio::vals::Ospeedr::VERY_HIGH_SPEED));
    embassy_stm32::pac::GPIOE.ospeedr().modify(|w| w.set_ospeedr(5, embassy_stm32::pac::gpio::vals::Ospeedr::VERY_HIGH_SPEED));
    embassy_stm32::pac::GPIOE.ospeedr().modify(|w| w.set_ospeedr(6, embassy_stm32::pac::gpio::vals::Ospeedr::VERY_HIGH_SPEED));
    // set alternate function for trace pins
    embassy_stm32::pac::GPIOE.afr(0).modify(|w| w.set_afr(2, 0));
    embassy_stm32::pac::GPIOE.afr(0).modify(|w| w.set_afr(3, 0));
    embassy_stm32::pac::GPIOE.afr(0).modify(|w| w.set_afr(4, 0));
    embassy_stm32::pac::GPIOE.afr(0).modify(|w| w.set_afr(5, 0));
    embassy_stm32::pac::GPIOE.afr(0).modify(|w| w.set_afr(6, 0));

    //FIXME: program will halt when following lines are compiled *shrug*

    // // enable clock for IO port E
    // embassy_stm32::pac::RCC.ahb1enr().write(|w| w.set_gpioeen(true));
    // // set trace mode to 4 pin synchronous trace
    // embassy_stm32::pac::DBGMCU.cr().write(|w| w.set_trace_mode(3));
    // // enable clock
    // embassy_stm32::pac::DBGMCU.cr().write(|w| w.set_trace_ioen(true));
}

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let p = embassy_stm32::init(Default::default());

    setup_trace();

    let led1 = Output::new(p.PA8, Level::High, Speed::Low);
    let led2 = Output::new(p.PA9, Level::High, Speed::Low);
    let led3 = Output::new(p.PA10, Level::High, Speed::Low);

    spawner.spawn(handle_led(led1, &LED1_SIGNAL, "LED1", &LED2_SIGNAL, 500)).unwrap();
    spawner.spawn(handle_led(led2, &LED2_SIGNAL, "LED2", &LED3_SIGNAL, 200)).unwrap();
    spawner.spawn(handle_led(led3, &LED3_SIGNAL, "LED3", &MAIN_SIGNAL, 500)).unwrap();

    LED1_SIGNAL.signal(true);
    loop {
        LED1_SIGNAL.signal( !MAIN_SIGNAL.wait().await);
    }
}