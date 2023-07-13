#![no_std]

pub mod prelude;
use embedded_hal::blocking::i2c;
use prelude::*;

use adafruit_feather_rp2040 as feather;

use feather::{Pins, XOSC_CRYSTAL_FREQ};
use feather::hal::{
	pac,
	pac::interrupt,
    clocks::{init_clocks_and_plls, Clock},
    watchdog::Watchdog,
	timer::Timer,
    Sio,
	gpio::pin::Pin,
	gpio::pin::PushPullOutput,
	gpio::pin::bank0::*,		// all gpios into scope
	gpio::FunctionI2C,
	I2C,
	pio::PIOExt,
};

use fugit::RateExtU32;

// a SYSTIC-based delay timer
use cortex_m::delay::Delay;
// struct for neopixels
use ws2812_pio::Ws2812Direct;

// USB Device support
use usb_device::class_prelude::*;
// USB Communications Class Device support
mod usb_manager;
use usb_manager::UsbManager;
use core::fmt::Write as _;

// Global USB objects & interrupt
static mut USB_BUS: Option<UsbBusAllocator<feather::hal::usb::UsbBus>> = None;
static mut USB_MANAGER: Option<UsbManager> = None;
#[allow(non_snake_case)]
#[interrupt]
unsafe fn USBCTRL_IRQ() {
    match USB_MANAGER.as_mut() {
        Some(manager) => manager.interrupt(),
        None => (),
    };
}

type OnboardLed = Pin<Gpio13, PushPullOutput>;

type OnboardNeopixel = Ws2812Direct<
    feather::hal::pac::PIO0,
    feather::hal::pio::SM0,
	feather::hal::gpio::pin::bank0::Gpio16,
>;

pub type I2CBus = I2C<
	pac::I2C1,
	(Pin<Gpio2, FunctionI2C>, Pin<Gpio3, FunctionI2C>)
>;

pub struct Board {
	red_led: OnboardLed,
	neopixel: OnboardNeopixel,
	pub i2c_bus: Option<I2CBus>,
	delay_timer: Delay,
	pub test: Option<bool>,
	usb: &'static mut UsbManager,
}

impl Board {
	pub fn new() -> Self {
		let mut pac = pac::Peripherals::take().unwrap();
    	let core = pac::CorePeripherals::take().unwrap();
		let mut watchdog = Watchdog::new(pac.WATCHDOG);
		let clocks = init_clocks_and_plls(
        	XOSC_CRYSTAL_FREQ,
        	pac.XOSC,
        	pac.CLOCKS,
        	pac.PLL_SYS,
        	pac.PLL_USB,
        	&mut pac.RESETS,
        	&mut watchdog,
    	).ok().unwrap();

   	 // Setup USB
    	let usb = unsafe {
        	USB_BUS = Some(UsbBusAllocator::new(feather::hal::usb::UsbBus::new(
            	pac.USBCTRL_REGS,
            	pac.USBCTRL_DPRAM,
            	clocks.usb_clock,
            	true,
            	&mut pac.RESETS,
        	)));
        	USB_MANAGER = Some(UsbManager::new(USB_BUS.as_ref().unwrap()));
        	// Enable the USB interrupt
        	feather::pac::NVIC::unmask(feather::hal::pac::Interrupt::USBCTRL_IRQ);
        	USB_MANAGER.as_mut().unwrap()
    	};

		// initialize the Single Cycle IO
    	let sio = Sio::new(pac.SIO);
    	// initialize the pins to default state
    	let pins = Pins::new(
        	pac.IO_BANK0,
        	pac.PADS_BANK0,
        	sio.gpio_bank0,
        	&mut pac.RESETS,
    	);

		// Setup the I2C1 instance, connected to the SCL/SDA pins on the Feather
		let scl = pins.scl.into_mode::<FunctionI2C>();
		let sda = pins.sda.into_mode::<FunctionI2C>();
		let i2c1: I2CBus = I2C::i2c1(
			pac.I2C1,
			sda,
			scl,
			RateExtU32::kHz(400),
			&mut pac.RESETS,
			&clocks.system_clock,
		);

		// setup the general-purpose delay timer
		let dt = Delay::new(core.SYST, clocks.system_clock.freq().to_Hz());

		// setup the on-board neopixel
		let _timer = Timer::new(pac.TIMER, &mut pac.RESETS);
		let (mut pio, sm0, _, _, _) = pac.PIO0.split(&mut pac.RESETS);
		let np = Ws2812Direct::new(
        	pins.neopixel.into_mode(),
        	&mut pio,
        	sm0,
        	clocks.peripheral_clock.freq(),
        	//timer.count_down(),
    	);

		Self {
			red_led: pins.d13.into_push_pull_output(),
			neopixel: np,
			i2c_bus: Some(i2c1),
			delay_timer: dt,
			test: Some(true),
			usb: usb,
		}
	}

	pub fn delay(&mut self, ms: u32) {
		self.delay_timer.delay_ms(ms);
	}

	pub fn set_led(&mut self, state: bool) {
		match state {
			true  => self.red_led.set_high().unwrap(),
			false => self.red_led.set_low().unwrap(),
		}
	}

	pub fn set_neopixel_color(&mut self, color: smart_leds::RGB8) {
		let _ = self.neopixel.write(brightness([color].iter().cloned(), 50));
	}

	pub fn serial_write(&mut self, s: impl core::fmt::Debug) {
		write!(self.usb, "{:?}\r\n", s).unwrap();
	}

}