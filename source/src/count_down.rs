use fugit::MicrosDurationU32;
use nb::Result;
use stm32f1xx_hal::timer::{self, CounterUs, Instance};

pub trait CountDown {
    type Time;

    fn start(&mut self, count: Self::Time);
    fn wait(&mut self) -> Result<(), timer::Error>;
}

pub struct CountDowner<T> {
    counter: CounterUs<T>,
}

impl<T> CountDowner<T>
where
    T: Instance,
{
    pub fn new(counter: CounterUs<T>) -> Self {
        Self { counter }
    }
}

impl<T> CountDown for CountDowner<T>
where
    T: Instance,
{
    type Time = MicrosDurationU32;

    fn start(&mut self, count: Self::Time) {
        self.counter.start(count).unwrap();
    }

    fn wait(&mut self) -> nb::Result<(), timer::Error> {
        self.counter.wait()
    }
}
