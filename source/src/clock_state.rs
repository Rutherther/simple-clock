use stm32f1xx_hal::time::MonoTimer;

use crate::calendar::Calendar;

pub struct ClockState {
    calendar: Calendar,
    timer: MonoTimer,
}

impl ClockState {
    pub fn new(calendar: Calendar, timer: MonoTimer) -> Self {
        Self { calendar, timer }
    }

    pub fn timer(&self) -> MonoTimer {
        self.timer
    }

    pub fn calendar(&self) -> &Calendar {
        &self.calendar
    }

    pub fn mut_calendar(&mut self) -> &mut Calendar {
        &mut self.calendar
    }

    pub fn second_elapsed(&mut self) {
        self.calendar.second_elapsed()
    }
}
