use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use chrono::{DateTime, Utc};

use super::enums::{Direction, Freq, Mark};

pub type GenericCache = HashMap<String, Box<dyn std::any::Any>>;

pub type Symbol = String;

//原始K线元素
pub(crate) struct Bar {
    pub(crate) symbol: Symbol,
    pub(crate) id: usize,
    pub(crate) dt: DateTime<Utc>,
    pub(crate) freq: Freq,
    pub(crate) open: f32,
    pub(crate) close: f32,
    pub(crate) high: f32,
    pub(crate) low: f32,
    pub(crate) vol: f32,
    pub(crate) amount: f32,
    cache: GenericCache, // cache 用户缓存，一个最常见的场景是缓存技术指标计算结果
}

// 去除包含关系后的K线元素
pub(crate) struct NewBar {
    pub(crate) symbol: Symbol,
    pub(crate) id: usize,
    pub(crate) dt: DateTime<Utc>,
    pub(crate) freq: Freq,
    pub(crate) open: f32,
    pub(crate) close: f32,
    pub high: f32,
    pub(crate) low: f32,
    pub(crate) vol: f32,
    pub(crate) amount: f32,
    // cache 用户缓存，一个最常见的场景是缓存技术指标计算结果
    pub(crate) cache: GenericCache,
    pub(crate) raw_bars: Vec<Rc<RefCell<Bar>>>, // 存入具有包含关系的原始K线
}

impl Default for NewBar {
    fn default() -> Self {
        Self {
            symbol: "".to_string(),
            id: 0,
            dt: Default::default(),
            freq: Freq::Tick,
            open: 0.0,
            close: 0.0,
            high: 0.0,
            low: 0.0,
            vol: 0.0,
            amount: 0.0,
            cache: Default::default(),
            raw_bars: vec![],
        }
    }
}

impl NewBar {
    pub fn new() -> Self {
        NewBar::default()
    }
}

pub struct FX {
    pub(crate) symbol: Symbol,
    pub(crate) dt: DateTime<Utc>,
    pub(crate) mark: Mark,
    pub(crate) high: f32,
    pub(crate) low: f32,
    pub(crate) fx: f32,
    pub(crate) elements: Vec<Rc<NewBar>>,
    pub(crate) cache: GenericCache,
}

pub struct BI {
    pub(crate) symbol: Symbol,
    // 笔开始的分型
    pub(crate) fx_a: Rc<FX>,
    // 笔结束的分型
    pub(crate) fx_b: Rc<FX>,
    // 笔内部的分型列表
    pub(crate) fxs: Vec<Rc<FX>>,
    pub(crate) direction: Direction,
    pub(crate) bars: Vec<Rc<NewBar>>,
    pub(crate) cache: GenericCache,
}

impl BI {
    pub fn power_price(&self) -> f32 {
        return (self.fx_a.fx - self.fx_b.fx).abs();
    }
    pub fn high(&self) -> f32 {
        f32::max(self.fx_a.high, self.fx_b.high)
    }
    pub fn low(&self) -> f32 {
        f32::min(self.fx_a.low, self.fx_b.low)
    }
}
