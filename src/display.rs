use core::convert::Infallible;

use alloc::boxed::Box;
use embedded_hal::PwmPin;
use fugit::MicrosDurationU32;
use stm32f1xx_hal::timer;

use crate::count_down::CountDown;

type OutputPin = dyn embedded_hal::digital::v2::OutputPin<Error = Infallible> + Send;

// How long to turn on a digit to show a number
// (every digit will be turned on for this time, so this number shouldn't bee to large - so it's not flickering,
// and it shouldn't be too small to allow the transistor and LED operate)
const DIGIT_ON_TIME: MicrosDurationU32 = MicrosDurationU32::micros(1490);

// How long to turn off the digits when moving from one digit to another
// This is important to close off the previous transistor before next digit
// is lit up.
const DIGITS_OFF_TIME: MicrosDurationU32 = MicrosDurationU32::micros(500);

struct DisplayState<const DIGITS: usize> {
    digit_index: usize,
    next_show: bool, // if 1, next timer step is to show, if 0, next timer step is to hide
}

impl<const DIGITS: usize> DisplayState<DIGITS> {
    fn empty() -> Self {
        Self {
            digit_index: 0,
            next_show: true,
        }
    }

    fn step(&mut self) {
        self.next_show = !self.next_show;
        if self.next_show {
            self.digit_index = (self.digit_index + 1) % DIGITS;
        }
    }
}

pub struct Display<const DIGITS: usize> {
    segments: [Box<OutputPin>; 8],
    digits: [Box<dyn PwmPin<Duty = u16> + Send>; DIGITS],
    timer: Box<dyn CountDown<Time = MicrosDurationU32> + Send>,
    data: [u8; DIGITS],
    brightness: [u16; DIGITS],

    state: DisplayState<DIGITS>,
}

impl<const DIGITS: usize> Display<DIGITS> {
    pub fn new(
        segments: [Box<OutputPin>; 8],
        mut digits: [Box<dyn PwmPin<Duty = u16> + Send>; DIGITS],
        timer: Box<dyn CountDown<Time = MicrosDurationU32> + Send>,
    ) -> Self {
        for digit in digits.iter_mut() {
            digit.enable();
        }

        Self {
            segments,
            digits,
            timer,
            data: [0; DIGITS],
            brightness: [0xFFFF; DIGITS],
            state: DisplayState::<DIGITS>::empty(),
        }
    }

    pub fn data(&self) -> [u8; DIGITS] {
        self.data
    }

    pub fn set_data(&mut self, digits: [u8; DIGITS]) {
        self.data = digits;
    }

    pub fn set_digit(&mut self, digit: usize, set: u8) {
        self.data[digit] = set;
    }

    pub fn brightness(&self) -> [u16; DIGITS] {
        self.brightness
    }

    pub fn ref_brightness(&self) -> &[u16] {
        &self.brightness
    }

    pub fn set_brightness(&mut self, brightness: [u16; DIGITS]) {
        self.brightness = brightness;
    }

    pub fn set_digit_brightness(&mut self, digit: usize, brightness: u16) {
        self.brightness[digit] = brightness;
    }

    pub fn update(&mut self) -> nb::Result<(), timer::Error> {
        self.timer.wait()?;
        let now_show = self.state.next_show;
        let digit_index = self.state.digit_index;

        // turn every digit off
        for digit in self.digits.iter_mut() {
            digit.set_duty(0xFFFF);
        }

        if now_show {
            let digit = &mut self.digits[digit_index];
            let data = self.data[digit_index];
            let brightness = self.brightness[digit_index];

            for (i, segment) in self.segments.iter_mut().enumerate() {
                segment
                    .set_state((!(data & (1 << (7 - i)) > 0)).into())
                    .unwrap();
            }

            digit.set_duty(0xFFFF - brightness);
        }

        self.timer.start(if now_show {
            DIGIT_ON_TIME
        } else {
            DIGITS_OFF_TIME
        });
        self.state.step();

        Ok(())
    }
}
