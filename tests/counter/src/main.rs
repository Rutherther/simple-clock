#![no_std]
#![no_main]

use core::convert::Infallible;
use embedded_hal::digital::v2::{OutputPin, InputPin};
use panic_halt as _;

use nb::block;

use cortex_m_rt::entry;
use stm32f1xx_hal::{pac, prelude::*, timer::{Timer, Tim1NoRemap, Tim2NoRemap, Tim3NoRemap}, rtc::Rtc};

struct Segments<'a> {
    a: &'a mut dyn OutputPin<Error = Infallible>,
    b: &'a mut dyn OutputPin<Error = Infallible>,
    c: &'a mut dyn OutputPin<Error = Infallible>,
    d: &'a mut dyn OutputPin<Error = Infallible>,
    e: &'a mut dyn OutputPin<Error = Infallible>,
    f: &'a mut dyn OutputPin<Error = Infallible>,
    g: &'a mut dyn OutputPin<Error = Infallible>
}

fn set_segments(segments: &mut Segments, enabled_segments: (bool, bool, bool, bool, bool, bool, bool)) {
    segments.a.set_state((!enabled_segments.0).into()).unwrap();
    segments.b.set_state((!enabled_segments.1).into()).unwrap();
    segments.c.set_state((!enabled_segments.2).into()).unwrap();
    segments.d.set_state((!enabled_segments.3).into()).unwrap();
    segments.e.set_state((!enabled_segments.4).into()).unwrap();
    segments.f.set_state((!enabled_segments.5).into()).unwrap();
    segments.g.set_state((!enabled_segments.6).into()).unwrap();
}

fn show_digit(segments: &mut Segments, digit: u8) {
    match digit {
        0 => set_segments(segments, (true, true, true, true, true, true, false)),
        1 => set_segments(segments, (false, true, true, false, false, false, false)),
        2 => set_segments(segments, (true, true, false, true, true, false, true)),
        3 => set_segments(segments, (true, true, true, true, false, false, true)),
        4 => set_segments(segments, (false, true, true, false, false, true, true)),
        5 => set_segments(segments, (true, false, true, true, false, true, true)),
        6 => set_segments(segments, (true, false, true, true, true, true, true)),
        7 => set_segments(segments, (true, true, true, false, false, false, false)),
        8 => set_segments(segments, (true, true, true, true, true, true, true)),
        9 => set_segments(segments, (true, true, true, true, false, true, true)),
        _ => set_segments(segments, (true, true, true, false, false, false, true))
    }
}

fn get_digit(number: u32, digit_index: u8) -> u8 {
    let mut number = number;
    for _ in 0..digit_index {
        number /= 10;
    }
    (number % 10) as u8
}

#[entry]
fn main() -> ! {
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

    let mut rtc = Rtc::new(dp.RTC, &mut backup_domain);

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

    let mut leds: [&mut dyn OutputPin<Error = Infallible>; 4] = [&mut led1, &mut led2, &mut led3, &mut led4];

    let btn1 = gpiob.pb15.into_pull_down_input(&mut gpiob.crh);
    let btn2 = gpiob.pb14.into_pull_down_input(&mut gpiob.crh);
    let btn3 = gpiob.pb13.into_pull_down_input(&mut gpiob.crh);
    let btn4 = gpioc.pc13.into_floating_input(&mut gpioc.crh);

    let btns: [&dyn InputPin<Error = Infallible>; 4] = [&btn1, &btn2, &btn3, &btn4];

    let mut a = gpiob.pb10.into_open_drain_output(&mut gpiob.crh);
    let mut b = gpiob.pb2.into_open_drain_output(&mut gpiob.crl);
    let mut c = gpiob.pb8.into_open_drain_output(&mut gpiob.crh);
    let mut d = gpiob.pb6.into_open_drain_output(&mut gpiob.crl);
    let mut e = gpiob.pb9.into_open_drain_output(&mut gpiob.crh);
    let mut f = pb3.into_open_drain_output(&mut gpiob.crl);
    let mut g = pb4.into_open_drain_output(&mut gpiob.crl);
    let mut dpp = gpiob.pb7.into_open_drain_output(&mut gpiob.crl);
    dpp.set_high();

    let mut segments = Segments { a: &mut a, b: &mut b, c: &mut c, d: &mut d, e: &mut e, f: &mut f, g: &mut g };

    let dig1 = gpioa.pa6.into_alternate_open_drain(&mut gpioa.crl);
    let dig2 = gpioa.pa3.into_alternate_open_drain(&mut gpioa.crl);
    let dig3 = gpioa.pa7.into_alternate_open_drain(&mut gpioa.crl);
    let dig4 = gpioa.pa8.into_alternate_open_drain(&mut gpioa.crh);
    let dig5 = gpioa.pa9.into_alternate_open_drain(&mut gpioa.crh);
    let dig6 = gpioa.pa2.into_alternate_open_drain(&mut gpioa.crl);
    let dig7 = gpioa.pa10.into_alternate_open_drain(&mut gpioa.crh);
    let dig8 = gpioa.pa1.into_alternate_open_drain(&mut gpioa.crl);

    let tim1 = Timer::new(dp.TIM1, &clocks);
    let tim2 = Timer::new(dp.TIM2, &clocks);
    let tim3 = Timer::new(dp.TIM3, &clocks);

    let pins1 = (dig4, dig5, dig7);
    let pwm1 = tim1
        .pwm_hz::<Tim1NoRemap, _, _>(pins1, &mut afio.mapr, 2.kHz());

    let pins2 = (dig8, dig6, dig2);
    let pwm2 = tim2
        .pwm_hz::<Tim2NoRemap, _, _>(pins2, &mut afio.mapr, 2.kHz());

    let pins3 = (dig1, dig3);
    let pwm3 = tim3
        .pwm_hz::<Tim3NoRemap, _, _>(pins3, &mut afio.mapr, 2.kHz());

    let (mut dig4, mut dig5, mut dig7) = pwm1.split();
    let (mut dig8, mut dig6, mut dig2) = pwm2.split();
    let (mut dig1, mut dig3) = pwm3.split();

    for led in leds.iter_mut() {
        led.set_high().unwrap();
    }

    let mut digits: [&mut dyn _embedded_hal_PwmPin<Duty = u16>; 8] = [ &mut dig1, &mut dig2, &mut dig3, &mut dig4, &mut dig5, &mut dig6, &mut dig7, &mut dig8 ];
    for digit in digits.iter_mut() {
        // has to be enabled, when disabled, 0 is outputted, meaning digit is turned on...
        digit.enable();
        digit.set_duty(0xFFFF);
    }

    // Configure the timer to trigger an update with freq of 500 Hz
    let mut delay = cp.SYST.delay(&clocks);
    let mut timer = Timer::new(dp.TIM4, &clocks).counter_hz();
    timer.start(500.Hz()).unwrap();

    let mut digit_index = 0usize;
    let mut counter = 1;
    loop {
        for digit in digits.iter_mut() {
            digit.set_duty(0xFFFF);
        }

        delay.delay_us(480_u16);

        show_digit(&mut segments, get_digit(counter, (digits.len() - digit_index - 1) as u8));
        // digits[digit_index].set_duty(11500);
        digits[digit_index].set_duty(5000);

        digit_index = (digit_index + 1) % digits.len();
        if digit_index == 0 {
            counter = rtc.current_time();
        }

        block!(timer.wait()).unwrap();
    }
}
