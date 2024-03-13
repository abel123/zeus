pub mod ema;
pub mod macd;

pub trait Indicator {
    fn next(&mut self, val: f32);
    fn update(&mut self, val: f32);
}
