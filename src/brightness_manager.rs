use alloc::vec;

use crate::{
    clock_display_viewer::ClockDisplayViewer,
    clock_state::ClockState,
    linear_interpolation::{LinearInterpolation, Point},
};

pub struct BrightnessManager {
    yellow_interpolation: LinearInterpolation<u16, u16>,
    blue_interpolation: LinearInterpolation<u16, u16>,
    brightness_interpolation: LinearInterpolation<u16, u16>,
    current_brightness: u8,
    display_brightness: [u16; 8],
}

impl BrightnessManager {
    pub fn new() -> Self {
        Self {
            yellow_interpolation: LinearInterpolation::new(vec![
                Point::new(0, 0xFFFF - 11980),
                Point::new(1, 0xFFFF - 11970),
                Point::new(10, 0xFFFF - 11600),
                Point::new(20, 0xFFFF - 11200),
                Point::new(50, 0xFFFF - 9900),
                Point::new(100, 0xFFFF - 2500),
            ]),
            blue_interpolation: LinearInterpolation::new(vec![
                Point::new(0, 0xFFFF - 12000),
                Point::new(1, 0xFFFF - 11990),
                Point::new(10, 0xFFFF - 11700),
                Point::new(20, 0xFFFF - 11300),
                Point::new(50, 0xFFFF - 10000),
                Point::new(100, 0xFFFF - 3000),
            ]),
            brightness_interpolation: LinearInterpolation::new(vec![
                Point::new(0, 1),
                Point::new(6 * 60, 1),
                Point::new(8 * 60, 50),
                Point::new(12 * 60, 100),
                Point::new(18 * 60, 90),
                Point::new(20 * 60, 70),
                Point::new(21 * 60, 30),
                Point::new(22 * 60, 20),
                Point::new(23 * 60, 1),
                Point::new(24 * 60, 1),
            ]),
            current_brightness: 100,
            display_brightness: [0xFFFF; 8],
        }
    }

    pub fn set_brightness(&mut self, brightness: i8) {
        self.current_brightness = brightness.clamp(1, 100) as u8;

        let yellow_brightness = self
            .yellow_interpolation
            .interpolate(self.current_brightness as u16)
            .unwrap();
        let blue_brightness = self
            .blue_interpolation
            .interpolate(self.current_brightness as u16)
            .unwrap();

        for (i, brightness) in self.display_brightness.iter_mut().enumerate() {
            if i > 1 && i < 6 {
                *brightness = blue_brightness;
            } else {
                *brightness = yellow_brightness;
            }
        }
    }

    pub fn brightness(&self) -> u8 {
        self.current_brightness
    }

    pub fn apply_brightness(&self, display: &mut ClockDisplayViewer) {
        display
            .clock_display()
            .display()
            .set_brightness(self.display_brightness);
    }

    pub fn update(&mut self, state: &ClockState) {
        let calendar = state.calendar();
        let minutes_in_day = calendar.hours() as u16 * 60u16 + calendar.minutes() as u16;

        let interpolated = self
            .brightness_interpolation
            .interpolate(minutes_in_day)
            .unwrap();
        if self.brightness() != interpolated as u8 {
            self.set_brightness(interpolated as i8);
        }
    }
}
