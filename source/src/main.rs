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
    afio::MAPR,
    gpio::{Cr, Floating, Input, Pin},
    pac,
    pac::interrupt,
    prelude::*,
    rcc::Clocks,
    rtc::{
        RestoredOrNewRtc::{New, Restored},
        Rtc,
    },
    time::MonoTimer,
    timer::{Event, SysDelay, Tim1NoRemap, Tim2NoRemap, Tim3NoRemap, TimerExt},
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

/// Puts given pins into open drain,
/// then makes PWM out of digit pins,
/// lastly, timer 4 is constructed for refreshing the display,
/// and ClockDisplayViewer is created.
fn init_segment_display(
    pb10: Pin<'B', 10, Input<Floating>>,
    pb2: Pin<'B', 2, Input<Floating>>,
    pb8: Pin<'B', 8, Input<Floating>>,
    pb6: Pin<'B', 6, Input<Floating>>,
    pb9: Pin<'B', 9, Input<Floating>>,
    pb3: Pin<'B', 3, Input<Floating>>,
    pb4: Pin<'B', 4, Input<Floating>>,
    pb7: Pin<'B', 7, Input<Floating>>,
    pa6: Pin<'A', 6, Input<Floating>>,
    pa3: Pin<'A', 3, Input<Floating>>,
    pa7: Pin<'A', 7, Input<Floating>>,
    pa8: Pin<'A', 8, Input<Floating>>,
    pa9: Pin<'A', 9, Input<Floating>>,
    pa2: Pin<'A', 2, Input<Floating>>,
    pa10: Pin<'A', 10, Input<Floating>>,
    pa1: Pin<'A', 1, Input<Floating>>,
    tim1: pac::TIM1,
    tim2: pac::TIM2,
    tim3: pac::TIM3,
    tim4: pac::TIM4,
    gpioa_crl: &mut Cr<'A', false>,
    gpioa_crh: &mut Cr<'A', true>,
    gpiob_crl: &mut Cr<'B', false>,
    gpiob_crh: &mut Cr<'B', true>,
    afio_mapr: &mut MAPR,
    clocks: &Clocks,
) -> ClockDisplayViewer {
    let a = pb10.into_open_drain_output(gpiob_crh);
    let b = pb2.into_open_drain_output(gpiob_crl);
    let c = pb8.into_open_drain_output(gpiob_crh);
    let d = pb6.into_open_drain_output(gpiob_crl);
    let e = pb9.into_open_drain_output(gpiob_crh);
    let f = pb3.into_open_drain_output(gpiob_crl);
    let g = pb4.into_open_drain_output(gpiob_crl);
    let dpp = pb7.into_open_drain_output(gpiob_crl);

    let dig1 = pa6.into_alternate_open_drain(gpioa_crl);
    let dig2 = pa3.into_alternate_open_drain(gpioa_crl);
    let dig3 = pa7.into_alternate_open_drain(gpioa_crl);
    let dig4 = pa8.into_alternate_open_drain(gpioa_crh);
    let dig5 = pa9.into_alternate_open_drain(gpioa_crh);
    let dig6 = pa2.into_alternate_open_drain(gpioa_crl);
    let dig7 = pa10.into_alternate_open_drain(gpioa_crh);
    let dig8 = pa1.into_alternate_open_drain(gpioa_crl);

    let pwm_freq = 2.kHz();
    let pins1 = (dig4, dig5, dig7);
    let pwm1 = tim1.pwm_hz::<Tim1NoRemap, _, _>(pins1, afio_mapr, pwm_freq, &clocks);

    let pins2 = (dig8, dig6, dig2);
    let pwm2 = tim2.pwm_hz::<Tim2NoRemap, _, _>(pins2, afio_mapr, pwm_freq, &clocks);

    let pins3 = (dig1, dig3);
    let pwm3 = tim3.pwm_hz::<Tim3NoRemap, _, _>(pins3, afio_mapr, pwm_freq, &clocks);

    let mut tim4 = tim4.counter_us(&clocks);
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
    ClockDisplayViewer::new(display)
}

fn init_buttons(
    pb15: Pin<'B', 15, Input<Floating>>,
    pb14: Pin<'B', 14, Input<Floating>>,
    pb13: Pin<'B', 13, Input<Floating>>,
    pc13: Pin<'C', 13, Input<Floating>>,
    gpiob_crh: &mut Cr<'B', true>,
    gpioc_crh: &mut Cr<'C', true>,
) -> [Button<ActiveHigh>; 4] {
    let btn1 = Button::<ActiveHigh>::new(Box::new(pb15.into_pull_down_input(gpiob_crh)));
    let btn2 = Button::<ActiveHigh>::new(Box::new(pb14.into_pull_down_input(gpiob_crh)));
    let btn3 = Button::<ActiveHigh>::new(Box::new(pb13.into_pull_down_input(gpiob_crh)));
    let btn4 = Button::<ActiveHigh>::new(Box::new(pc13.into_pull_down_input(gpioc_crh)));

    [btn1, btn2, btn3, btn4]
}

fn init_leds<'a>(
    pb12: Pin<'B', 12, Input<Floating>>,
    pb11: Pin<'B', 11, Input<Floating>>,
    pb1: Pin<'B', 1, Input<Floating>>,
    pb0: Pin<'B', 0, Input<Floating>>,
    gpiob_crl: &mut Cr<'B', false>,
    gpiob_crh: &mut Cr<'B', true>,
) -> [Box<dyn OutputPin<Error = Infallible> + Send>; 4] {
    let led1 = pb12.into_open_drain_output(gpiob_crh);
    let led2 = pb11.into_open_drain_output(gpiob_crh);
    let led3 = pb1.into_open_drain_output(gpiob_crl);
    let led4 = pb0.into_open_drain_output(gpiob_crl);

    [
        Box::new(led1),
        Box::new(led2),
        Box::new(led3),
        Box::new(led4),
    ]
}

fn init_heap() {
    use core::mem::MaybeUninit;
    const HEAP_SIZE: usize = 512;
    static mut HEAP_MEM: [MaybeUninit<u8>; HEAP_SIZE] = [MaybeUninit::uninit(); HEAP_SIZE];
    unsafe { HEAP.init(HEAP_MEM.as_ptr() as usize, HEAP_SIZE) }
}

#[entry]
fn main() -> ! {
    init_heap();

    let cp = cortex_m::Peripherals::take().unwrap();
    let dp = pac::Peripherals::take().unwrap();

    let mut pwr = dp.PWR;
    let mut flash = dp.FLASH.constrain();
    let rcc = dp.RCC.constrain();
    let mut backup_domain = rcc.bkp.constrain(dp.BKP, &mut pwr);

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

    let leds = init_leds(
        gpiob.pb12,
        gpiob.pb11,
        gpiob.pb1,
        gpiob.pb0,
        &mut gpiob.crl,
        &mut gpiob.crh,
    );
    let btns = init_buttons(
        gpiob.pb15,
        gpiob.pb14,
        gpiob.pb13,
        gpioc.pc13,
        &mut gpiob.crh,
        &mut gpioc.crh,
    );

    let mut display = init_segment_display(
        gpiob.pb10,
        gpiob.pb2,
        gpiob.pb8,
        gpiob.pb6,
        gpiob.pb9,
        pb3,
        pb4,
        gpiob.pb7,
        gpioa.pa6,
        gpioa.pa3,
        gpioa.pa7,
        gpioa.pa8,
        gpioa.pa9,
        gpioa.pa2,
        gpioa.pa10,
        gpioa.pa1,
        dp.TIM1,
        dp.TIM2,
        dp.TIM3,
        dp.TIM4,
        &mut gpioa.crl,
        &mut gpioa.crh,
        &mut gpiob.crl,
        &mut gpiob.crh,
        &mut afio.mapr,
        &clocks,
    );
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
            Calendar::from_seconds(Calendar::new(0, 0, 0, 1, 8, 2023), current_time),
            MonoTimer::new(cp.DWT, cp.DCB, clocks),
        );

        let app = ClockApp::new(rtc, display, state);
        APP.borrow(cs).replace(Some(app));
    });

    unsafe {
        cortex_m::peripheral::NVIC::unmask(interrupt::TIM4);
    }

    let delay = cp.SYST.delay(&clocks);
    main_loop(delay, btns, leds)
}

fn main_loop(
    mut delay: SysDelay,
    mut btns: [Button<ActiveHigh>; 4],
    mut leds: [Box<dyn OutputPin<Error = Infallible> + Send>; 4],
) -> ! {
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
