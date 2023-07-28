use crate::clock_display::DisplayPart;

use super::DisplayView;

pub struct DateDisplayView;

impl DateDisplayView {
    pub fn new() -> Self {
        Self
    }
}

impl Default for DateDisplayView {
    fn default() -> Self {
        Self::new()
    }
}

impl DisplayView for DateDisplayView {
    fn update_display(
        &mut self,
        state: &crate::clock_state::ClockState,
        display: &mut crate::clock_display::ClockDisplay,
    ) -> Result<(), super::DisplayViewError> {
        let calendar = state.calendar();

        display.show_ordinal(DisplayPart::SideDisplay1, calendar.day() as u32, true)?;
        display.show_ordinal(DisplayPart::SideDisplay2, calendar.month() as u32, true)?;
        display.show_number(DisplayPart::MainDisplay, calendar.year() as u32, false)?;
        display.set_colon(false);
        Ok(())
    }
}
