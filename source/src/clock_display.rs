use stm32f1xx_hal::timer;

use crate::{display::Display, seven_segments::SevenSegments};

const MAIN_DISPLAY_OFFSET: usize = 2;
const MAIN_DISPLAY_SIZE: usize = 4;
const SIDE_DISPLAY_1_OFFSET: usize = 0;
const SIDE_DISPLAY_1_SIZE: usize = 2;
const SIDE_DISPLAY_2_OFFSET: usize = 6;
const SIDE_DISPLAY_2_SIZE: usize = 2;

const COLON_DIGIT_1: usize = 3;
const COLON_DIGIT_2: usize = 4;

pub struct ClockDisplay {
    display: Display<8>,
    colon: bool,
}

#[derive(Copy, Clone)]
pub enum DisplayPart {
    Whole,
    MainDisplay,
    SideDisplay1,
    SideDisplay2,
}

#[derive(Debug)]
pub enum DisplayError {
    DoesNotFit,
}

impl ClockDisplay {
    pub fn new(display: Display<8>) -> Self {
        Self {
            display,
            colon: false,
        }
    }

    pub fn display(&mut self) -> &mut Display<8> {
        &mut self.display
    }

    pub fn show_ordinal(
        &mut self,
        part: DisplayPart,
        number: u32,
        pad: bool,
    ) -> Result<(), DisplayError> {
        self.show_number(part, number, pad)?;

        let offset = Self::get_part_offset(part);
        let size = Self::get_part_size(part);
        let target_index = offset + size - 1;

        let data = self.display.data()[target_index];
        self.display.set_digit(target_index, data | 1);

        Ok(())
    }

    pub fn show_number(
        &mut self,
        part: DisplayPart,
        number: u32,
        pad: bool
    ) -> Result<(), DisplayError> {
        let offset = Self::get_part_offset(part);
        let size = Self::get_part_size(part);

        self.show_number_at(offset, size, number, pad)
    }

    pub fn show_number_at(
        &mut self,
        offset: usize,
        size: usize,
        number: u32,
        pad: bool,
    ) -> Result<(), DisplayError> {
        let mut data = self.display.data();

        let mut number = number;
        for i in 1..=size {
            let digit = number % 10;
            number /= 10;

            data[offset + size - i] = if number != 0 || digit != 0 || pad {
                SevenSegments::digit_to_segments(digit as u8)
            } else {
                0
            };
        }

        if number > 0 {
            return Err(DisplayError::DoesNotFit);
        }

        self.display.set_data(data);
        self.update_colon();
        Ok(())
    }

    pub fn show_text(&mut self, part: DisplayPart, text: &str) -> Result<(), DisplayError> {
        let offset = Self::get_part_offset(part);
        let size = Self::get_part_size(part);

        if text.len() > size {
            return Err(DisplayError::DoesNotFit);
        }

        let mut data = self.display.data();
        for (i, c) in text.chars().enumerate() {
            data[offset + i] = SevenSegments::letter_to_segments(c);
        }

        self.display.set_data(data);
        self.update_colon();
        Ok(())
    }

    pub fn hide(&mut self, part: DisplayPart) {
        let offset = Self::get_part_offset(part);
        let size = Self::get_part_size(part);

        let mut data = self.display.data();
        for current_data in data.iter_mut().skip(offset).take(size) {
            *current_data = 0;
        }

        self.display.set_data(data);
    }

    pub fn brightness(&self, part: DisplayPart) -> &[u16] {
        let offset = Self::get_part_offset(part);
        let size = Self::get_part_size(part);
        &self.display.ref_brightness()[offset..offset + size]
    }

    pub fn set_brightness(&mut self, part: DisplayPart, brightness: u16) {
        let offset = Self::get_part_offset(part);
        let size = Self::get_part_size(part);
        let mut brightnesses = self.display.brightness();

        for current_brightness in brightnesses.iter_mut().skip(offset).take(size) {
            *current_brightness = brightness;
        }

        self.display.set_brightness(brightnesses);
    }

    pub fn set_colon(&mut self, colon: bool) {
        self.colon = colon;
        self.update_colon();
    }

    pub fn update(&mut self) -> nb::Result<(), timer::Error> {
        self.display.update()
    }

    pub fn get_part_size(part: DisplayPart) -> usize {
        match part {
            DisplayPart::Whole => SIDE_DISPLAY_1_SIZE + MAIN_DISPLAY_SIZE + SIDE_DISPLAY_2_SIZE,
            DisplayPart::MainDisplay => MAIN_DISPLAY_SIZE,
            DisplayPart::SideDisplay1 => SIDE_DISPLAY_1_SIZE,
            DisplayPart::SideDisplay2 => SIDE_DISPLAY_2_SIZE,
        }
    }

    pub fn get_part_offset(part: DisplayPart) -> usize {
        match part {
            DisplayPart::Whole => 0,
            DisplayPart::MainDisplay => MAIN_DISPLAY_OFFSET,
            DisplayPart::SideDisplay1 => SIDE_DISPLAY_1_OFFSET,
            DisplayPart::SideDisplay2 => SIDE_DISPLAY_2_OFFSET,
        }
    }

    fn update_colon(&mut self) {
        let data = self.display.data();
        if self.colon {
            self.display
                .set_digit(COLON_DIGIT_1, data[COLON_DIGIT_1] | 0b1);
            self.display
                .set_digit(COLON_DIGIT_2, data[COLON_DIGIT_2] | 0b1);
        } else {
            self.display
                .set_digit(COLON_DIGIT_1, data[COLON_DIGIT_1] & 0xFE);
            self.display
                .set_digit(COLON_DIGIT_2, data[COLON_DIGIT_2] & 0xFE);
        }
    }
}
