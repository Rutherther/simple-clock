use core::cmp::{max, min};

#[derive(Clone, PartialEq, Eq)]
pub struct Calendar {
    base_year: u16,
    frozen: bool,
    hours: u8,
    minutes: u8,
    seconds: u8,
    day: u8,
    month: u8,
    year: u16,
}

impl Calendar {
    pub fn new(hours: u8, minutes: u8, seconds: u8, day: u8, month: u8, year: u16) -> Self {
        Self::with_base_year(year, hours, minutes, seconds, day, month, year)
    }

    /// Calendar may have a base year, that will be used when calling [to_ticks](`Calendar::to_ticks`).
    pub fn with_base_year(
        base_year: u16,
        hours: u8,
        minutes: u8,
        seconds: u8,
        day: u8,
        month: u8,
        year: u16,
    ) -> Self {
        Self {
            base_year,
            hours,
            minutes,
            seconds,
            day,
            month,
            year,
            frozen: false,
        }
    }

    /// Calculate current date based off of the seconds elapsed since
    /// a base date
    pub fn from_ticks(base_year: u16, mut seconds: u32) -> Self {
        let total_seconds = seconds;
        let seconds = total_seconds % 60;

        let total_minutes = total_seconds / 60;
        let minutes = total_minutes % 60;

        let total_hours = total_minutes / 60;
        let hours = total_hours % 24;

        let elapsed_days = total_hours / 24;
        // elapsed_days / 365 / 4 subtracts leap days each 4 years, except for the last leap day
        // that could have been in the last 3 years, solution for that day is presented
        // below.
        let estimated_years_elapsed = (elapsed_days - elapsed_days / 365 / 4) / 365;
        let estimated_year = base_year as u32 + estimated_years_elapsed;

        // estimated_years_elapsed may not be correct, imagine a situation,
        // the current year is a leap year, ie. 2024, base is 2021 (1. 1. 00:00:00),
        // it's 31. 12. 2024, the problem is that the last leap year, 2024, is not captured
        // by elapsed_days / 365 / 4 in this case.
        // If we did not subtract a day from the elapsed days, the program would
        // incorrectly conclude it's 1. 1. 2025.
        let mut elapsed_days_without_leap_day_in_past_3_years = total_hours / 24;
        // uhh... naming of this property is 1. long, 2. still not correct,
        // the variable should have leap days that were in last 3 years,
        // but not in the current one.
        for current_year in max(base_year as u32, estimated_year - 3)..=estimated_year - 1 {
            elapsed_days_without_leap_day_in_past_3_years -=
                if Self::is_leap_year(current_year as u16) {
                    1
                } else {
                    0
                };
        }

        // this should be the correct year
        let years_elapsed = (elapsed_days_without_leap_day_in_past_3_years
            - elapsed_days_without_leap_day_in_past_3_years / 365 / 4)
            / 365;
        let year = base_year as u32 + years_elapsed;

        if year != estimated_year && Self::is_leap_year(year as u16) {
            // we went back one year, the current year is a leap year,
            // because leap day is included in Self::days_in_month,
            // it should not be present in the elapsed days...
            elapsed_days_without_leap_day_in_past_3_years += 1;
        }

        let leap_year = Self::is_leap_year(year as u16);

        let days_from_year_start =
            elapsed_days_without_leap_day_in_past_3_years - years_elapsed * 365 - years_elapsed / 4;
        let mut month = 1;
        let mut day = days_from_year_start;
        let mut total_days_in_month = Self::days_in_month(month, leap_year) as u32;
        while day >= total_days_in_month {
            day -= total_days_in_month;
            month += 1;
            total_days_in_month = Self::days_in_month(month, leap_year) as u32;
        }
        day += 1;

        Self {
            base_year,
            seconds: seconds as u8,
            minutes: minutes as u8,
            hours: hours as u8,
            day: day as u8,
            month: month as u8,
            year: year as u16,
            frozen: false,
        }
    }

    /// Converts the date into ticks, elapsed seconds
    /// from base year, that was specified upon creation of the calendar.
    pub fn to_ticks(&self) -> u32 {
        let mut ticks = 0u32;

        ticks += self.seconds as u32;
        ticks += self.minutes as u32 * 60;
        ticks += self.hours as u32 * 60 * 60;
        ticks += (self.day - 1) as u32 * 24 * 60 * 60;
        ticks +=
            Self::days_in_year(self.month, Self::is_leap_year(self.year)) as u32 * 24 * 60 * 60;
        let elapsed_years = self.year as u32 - self.base_year as u32;
        ticks += (elapsed_years * 365 + elapsed_years / 4) * 24 * 60 * 60;

        for year in max(self.year - 3, self.base_year)..=self.year - 1 {
            ticks += if Self::is_leap_year(year) {
                24 * 60 * 60
            } else {
                0
            };
        }

        ticks
    }

    /// Adds a second, correctly updating
    /// minutes, hours, days, month, year...
    pub fn second_elapsed(&mut self) {
        if self.frozen {
            return;
        }

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

    /// Sets the current hour of the day,
    /// gets clamped to 0 - 24.
    pub fn set_hours(&mut self, hours: u8) {
        self.hours = hours.clamp(0, 23);
    }

    /// Sets the current minute of the hour,
    /// gets clamped to 0 - 59.
    pub fn set_minutes(&mut self, minutes: u8) {
        self.minutes = minutes.clamp(0, 59);
    }

    /// Sets the current seconds of the minute,
    /// gets clamped to 0 - 59.
    pub fn set_seconds(&mut self, seconds: u8) {
        self.seconds = seconds.clamp(0, 59);
    }

    /// Sets the current day of the month,
    /// gets clamped to 1 - days in the month.
    pub fn set_day(&mut self, day: u8) {
        self.day = day.clamp(1, Self::days_in_month(self.month, Self::is_leap_year(self.year)));
    }

    /// Sets the current day of the month,
    /// gets clamped to 1 - days in the month.
    pub fn set_month(&mut self, month: u8) {
        self.month = month.clamp(1, 12);
    }

    /// Sets the current year,
    /// The minimum is the base year specified
    /// upon Calendar creation. Lower year
    /// will be adjusted to base year.
    pub fn set_year(&mut self, year: u16) {
        self.year = max(year, self.base_year);
    }

    fn is_leap_year(year: u16) -> bool {
        matches!(year % 4, 0 if year % 100 != 0 || year % 400 == 0)
    }

    pub fn freeze(&mut self) {
        self.frozen = true;
    }

    pub fn unfreeze(&mut self) {
        self.frozen = true;
    }

    fn days_in_month(month: u8, leap_year: bool) -> u8 {
        match month {
            2 if leap_year => 29,
            2 => 28,
            1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
            4 | 6 | 9 | 11 => 30,
            _ => panic!("Month {month} does not exist, cannot get days in that month."),
        }
    }

    fn days_in_year(before_month: u8, leap_year: bool) -> u16 {
        let mut days_in_year = 0u16;
        for i in 1..before_month {
            days_in_year += Self::days_in_month(i, leap_year) as u16;
        }

        days_in_year
    }
}
