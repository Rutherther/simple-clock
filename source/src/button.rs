use core::convert::Infallible;

use alloc::boxed::Box;
use core::marker::PhantomData;
use embedded_hal::digital::v2::InputPin;

pub struct ActiveHigh;
pub struct ActiveLow;

const DEBOUNCE: u8 = 2;
const LONG_PRESS: u8 = 5;

#[derive(PartialEq, Eq)]
pub enum ButtonState {
    Off,
    JustPressed,
    LongPress,
    Click,
    DoubleClick,
    Released,
}

pub struct Button<ActiveLevel> {
    pin: Box<dyn InputPin<Error = Infallible>>,
    debounce: u8,
    prev_state: bool,
    level: PhantomData<ActiveLevel>,
}

pub trait ActiveLevel {
    fn raw_is_pressed(pin_state: bool) -> bool;
}

impl ActiveLevel for ActiveHigh {
    fn raw_is_pressed(pin_state: bool) -> bool {
        pin_state
    }
}

impl ActiveLevel for ActiveLow {
    fn raw_is_pressed(pin_state: bool) -> bool {
        !pin_state
    }
}

impl<ACTIVELEVEL: ActiveLevel> Button<ACTIVELEVEL> {
    pub fn new(pin: Box<dyn InputPin<Error = Infallible>>) -> Self {
        Self {
            pin,
            debounce: 0,
            prev_state: false,
            level: PhantomData::<ACTIVELEVEL>,
        }
    }

    pub fn raw_is_pressed(&self) -> bool {
        ACTIVELEVEL::raw_is_pressed(self.pin.is_high().unwrap())
    }

    pub fn is_just_pressed(&self) -> bool {
        self.raw_is_pressed() && self.debounce == DEBOUNCE
    }

    pub fn is_pressed(&self) -> bool {
        self.raw_is_pressed() && self.debounce >= DEBOUNCE
    }

    pub fn is_long_pressed(&self) -> bool {
        self.raw_is_pressed() && self.debounce >= LONG_PRESS
    }

    pub fn state(&self) -> ButtonState {
        if !self.raw_is_pressed() {
            ButtonState::Off
        } else if self.is_just_pressed() {
            ButtonState::JustPressed
        } else if self.is_long_pressed() {
            ButtonState::LongPress
        } else {
            ButtonState::Off
        }
    }

    pub fn update(&mut self) {
        let raw_pressed = self.raw_is_pressed();

        if raw_pressed != self.prev_state {
            self.debounce = 0;
        } else {
            self.debounce = self.debounce.saturating_add(1);
        }

        self.prev_state = raw_pressed;
    }

    pub fn reset() {
        // resets is_clicked etc.
    }
}
