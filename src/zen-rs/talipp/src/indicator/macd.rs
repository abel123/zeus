use crate::indicator::ema::EMA;
use crate::indicator::Indicator;

#[derive(Debug)]
pub struct MACD {
    slow: EMA,
    fast: EMA,
    signal: EMA,
}

impl MACD {
    pub fn new(fast: usize, slow: usize, signal: usize) -> Self {
        Self {
            slow: EMA::new(slow),
            fast: EMA::new(fast),
            signal: EMA::new(signal),
        }
    }

    pub fn value(&self) -> (f32, f32, f32) {
        (
            self.fast.value() - self.slow.value(),
            self.signal.value(),
            self.fast.value() - self.slow.value() - self.signal.value(),
        )
    }
}

impl Indicator for MACD {
    fn next(&mut self, val: f32) {
        self.fast.next(val);
        //log::debug!("ema 4 {}, value {}", self.fast.value(), val);
        self.slow.next(val);
        self.signal.next(self.fast.value() - self.slow.value());
    }

    fn update(&mut self, val: f32) {
        self.fast.update(val);
        self.slow.update(val);
        self.signal.update(self.fast.value() - self.slow.value());
    }
}
