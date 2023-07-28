pub struct SevenSegments;

impl SevenSegments {
    pub fn digit_to_segments(digit: u8) -> u8 {
        (match digit {
            0 => 0b1111110,
            1 => 0b0110000,
            2 => 0b1101101,
            3 => 0b1111001,
            4 => 0b0110011,
            5 => 0b1011011,
            6 => 0b1011111,
            7 => 0b1110000,
            8 => 0b1111111,
            9 => 0b1111011,
            _ => 0b0000001,
        }) << 1
    }

    pub fn letter_to_segments(letter: char) -> u8 {
        (match letter {
            'E' => 0b1001111,
            'r' => 0b0000101,
            'o' => 0b0011101,
            _ => 0b0000001,
        }) << 1
    }
}
