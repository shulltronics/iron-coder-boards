//! Propmaker Featherwing Iron Coder BSP

#![no_std]

use lis3dh::{Lis3dh, Lis3dhI2C, SlaveAddr};
use lis3dh::accelerometer::Accelerometer;
use embedded_hal::blocking::i2c::{WriteRead, Write};

pub struct Board<I2C> 
    where I2C: WriteRead + Write
{
    accelerometer: Option<Lis3dh<Lis3dhI2C<I2C>>>,
    // other parts, LED driver, neopixel driver, etc
}

impl<T, E> Board<T>
    where T: WriteRead<Error = E> + Write<Error = E>, E: core::fmt::Debug
{
    // Create a new PropMaker. This should probably take arguments of the
    // configured interfaces from the programmable board.
    pub fn new() -> Self {
        Self {
            accelerometer: None,
        }
    }

    // Initialize and store the accelerometer object, using an object from the
    // programmable board that implements the embedded_hal I2C traits.
    pub fn init_accelerometer(&mut self, i2c: T) {
        let a = Lis3dh::new_i2c(i2c, SlaveAddr::Default).unwrap();
        self.accelerometer = Some(a);
    }

    pub fn read_accelerometer(&mut self) -> Result<f32, ()> {
        if let Some(ref mut a) = self.accelerometer {
            Ok(a.accel_norm().unwrap()[0])
        } else {
            Err(())
        }
    }
}