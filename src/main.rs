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

#[embassy_executor::task(pool_size = 3)]
async fn handle_led(mut led: Output<'static>, signal: &'static Signal<ThreadModeRawMutex, bool>, name: &'static str) {
    loop {
        let state = signal.wait().await;
        led.set_level(
            match state {
                true => Level::Low,
                false => Level::High
            }
        );
        info!("Set {} to: {}", name, state);
    }
}

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let p = embassy_stm32::init(Default::default());
    info!("Hello World!");

    let led1 = Output::new(p.PA8, Level::High, Speed::Low);
    let led2 = Output::new(p.PA9, Level::High, Speed::Low);
    let led3 = Output::new(p.PA10, Level::High, Speed::Low);

    spawner.spawn(handle_led(led1, &LED1_SIGNAL, "LED1")).unwrap();
    spawner.spawn(handle_led(led2, &LED2_SIGNAL, "LED2")).unwrap();
    spawner.spawn(handle_led(led3, &LED3_SIGNAL, "LED3")).unwrap();

    let mut count = 1;

    loop {
        match count {
            1 => { LED1_SIGNAL.signal(true); }
            2 => { LED2_SIGNAL.signal(true); }
            3 => { LED3_SIGNAL.signal(true); }
            4 => { LED1_SIGNAL.signal(false); }
            5 => { LED2_SIGNAL.signal(false); }
            6 => { LED3_SIGNAL.signal(false); }
            _ => { count = 1; continue; }
        }
        count += 1;
        Timer::after_millis(500).await;
    }
}