use core::marker::PhantomData;

use alloc::boxed::Box;
use stm32f1xx_hal::rtc::Rtc;

use crate::{
    brightness_manager::BrightnessManager,
    button::ButtonState,
    clock_display_viewer::{ClockDisplayViewer, DisplayView},
    clock_state::ClockState,
};

pub struct ClockApp {
    rtc: Rtc,
    display: ClockDisplayViewer,
    state: ClockState,
    buttons: [Box<dyn ClockButton + Send>; 4],
    brightness: BrightnessManager,
    current_view: DisplayView,
}

struct AppState<'a> {
    rtc: &'a mut Rtc,
    display: &'a mut ClockDisplayViewer,
    state: &'a mut ClockState,
    brightness: &'a mut BrightnessManager,
    current_view: &'a mut DisplayView,
}

trait ClockButton {
    fn handle(&self, state: ButtonState, state: AppState);
}

struct ButtonSwitchView;
struct ButtonChangeTime;

struct Up;
struct Down;
struct ButtonBrightness<Direction> {
    direction: PhantomData<Direction>,
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
            current_view: DisplayView::ClockView,
            buttons: [
                Box::new(ButtonSwitchView),
                Box::new(ButtonChangeTime),
                Box::new(ButtonBrightness::<Down> {
                    direction: PhantomData::<Down>,
                }),
                Box::new(ButtonBrightness::<Up> {
                    direction: PhantomData::<Up>,
                }),
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
                self.brightness.update(&self.state);
                self.brightness.apply_brightness(&mut self.display);
            }
        }
    }

    pub fn handle_button(&mut self, index: usize, state: ButtonState) {
        self.buttons[index].handle(
            state,
            AppState {
                rtc: &mut self.rtc,
                display: &mut self.display,
                state: &mut self.state,
                brightness: &mut self.brightness,
                current_view: &mut self.current_view,
            },
        );
    }

    pub fn display(&mut self) -> &mut ClockDisplayViewer {
        &mut self.display
    }
}

impl ClockButton for ButtonSwitchView {
    fn handle(&self, state: ButtonState, app: AppState) {
        match state {
            ButtonState::JustPressed => {
                let display = app.display;
                let current_view = *app.current_view as usize;
                let new_view = ((current_view + 1) % core::mem::variant_count::<DisplayView>())
                    .try_into()
                    .unwrap();
                display.set_current_view(new_view);
                *app.current_view = new_view;
            }
            _ => (),
        }
    }
}

impl ClockButton for ButtonBrightness<Down> {
    fn handle(&self, state: ButtonState, app: AppState) {
        match state {
            ButtonState::JustPressed => {
                app.brightness
                    .set_brightness(app.brightness.brightness() as i8 - 10);
            }
            _ => (),
        }
    }
}

impl ClockButton for ButtonBrightness<Up> {
    fn handle(&self, state: ButtonState, app: AppState) {
        match state {
            ButtonState::JustPressed => {
                app.brightness
                    .set_brightness(app.brightness.brightness() as i8 + 10);
            }
            _ => (),
        }
    }
}

impl ClockButton for ButtonChangeTime {
    fn handle(&self, state: ButtonState, app: AppState) {
        match state {
            ButtonState::JustPressed => {
                app.rtc.set_time(app.rtc.current_time() + 2);
            }
            _ => (),
        }
    }
}

// to edit date, second button can be used
// to enter edit mode
// DateEditMode
//   editing_field
//     1. Hours, 2. Minutes, 3. Seconds,
//     4. Year,  5. Month,   6. Day
//   button functions:
//     1. next field
//     2. current_field++
//     3. current_field--
//     4. set
