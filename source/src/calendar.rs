pub struct Calendar {
    hours: u8,
    minutes: u8,
    seconds: u8,
    day: u8,
    month: u8,
    year: u16,
}

impl Calendar {
    pub fn new(hours: u8, minutes: u8, seconds: u8, day: u8, month: u8, year: u16) -> Self {
        Self {
            hours,
            minutes,
            seconds,
            day,
            month,
            year,
        }
    }

    pub fn from_seconds(base: Calendar, seconds: u32) -> Self {
        let total_seconds = seconds + base.seconds as u32;
        let seconds = total_seconds % 60;

        let total_minutes = total_seconds / 60 + base.minutes as u32;
        let minutes = total_minutes % 60;

        let total_hours = total_minutes / 60 + base.hours as u32;
        let hours = total_hours % 24;

        let total_days = total_hours / 24 + base.day as u32;
        let day = total_days % 30; // TODO...
                                   // the current month and year has to be known prior to the calculation of the day

        Self {
            seconds: seconds as u8,
            minutes: minutes as u8,
            hours: hours as u8,
            day: day as u8,
            month: base.month,
            year: base.year,
        }
    }

    pub fn hours(&self) -> u8 {
        self.hours
    }

    pub fn minutes(&self) -> u8 {
        self.minutes
    }

    pub fn seconds(&self) -> u8 {
        self.seconds
    }

    pub fn day(&self) -> u8 {
        self.day
    }

    pub fn month(&self) -> u8 {
        self.month
    }

    pub fn year(&self) -> u16 {
        self.year
    }

    pub fn second_elapsed(&mut self) {
        self.seconds = (self.seconds + 1) % 60;
        let minute_elapsed = self.seconds == 0;
        self.minutes = (self.minutes + if minute_elapsed { 1 } else { 0 }) % 60;

        let hour_elapsed = minute_elapsed && self.minutes == 0;
        self.hours = (self.hours + if hour_elapsed { 1 } else { 0 }) % 24;

        let day_elapsed = hour_elapsed && self.hours == 0;
        let day = self.day - 1 + if day_elapsed { 1 } else { 0 };
        let days_in_month = Self::days_in_month(self.month, Self::is_leap_year(self.year));
        self.day = day % days_in_month + 1;

        let month_elapsed = day_elapsed && self.day == 1;
        self.month = (self.month - 1 + if month_elapsed { 1 } else { 0 }) % 12 + 1;
        let year_elapsed = month_elapsed && self.month == 1;
        self.year += if year_elapsed { 1 } else { 0 };
    }

    fn is_leap_year(year: u16) -> bool {
        matches!(year % 4, 0 if year % 100 != 0 || year % 400 == 0)
    }

    fn days_in_month(month: u8, leap_year: bool) -> u8 {
        match month {
            2 if leap_year => 29,
            2 => 28,
            1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
            4 | 6 | 9 | 11 => 31,
            _ => panic!(),
        }
    }
}
