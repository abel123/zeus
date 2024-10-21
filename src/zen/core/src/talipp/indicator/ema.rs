use super::Indicator;

#[derive(Debug)]
pub struct EMA {
    head: Vec<f32>,
    prev_value: f32,
    value: f32,
    period: usize,
    multiplier: f32,
}

impl EMA {
    pub fn new(period: usize) -> Self {
        Self {
            head: vec![],
            prev_value: 0.0,
            value: 0.0,
            period,
            multiplier: 2.0 / (period as f32 + 1.0),
        }
    }

    pub fn value(&self) -> f32 {
        self.value
    }
}

impl Indicator for EMA {
    fn next(&mut self, val: f32) {
        if self.head.len() < self.period {
            self.head.push(val);
            self.prev_value = self.head.iter().sum::<f32>() / self.head.len() as f32;
            self.value = self.prev_value;
        } else {
            let value = (val - self.value) * self.multiplier + self.value;
            self.prev_value = self.value;
            self.value = value;
        }
    }

    fn update(&mut self, val: f32) {
        self.value = (val - self.prev_value) * self.multiplier + self.prev_value;
    }
}
