#![no_std]
#![no_main]
#![feature(alloc_error_handler)]

extern crate alloc;

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

use alloc::boxed::Box;
use button::{ActiveHigh, Button, ButtonState};
use calendar::Calendar;
use clock_app::{ClockApp, ClockInterrupt};
use clock_display::{ClockDisplay, DisplayPart};
use clock_display_viewer::ClockDisplayViewer;
use clock_state::ClockState;
use core::{alloc::Layout, cell::RefCell, convert::Infallible, panic::PanicInfo};
use cortex_m::asm::wfi;
use cortex_m_rt::entry;
use count_down::{CountDown, CountDowner};
use critical_section::Mutex;
use display::Display;
use display_view::DisplayViews;
use embedded_alloc::Heap;
use embedded_hal::digital::v2::OutputPin;
use fugit::MicrosDurationU32;
use stm32f1xx_hal::{
    pac,
    pac::interrupt,
    prelude::*,
    rtc::{
        RestoredOrNewRtc::{New, Restored},
        Rtc,
    },
    time::MonoTimer,
    timer::{Event, Tim1NoRemap, Tim2NoRemap, Tim3NoRemap, TimerExt},
};

use defmt_rtt as _;

#[global_allocator]
static HEAP: Heap = Heap::empty();

static APP: Mutex<RefCell<Option<ClockApp>>> = Mutex::new(RefCell::new(Option::None));

#[interrupt]
fn RTC() {
    critical_section::with(|cs| {
        let mut app = APP.borrow_ref_mut(cs);
        let app = app.as_mut().unwrap();

        app.interrupt(ClockInterrupt::Rtc);
    });
}

#[interrupt]
fn TIM4() {
    critical_section::with(|cs| {
        let mut app = APP.borrow_ref_mut(cs);
        let app = app.as_mut().unwrap();

        app.interrupt(ClockInterrupt::DisplayTimer);
    });
}

#[entry]
fn main() -> ! {
    {
        use core::mem::MaybeUninit;
        const HEAP_SIZE: usize = 512;
        static mut HEAP_MEM: [MaybeUninit<u8>; HEAP_SIZE] = [MaybeUninit::uninit(); HEAP_SIZE];
        unsafe { HEAP.init(HEAP_MEM.as_ptr() as usize, HEAP_SIZE) }
    }

    // Get access to the core peripherals from the cortex-m crate
    let cp = cortex_m::Peripherals::take().unwrap();
    // Get access to the device specific peripherals from the peripheral access crate
    let dp = pac::Peripherals::take().unwrap();

    // Take ownership over the raw flash and rcc devices and convert them into the corresponding
    // HAL structs
    let mut pwr = dp.PWR;
    let mut flash = dp.FLASH.constrain();
    let rcc = dp.RCC.constrain();
    let mut backup_domain = rcc.bkp.constrain(dp.BKP, &mut pwr);

    // Freeze the configuration of all the clocks in the system and store the frozen frequencies in
    // `clocks`
    let clocks = rcc
        .cfgr
        .use_hse(8.MHz())
        .sysclk(24.MHz())
        .pclk1(24.MHz())
        .pclk2(24.MHz())
        .freeze(&mut flash.acr);

    let mut gpiob = dp.GPIOB.split();
    let mut gpioa = dp.GPIOA.split();
    let mut gpioc = dp.GPIOC.split();
    let mut afio = dp.AFIO.constrain();

    let (_, pb3, pb4) = afio.mapr.disable_jtag(gpioa.pa15, gpiob.pb3, gpiob.pb4);

    let mut led1 = gpiob.pb12.into_open_drain_output(&mut gpiob.crh);
    let mut led2 = gpiob.pb11.into_open_drain_output(&mut gpiob.crh);
    let mut led3 = gpiob.pb1.into_open_drain_output(&mut gpiob.crl);
    let mut led4 = gpiob.pb0.into_open_drain_output(&mut gpiob.crl);

    let leds: [&mut dyn OutputPin<Error = Infallible>; 4] =
        [&mut led1, &mut led2, &mut led3, &mut led4];

    let btn1 = Button::<ActiveHigh>::new(Box::new(gpiob.pb15.into_pull_down_input(&mut gpiob.crh)));
    let btn2 = Button::<ActiveHigh>::new(Box::new(gpiob.pb14.into_pull_down_input(&mut gpiob.crh)));
    let btn3 = Button::<ActiveHigh>::new(Box::new(gpiob.pb13.into_pull_down_input(&mut gpiob.crh)));
    let btn4 = Button::<ActiveHigh>::new(Box::new(gpioc.pc13.into_pull_down_input(&mut gpioc.crh)));

    let mut btns: [Button<ActiveHigh>; 4] = [btn1, btn2, btn3, btn4];

    let a = gpiob.pb10.into_open_drain_output(&mut gpiob.crh);
    let b = gpiob.pb2.into_open_drain_output(&mut gpiob.crl);
    let c = gpiob.pb8.into_open_drain_output(&mut gpiob.crh);
    let d = gpiob.pb6.into_open_drain_output(&mut gpiob.crl);
    let e = gpiob.pb9.into_open_drain_output(&mut gpiob.crh);
    let f = pb3.into_open_drain_output(&mut gpiob.crl);
    let g = pb4.into_open_drain_output(&mut gpiob.crl);
    let dpp = gpiob.pb7.into_open_drain_output(&mut gpiob.crl);

    let dig1 = gpioa.pa6.into_alternate_open_drain(&mut gpioa.crl);
    let dig2 = gpioa.pa3.into_alternate_open_drain(&mut gpioa.crl);
    let dig3 = gpioa.pa7.into_alternate_open_drain(&mut gpioa.crl);
    let dig4 = gpioa.pa8.into_alternate_open_drain(&mut gpioa.crh);
    let dig5 = gpioa.pa9.into_alternate_open_drain(&mut gpioa.crh);
    let dig6 = gpioa.pa2.into_alternate_open_drain(&mut gpioa.crl);
    let dig7 = gpioa.pa10.into_alternate_open_drain(&mut gpioa.crh);
    let dig8 = gpioa.pa1.into_alternate_open_drain(&mut gpioa.crl);

    let pwm_freq = 2.kHz();
    let pins1 = (dig4, dig5, dig7);
    let pwm1 = dp
        .TIM1
        .pwm_hz::<Tim1NoRemap, _, _>(pins1, &mut afio.mapr, pwm_freq, &clocks);

    let pins2 = (dig8, dig6, dig2);
    let pwm2 = dp
        .TIM2
        .pwm_hz::<Tim2NoRemap, _, _>(pins2, &mut afio.mapr, pwm_freq, &clocks);

    let pins3 = (dig1, dig3);
    let pwm3 = dp
        .TIM3
        .pwm_hz::<Tim3NoRemap, _, _>(pins3, &mut afio.mapr, pwm_freq, &clocks);

    let mut tim4 = dp.TIM4.counter_us(&clocks);
    tim4.listen(Event::Update);
    tim4.start(10.micros()).unwrap();

    let countdown: Box<dyn CountDown<Time = MicrosDurationU32> + Send> =
        Box::new(CountDowner::new(tim4));

    let (dig4, dig5, dig7) = pwm1.split();
    let (dig8, dig6, dig2) = pwm2.split();
    let (dig1, dig3) = pwm3.split();

    let display = Display::<8>::new(
        [
            Box::new(a),
            Box::new(b),
            Box::new(c),
            Box::new(d),
            Box::new(e),
            Box::new(f),
            Box::new(g),
            Box::new(dpp),
        ],
        [
            Box::new(dig1),
            Box::new(dig2),
            Box::new(dig3),
            Box::new(dig4),
            Box::new(dig5),
            Box::new(dig6),
            Box::new(dig7),
            Box::new(dig8),
        ],
        countdown,
    );

    let display = ClockDisplay::new(display);
    let mut display = ClockDisplayViewer::new(display);
    display.set_current_view(DisplayViews::ClockSecondsView);

    let mut rtc = match Rtc::restore_or_new(dp.RTC, &mut backup_domain) {
        Restored(rtc) => rtc,
        New(rtc) => rtc,
    };

    rtc.listen_seconds();

    // Initialize the state inside of a critical section,
    // to make sure that if a second will elapse during state initialization,
    // an interrupt will be called afterwards.
    critical_section::with(|cs| {
        let current_time = rtc.current_time();
        unsafe {
            // RTC interrupt cannot be called prior APP being Some,
            // otherwise a panic would result.
            cortex_m::peripheral::NVIC::unmask(interrupt::RTC);
        }

        let state = ClockState::new(
            Calendar::from_seconds(Calendar::new(0, 0, 0, 1, 7, 2023), current_time),
            MonoTimer::new(cp.DWT, cp.DCB, clocks),
        );

        let app = ClockApp::new(rtc, display, state);
        APP.borrow(cs).replace(Some(app));
    });

    unsafe {
        cortex_m::peripheral::NVIC::unmask(interrupt::TIM4);
    }

    let mut delay = cp.SYST.delay(&clocks);

    loop {
        for (i, btn) in btns.iter_mut().enumerate() {
            btn.update();

            if btn.is_pressed() {
                leds[i].set_low().unwrap();
            } else {
                leds[i].set_high().unwrap();
            }

            let state = btn.state();
            if state != ButtonState::Off {
                critical_section::with(|cs| {
                    let mut app = APP.borrow_ref_mut(cs);
                    let app = app.as_mut().unwrap();

                    app.handle_button(i, state);
                });
            }
        }

        delay.delay_ms(50u16);
    }
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    defmt::error!("{}", defmt::Display2Format(info));
    defmt::flush();

    cortex_m::peripheral::NVIC::mask(interrupt::RTC);
    critical_section::with(|cs| {
        let mut app = APP.borrow_ref_mut(cs);
        let app = app.as_mut().unwrap();
        let display = app.display();
        display.clear_current_view();
        let _ = display
            .clock_display()
            .show_text(DisplayPart::MainDisplay, "Erro");
        display.clock_display().hide(DisplayPart::SideDisplay1);
        display.clock_display().hide(DisplayPart::SideDisplay2);
    });

    loop {
        wfi();
    }
}

#[alloc_error_handler]
fn oom(_: Layout) -> ! {
    cortex_m::peripheral::NVIC::mask(interrupt::RTC);
    critical_section::with(|cs| {
        let mut app = APP.borrow_ref_mut(cs);
        let app = app.as_mut().unwrap();
        let display = app.display();
        display.clear_current_view();
        let _ = display
            .clock_display()
            .show_text(DisplayPart::MainDisplay, "oom");
    });

    loop {
        wfi();
    }
}
