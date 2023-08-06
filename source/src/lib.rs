#![cfg_attr(test, no_main)]
#![no_std]
#![feature(variant_count)]

use defmt_rtt as _; // global logger

use embedded_alloc::Heap;

use panic_probe as _;

pub mod brightness_manager;
pub mod button;
pub mod calendar;
pub mod clock_app;
pub mod clock_display;
pub mod clock_display_viewer;
pub mod clock_state;
pub mod count_down;
pub mod display;
pub mod linear_interpolation;
pub mod mono_timer;
pub mod number_digits;
pub mod seven_segments;

extern crate alloc;

#[global_allocator]
static HEAP: Heap = Heap::empty();

// same panicking *behavior* as `panic-probe` but doesn't print a panic message
// this prevents the panic message being printed *twice* when `defmt::panic` is invoked
#[defmt::panic_handler]
fn panic() -> ! {
    cortex_m::asm::udf()
}

/// Terminates the application and makes `probe-run` exit with exit-code = 0
pub fn exit() -> ! {
    loop {
        cortex_m::asm::bkpt();
    }
}

#[cfg(test)]
#[defmt_test::tests]
mod unit_tests {
    use crate::{
        calendar::Calendar,
        linear_interpolation::{LinearInterpolation, Point},
    };
    use defmt::assert_eq;

    #[init]
    fn init() {
        use core::mem::MaybeUninit;
        const HEAP_SIZE: usize = 512;
        static mut HEAP_MEM: [MaybeUninit<u8>; HEAP_SIZE] = [MaybeUninit::uninit(); HEAP_SIZE];
        unsafe { crate::HEAP.init(HEAP_MEM.as_ptr() as usize, HEAP_SIZE) }
    }

    #[test]
    fn interpolation() {
        let li = LinearInterpolation::new(alloc::vec![
            Point::new(0, 0xFFFF - 12000),
            Point::new(100, 0xFFFF - 3000),
        ]);

        assert_eq!(li.interpolate(0).unwrap(), 0xFFFF - 12000);
        assert_eq!(li.interpolate(50).unwrap(), 0xFFFF - 7500);
        assert_eq!(li.interpolate(90).unwrap(), 0xFFFF - 3000 - 900);
        assert_eq!(li.interpolate(100).unwrap(), 0xFFFF - 3000);
    }

    #[test]
    fn calendar_to_leap_year() {
        let base = Calendar::new(0, 0, 0, 1, 1, 2023);
        let end_of_leap_year = Calendar::with_base_year(2023, 0, 0, 0, 31, 12, 2024);
        let end_of_leap_year_ticks = end_of_leap_year.to_ticks();
        assert_eq!(end_of_leap_year_ticks, (365 + 365) * 24 * 60 * 60);
        let end_of_leap_year_from_ticks =
            Calendar::from_ticks(base.clone(), end_of_leap_year_ticks);
        assert_eq!(end_of_leap_year_from_ticks.day(), 31);
        assert_eq!(end_of_leap_year_from_ticks.month(), 12);
        assert_eq!(end_of_leap_year_from_ticks.year(), 2024);
        assert_eq!(end_of_leap_year_from_ticks.hours(), 0);
        assert_eq!(end_of_leap_year_from_ticks.minutes(), 0);
        assert_eq!(end_of_leap_year_from_ticks.seconds(), 0);
    }

    #[test]
    fn calendar_past_leap_year() {
        let base = Calendar::new(0, 0, 0, 1, 1, 2023);
        let end_of_leap_year = Calendar::with_base_year(2023, 0, 0, 0, 31, 12, 2025);
        let end_of_leap_year_ticks = end_of_leap_year.to_ticks();
        assert_eq!(end_of_leap_year_ticks, (365 + 366 + 364) * 24 * 60 * 60);
        let end_of_leap_year_from_ticks =
            Calendar::from_ticks(base.clone(), end_of_leap_year_ticks);
        assert_eq!(end_of_leap_year_from_ticks.day(), 31);
        assert_eq!(end_of_leap_year_from_ticks.month(), 12);
        assert_eq!(end_of_leap_year_from_ticks.year(), 2025);
        assert_eq!(end_of_leap_year_from_ticks.hours(), 0);
        assert_eq!(end_of_leap_year_from_ticks.minutes(), 0);
        assert_eq!(end_of_leap_year_from_ticks.seconds(), 0);
    }

    #[test]
    fn calendar_basic_ticks() {
        let base = Calendar::new(0, 0, 0, 1, 1, 2023);
        assert_eq!(base.to_ticks(), 0);
        assert_eq!(Calendar::from_ticks(base.clone(), 0).to_ticks(), 0);
        assert_eq!(Calendar::from_ticks(base.clone(), 120).to_ticks(), 120);

        let two_days = Calendar::from_ticks(base.clone(), 2 * 24 * 60 * 60);
        assert_eq!(two_days.month(), 1);
        assert_eq!(two_days.year(), 2023);
        assert_eq!(two_days.day(), 3);
        assert_eq!(two_days.to_ticks(), 2 * 24 * 60 * 60);

        let one_month = Calendar::from_ticks(base.clone(), 31 * 24 * 60 * 60);
        assert_eq!(one_month.day(), 1);
        assert_eq!(one_month.month(), 2);
        assert_eq!(one_month.year(), 2023);
        assert_eq!(one_month.hours(), 0);
        assert_eq!(one_month.minutes(), 0);
        assert_eq!(one_month.seconds(), 0);
        assert_eq!(one_month.to_ticks(), 31 * 24 * 60 * 60);

        assert_eq!(
            Calendar::from_ticks(base.clone(), (31 + 28) * 24 * 60 * 60).to_ticks(),
            (31 + 28) * 24 * 60 * 60
        );
        assert_eq!(
            Calendar::from_ticks(base.clone(), 13423544).to_ticks(),
            13423544
        );
        assert_eq!(
            Calendar::from_ticks(base.clone(), 315360000).to_ticks(),
            315360000
        );
    }
}
