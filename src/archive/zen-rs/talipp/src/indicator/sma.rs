use crate::indicator::Indicator;
use std::collections::VecDeque;

#[derive(Debug)]
pub struct SMA {
    queue: VecDeque<f32>,
    period: isize,
    sum: f32,
}
impl SMA {
    pub fn new(period: isize) -> Self {
        SMA {
            queue: Default::default(),
            period,
            sum: 0.0,
        }
    }

    pub fn ma(&self) -> f32 {
        self.sum / self.period as f32
    }

    pub fn last(&self) -> f32 {
        *self.queue.back().unwrap_or(&0.0)
    }
}

impl Indicator for SMA {
    fn next(&mut self, val: f32) {
        self.queue.push_back(val);
        self.sum += val;

        if self.queue.len() > self.period as usize {
            let front = self.queue.pop_front();
            self.sum -= front.unwrap_or(0.0);
        }
    }

    fn update(&mut self, val: f32) {
        self.sum += val - self.queue.back().map(|x| *x).unwrap_or(0.0);
        self.queue.pop_back();
        self.queue.push_back(val);
    }
}
