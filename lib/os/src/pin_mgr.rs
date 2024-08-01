use std::cell::RefCell;
use std::sync::Mutex;

use crate::error::PinError;
use crate::physical::hardware;
use esp_idf_svc::hal::gpio::{AnyIOPin, Input, Level, Output, PinDriver, Pull};

pub type GpioInput<'a> = PinDriver<'a, AnyIOPin, Input>;
pub type GpioOutput<'a> = PinDriver<'a, AnyIOPin, Output>;

pub struct PinManager {
    pin_state: Mutex<RefCell<[bool; hardware::MAX_PINS as usize]>>,
}

impl PinManager {
    /// Create a new kernel instance.
    ///
    /// # Safety
    /// Singleton. Create only once.
    pub unsafe fn new() -> Self {
        Self {
            pin_state: Mutex::new(RefCell::new([false; hardware::MAX_PINS as usize])),
        }
    }

    /// Get a pin from the pin manager.
    ///
    /// This takes the pin, once taken, the pin cannot be retaken.
    pub fn get_pin(&self, pin: u8) -> Result<AnyIOPin, PinError> {
        if pin > hardware::MAX_PINS {
            return Err(PinError::InvalidPin(pin));
        }

        let lock = self.pin_state.lock();
        if lock.is_err() {
            return Err(PinError::Generic {
                pin,
                error: format!("Pin mutex poisoned: {:?}", lock.unwrap_err()),
            });
        }

        let mg = lock.unwrap();
        let mut ps = mg.borrow_mut();
        if ps[pin as usize] {
            return Err(PinError::PinInUse(pin));
        }
        ps[pin as usize] = true;

        Ok(unsafe { AnyIOPin::new(pin as i32) })
    }

    /// Get a pin and designate it as an input.
    pub fn get_input(&self, pin: u8, pull: Pull) -> Result<GpioInput, PinError> {
        let p = self.get_pin(pin)?;
        let mut input = PinDriver::input(p).unwrap();

        input.set_pull(pull).map_err(|e| PinError::Generic {
            pin,
            error: format!("Failed to set pin pull mode: {:?}", e),
        })?;

        Ok(input)
    }

    /// Get a pin and designate it as an output.
    pub fn get_output(&self, pin: u8, level: Level) -> Result<GpioOutput, PinError> {
        let p = self.get_pin(pin)?;
        let mut output = PinDriver::output(p).unwrap();

        output.set_level(level).map_err(|e| PinError::Generic {
            pin,
            error: format!("Failed to set pin level: {:?}", e),
        })?;

        Ok(output)
    }
}
