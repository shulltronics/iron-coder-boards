//! Iron Coder BSP for Feather nRF52840

#![no_std]

use embassy_nrf;
use embassy_nrf::gpio::{Level, Output, OutputDrive};

use embassy_time::Delay;
use embedded_hal::blocking::delay::DelayMs;

type OnBoardLed = Output<'static, embassy_nrf::peripherals::P1_15>;

pub struct Board {
    pub red_led: OnBoardLed,
}

impl Board {
    pub fn new() -> Self {
        let p = embassy_nrf::init(Default::default());
        let led = Output::new(p.P1_15, Level::Low, OutputDrive::Standard);
        Self {
            red_led: led,
        }
    }

    pub fn delay(&self, ms: u32) {
        // TODO -- why is this timer off? Something to do with the default impl of
        // the time-driver-rtc1 feature in embassy-nrf
        Delay.delay_ms(ms);
    }
}
