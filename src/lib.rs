#![no_main]
#![no_std]

use defmt_rtt as _; // global logger

use embedded_alloc::Heap;
use stm32f1xx_hal::prelude::*;

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
pub mod display_view;
pub mod linear_interpolation;
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
    use crate::linear_interpolation::{LinearInterpolation, Point};
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
}
