#![no_std]
#![no_main]

// Tell the compiler where our program should begin.
use cortex_m_rt::entry;
// Define the behavior when the firmware crashes.
use panic_halt as _;

mod system;
use system::System;

#[entry]
fn main() -> ! {
    let mut system = System::new();
    // put your setup code here!

    loop {
        // put your look code here!
    }
}