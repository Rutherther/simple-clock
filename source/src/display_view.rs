use crate::{
    clock_display::{ClockDisplay, DisplayError},
    clock_state::ClockState,
};

pub mod clock_display_view;
pub mod date_display_view;

#[derive(Copy, Clone, PartialEq, Eq)]
#[repr(usize)]
pub enum DisplayViews {
    ClockView = 0,
    ClockSecondsView = 1,
    ClockDateView = 2,
    DateView = 3,
}

impl DisplayViews {
    pub fn count() -> usize {
        4
    }
}

impl From<usize> for DisplayViews {
    fn from(value: usize) -> Self {
        match value {
            0 => Self::ClockView,
            1 => Self::ClockSecondsView,
            2 => Self::ClockDateView,
            3 => Self::DateView,
            _ => panic!(),
        }
    }
}

#[derive(Debug)]
pub enum DisplayViewError {
    Unknown,
    DisplayError(DisplayError),
}

impl From<DisplayError> for DisplayViewError {
    fn from(value: DisplayError) -> Self {
        Self::DisplayError(value)
    }
}

pub trait DisplayView {
    fn update_display(
        &mut self,
        state: &ClockState,
        display: &mut ClockDisplay,
    ) -> Result<(), DisplayViewError>;
}
