pub trait NumberDigits {
    fn get_digit(self, digit_index: u8) -> u8;
}

impl NumberDigits for u8 {
    fn get_digit(self, digit_index: u8) -> u8 {
        let mut number = self;
        for _ in 0..digit_index {
            number /= 10;
        }
        number % 10
    }
}

impl NumberDigits for u16 {
    fn get_digit(self, digit_index: u8) -> u8 {
        let mut number = self;
        for _ in 0..digit_index {
            number /= 10;
        }
        (number % 10) as u8
    }
}

impl NumberDigits for u32 {
    fn get_digit(self, digit_index: u8) -> u8 {
        let mut number = self;
        for _ in 0..digit_index {
            number /= 10;
        }
        (number % 10) as u8
    }
}
