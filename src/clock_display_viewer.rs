use crate::{
    clock_display::ClockDisplay,
    clock_state::ClockState,
    display_view::{
        clock_display_view::ClockDisplayView, date_display_view::DateDisplayView, DisplayView,
        DisplayViews,
    },
};
use alloc::{boxed::Box, vec, vec::Vec};
use stm32f1xx_hal::timer;

pub struct ClockDisplayViewer {
    clock_display: ClockDisplay,
    views: Vec<Box<dyn DisplayView + Send>>,
    current_view: Option<DisplayViews>,
}

impl ClockDisplayViewer {
    pub fn new(clock_display: ClockDisplay) -> Self {
        Self {
            clock_display,
            views: vec![
                Box::new(ClockDisplayView::new()),
                Box::new(ClockDisplayView::with_seconds()),
                Box::new(ClockDisplayView::with_date()),
                Box::new(DateDisplayView::new()),
            ],
            current_view: None,
        }
    }

    pub fn clock_display<'a>(&'a mut self) -> &'a mut ClockDisplay {
        &mut self.clock_display
    }

    pub fn current_view(&self) -> Option<DisplayViews> {
        self.current_view
    }

    pub fn set_current_view(&mut self, view: DisplayViews) {
        self.current_view = Some(view);
    }

    pub fn clear_current_view(&mut self) {
        self.current_view = None;
    }

    pub fn update(&mut self, state: &ClockState) -> nb::Result<(), timer::Error> {
        self.clock_display.update()?;

        if let Some(view) = self.current_view {
            let view = &mut self.views[view as usize];
            view.update_display(state, &mut self.clock_display).unwrap(); // TODO: get rid of the unwrap
        }
        Ok(())
    }
}
