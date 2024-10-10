use std::any::Any;
use anymap3::{Map};

use chrono::{DateTime, FixedOffset};
use pyo3::prelude::*;
use super::enums::{Direction, Freq, Mark};

pub type GenericCache = Map<dyn Any+Send>;
pub type DT = DateTime<FixedOffset>;

//原始K线元素
#[derive(Debug)]
#[pyclass]
pub struct Bar {
    pub dt: DT,
    pub freq: Freq,
    pub open: f32,
    pub close: f32,
    pub high: f32,
    pub low: f32,
    pub vol: f32,
    pub amount: f32,
    pub cache: GenericCache, // cache 用户缓存，一个最常见的场景是缓存技术指标计算结果
    pub macd_4_9_9: (f32, f32, f32),
}

#[pymethods]
impl Bar {
    #[new]
    fn new(dt: DT, o: f32, c:f32, h: f32, l:f32, vol: f32)->Self{
        Bar{
            dt,
            freq: Freq::Tick,
            open: o,
            close: c,
            high: h,
            low: l,
            vol,
            amount: 0.0,
            cache: Default::default(),
            macd_4_9_9: (0.0, 0.0, 0.0),
        }
    }

    fn __str__(&self) -> String {
        format!("{:?}", self)
    }

    fn __repr__(&self) -> String {
        format!("{:?}", self)
    }
}