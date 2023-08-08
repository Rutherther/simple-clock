use alloc::boxed::Box;
use stm32f1xx_hal::rtc::Rtc;

use crate::{
    brightness_manager::BrightnessManager,
    button::ButtonState,
    clock_display_viewer::ClockDisplayViewer,
    clock_state::ClockState, app_mode::{ClockAppMode, ClockAppModes, default_app_mode::DefaultAppMode, edit_app_mode::EditAppMode},
};

pub struct ClockApp {
    rtc: Rtc,
    display: ClockDisplayViewer,
    state: ClockState,
    modes: [Box<dyn ClockAppMode + Send>; core::mem::variant_count::<ClockAppModes>()],
    brightness: BrightnessManager,
    current_mode: ClockAppModes,
}

pub struct AppState<'a> {
    pub rtc: &'a mut Rtc,
    pub display: &'a mut ClockDisplayViewer,
    pub state: &'a mut ClockState,
    pub brightness: &'a mut BrightnessManager,
    pub current_mode: &'a mut ClockAppModes,
}

pub enum ClockInterrupt {
    Rtc,
    DisplayTimer,
}

impl ClockApp {
    pub fn new(rtc: Rtc, display: ClockDisplayViewer, state: ClockState) -> Self {
        Self {
            rtc,
            display,
            state,
            current_mode: ClockAppModes::NormalMode,
            modes: [
                Box::new(DefaultAppMode::new()),
                Box::new(EditAppMode::new())
            ],
            brightness: BrightnessManager::new(),
        }
    }

    pub fn interrupt(&mut self, interrupt: ClockInterrupt) {
        match interrupt {
            ClockInterrupt::Rtc => {
                self.state.second_elapsed();
                self.rtc.clear_second_flag();
            }
            ClockInterrupt::DisplayTimer => {
                let _ = self.display.update(&self.state);
                self.brightness.apply_brightness(&mut self.display);

                let mut mode = self.current_mode;
                let app_state = AppState {
                    rtc: &mut self.rtc,
                    display: &mut self.display,
                    state: &mut self.state,
                    brightness: &mut self.brightness,
                    current_mode: &mut mode,
                };
                self.modes[self.current_mode as usize].update(app_state);
            }
        }
    }

    pub fn handle_button(&mut self, index: usize, state: ButtonState) {
        let mut mode = self.current_mode;
        let current_mode = self.modes[self.current_mode as usize].as_mut();

        {
            let app_state = AppState {
                rtc: &mut self.rtc,
                display: &mut self.display,
                state: &mut self.state,
                brightness: &mut self.brightness,
                current_mode: &mut mode,
            };

            current_mode.handle_button(app_state, index, state);
        }

        if self.current_mode != mode {
            let mut temp_mode = mode;
            {
                current_mode.stop(AppState { rtc: &mut self.rtc, display: &mut self.display, state: &mut self.state, brightness: &mut self.brightness, current_mode: &mut temp_mode });
            }

            self.current_mode = temp_mode;

            let current_mode = self.modes[self.current_mode as usize].as_mut();
            current_mode.run(AppState { rtc: &mut self.rtc, display: &mut self.display, state: &mut self.state, brightness: &mut self.brightness, current_mode: &mut temp_mode });
        }
    }

    pub fn display(&mut self) -> &mut ClockDisplayViewer {
        &mut self.display
    }
}

