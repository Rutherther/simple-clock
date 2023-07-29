use crate::{
    clock_display::{ClockDisplay, DisplayPart},
    clock_state::ClockState,
};

use super::{DisplayView, DisplayViewError};

pub struct ClockDisplayView {
    show_seconds: bool,
    show_date: bool,
}

impl Default for ClockDisplayView {
    fn default() -> Self {
        Self::new()
    }
}

impl ClockDisplayView {
    pub fn new() -> Self {
        Self {
            show_date: false,
            show_seconds: false,
        }
    }

    pub fn with_seconds() -> Self {
        Self {
            show_date: false,
            show_seconds: true,
        }
    }

    pub fn with_date() -> Self {
        Self {
            show_date: true,
            show_seconds: false,
        }
    }
}

impl DisplayView for ClockDisplayView {
    fn update_display(
        &mut self,
        state: &ClockState,
        display: &mut ClockDisplay,
    ) -> Result<(), DisplayViewError> {
        let calendar = state.calendar();

        if self.show_seconds {
            display.hide(DisplayPart::SideDisplay1);
            display.show_number(DisplayPart::SideDisplay2, calendar.seconds() as u32, true)?;
        } else if self.show_date {
            display.show_ordinal(DisplayPart::SideDisplay1, calendar.day() as u32, true)?;
            display.show_ordinal(DisplayPart::SideDisplay2, calendar.month() as u32, true)?;
        } else {
            display.hide(DisplayPart::SideDisplay1);
            display.hide(DisplayPart::SideDisplay2);
        }

        display.set_colon(calendar.seconds() % 2 == 0);
        display.show_number(
            DisplayPart::MainDisplay,
            (calendar.hours() as u32) * 100 + (calendar.minutes() as u32),
            true,
        )?;
        Ok(())
    }
}
