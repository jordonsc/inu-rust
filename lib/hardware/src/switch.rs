//! Switch module for handling input from a button, NPN sensor, etc.

use core::cell::{Cell, RefCell};
use esp_idf_svc::hal::gpio::Level;
use inu_os::pin_mgr::GpioInput;
use std::time::{Duration, SystemTime};

/// Function signature for a callback executed when the switch state changes. The argument is the new state.
pub type OnToggle = fn(Level) -> ();

/// A switch that can be polled for state changes.
///
/// Using the polling is preferable to using an interrupt or manual testing as it enables the use of DelayOptions.
/// These are important to filtering out electrical interference.
pub struct InuSwitch<'s> {
    input: GpioInput<'s>,
    state: Cell<Level>,
    toggle_cb: Option<OnToggle>,
    delay_ops: DelayOptions,
    timer: RefCell<Option<SystemTime>>,
}

impl<'s> InuSwitch<'s> {
    /// Creates a new switch.
    ///
    /// If you're wiring so that the GPIO receives a voltage when the switch is closed, use Pull::Down.
    /// If you're wiring so that the GPIO is wired to the ground when closed, use Pull:Up.
    ///
    /// Switch has a default transition delay of 50ms if not specified.
    pub fn new(input: GpioInput<'s>) -> Self {
        let state = input.get_level();

        Self {
            input,
            state: Cell::new(state),
            toggle_cb: None,
            delay_ops: DelayOptions::default(),
            timer: RefCell::new(Some(SystemTime::now())),
        }
    }

    pub fn with_callback(mut self, cb: OnToggle) -> Self {
        self.toggle_cb = Some(cb);
        self
    }

    pub fn with_delay(mut self, delay_opts: DelayOptions) -> Self {
        self.delay_ops = delay_opts;
        self
    }

    pub fn set_callback(&mut self, c: OnToggle) {
        self.toggle_cb = Some(c);
    }

    pub fn set_delay_options(&mut self, delay_options: DelayOptions) {
        self.delay_ops = delay_options
    }

    /// Get the current state of the switch.
    ///
    /// Use this to query the state directly, if you want state change events to process properly, instead use poll().
    #[inline]
    pub fn is_active(&self) -> Level {
        self.input.get_level()
    }

    /// Poll the switch state and call the callback if the state has changed.
    pub fn poll(&self) {
        let is_active = self.is_active();
        let mut timer = self.timer.borrow_mut();

        if is_active == self.state.get() {
            if timer.is_some() {
                *timer = None;
            }
            return;
        }

        match self.delay_ops.min_transition_time {
            // If a delay is set, wait for the delay to pass before switching.
            Some(min_transition_time) => {
                if let Some(ref t) = *timer {
                    // Timer is running, check if it's time to switch
                    if t.elapsed().unwrap() >= min_transition_time {
                        *timer = None;
                        self.state.set(is_active);

                        if let Some(cb) = self.toggle_cb {
                            cb(self.state.get());
                        }
                    }
                } else {
                    // Start the timer, but take no action yet
                    *timer = Some(SystemTime::now());
                }
            }

            // If no delay is set, switch immediately.
            None => {
                self.state.set(is_active);

                if let Some(cb) = self.toggle_cb {
                    cb(self.state.get());
                }
            }
        }
    }
}

pub struct DelayOptions {
    /// The time that must lapse before acknowledging a state change.
    pub min_transition_time: Option<Duration>,
}

impl DelayOptions {
    pub fn none() -> Self {
        Self {
            min_transition_time: None,
        }
    }
}

impl Default for DelayOptions {
    fn default() -> Self {
        Self {
            min_transition_time: Some(Duration::from_millis(50)),
        }
    }
}
