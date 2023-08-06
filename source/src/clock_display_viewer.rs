use core::mem;

use crate::{
    clock_display::{ClockDisplay, DisplayPart},
    clock_state::ClockState,
};
use stm32f1xx_hal::timer;

#[derive(Copy, Clone, PartialEq, Eq)]
#[repr(usize)]
pub enum DisplayView {
    ClockView = 0,
    ClockSecondsView = 1,
    ClockDateView = 2,
    DateView = 3,
}

impl TryFrom<usize> for DisplayView {
    fn try_from(value: usize) -> Result<Self, ()> {
        if value <= DisplayView::DateView as usize {
            unsafe { core::mem::transmute(value) }
        } else {
            Err(())
        }
    }

    type Error = ();
}

#[derive(Copy, Clone, PartialEq, Eq)]
#[repr(usize)]
pub enum ClockPart {
    Hours = 0,
    Minutes = 1,
    Seconds = 2,
    Year = 3,
    Month = 4,
    Day = 5,
}

impl TryFrom<usize> for ClockPart {
    fn try_from(value: usize) -> Result<Self, ()> {
        if value <= ClockPart::Day as usize {
            unsafe { core::mem::transmute(value) }
        } else {
            Err(())
        }
    }

    type Error = ();
}

pub struct ClockDisplayViewer {
    clock_display: ClockDisplay,
    parts: [bool; core::mem::variant_count::<ClockPart>()]
}

impl ClockDisplayViewer {
    pub fn new(clock_display: ClockDisplay) -> Self {
        Self {
            clock_display,
            parts: [false; core::mem::variant_count::<ClockPart>()]
        }
    }

    pub fn show(&mut self, part: ClockPart) {
        self.parts[part as usize] = true;
    }

    pub fn hide(&mut self, part: ClockPart) {
        self.parts[part as usize] = false;
    }

    pub fn hide_all(&mut self) {
        for part in self.parts.iter_mut() {
            *part = false;
        }

        self.clock_display.hide(DisplayPart::MainDisplay);
        self.clock_display.hide(DisplayPart::SideDisplay1);
        self.clock_display.hide(DisplayPart::SideDisplay2);
    }

    pub fn clock_display(&mut self) -> &mut ClockDisplay {
        &mut self.clock_display
    }

    pub fn set_current_view(&mut self, view: DisplayView) {
        self.hide_all();
        match view {
            DisplayView::ClockView => {
                self.show(ClockPart::Hours);
                self.show(ClockPart::Minutes);
            },
            DisplayView::ClockSecondsView => {
                self.show(ClockPart::Hours);
                self.show(ClockPart::Minutes);
                self.show(ClockPart::Seconds);
            },
            DisplayView::ClockDateView => {
                self.show(ClockPart::Hours);
                self.show(ClockPart::Minutes);
                self.show(ClockPart::Day);
                self.show(ClockPart::Month);
            },
            DisplayView::DateView => {
                self.show(ClockPart::Day);
                self.show(ClockPart::Month);
                self.show(ClockPart::Year);
            }
        }
    }

    pub fn update(&mut self, state: &ClockState) -> nb::Result<(), timer::Error> {
        self.clock_display.update()?;

        for (i, show) in self.parts.iter().enumerate().filter(|(_, x)| **x) {
            if !show {
                continue;
            }

            let part: ClockPart = ClockPart::try_from(i).unwrap();
            match part {
                ClockPart::Day => {
                    self.clock_display.show_ordinal(DisplayPart::SideDisplay1, state.calendar().day() as u32, true).unwrap();
                },
                ClockPart::Month => {
                    self.clock_display.show_ordinal(DisplayPart::SideDisplay2, state.calendar().month() as u32, true).unwrap();
                },
                ClockPart::Year => {
                    self.clock_display.show_number(DisplayPart::MainDisplay, state.calendar().year() as u32, true).unwrap();
                },
                ClockPart::Hours => {
                    self.clock_display.show_number_at(ClockDisplay::get_part_offset(DisplayPart::MainDisplay), 2, state.calendar().hours() as u32, true).unwrap();
                },
                ClockPart::Minutes => {
                    self.clock_display.show_number_at(ClockDisplay::get_part_offset(DisplayPart::MainDisplay) + 2, 2, state.calendar().minutes() as u32, true).unwrap();
                },
                ClockPart::Seconds => {
                    self.clock_display.show_number(DisplayPart::SideDisplay2, state.calendar().seconds() as u32, true).unwrap();
                },
            }
        }

        if self.parts[ClockPart::Hours as usize] && self.parts[ClockPart::Minutes as usize] {
            self.clock_display.set_colon(state.calendar().seconds() % 2 == 0);
        } else {
            self.clock_display.set_colon(false);
        }

        Ok(())
    }
}
