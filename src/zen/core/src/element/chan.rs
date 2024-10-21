use std::any::Any;
use std::cell::RefCell;
use std::rc::Rc;
use anymap3::{Map};

use chrono::{DateTime, FixedOffset, Local};
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

impl Clone for Bar{
    fn clone(&self) -> Self {
       Self{
           dt: self.dt,
           freq: self.freq,
           open: self.open,
           close: self.close,
           high: self.high,
           low: self.low,
           vol: self.vol,
           amount: self.amount,
           cache: Default::default(),
           macd_4_9_9: (0.0, 0.0, 0.0),
       }
    }
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


// 去除包含关系后的K线元素
#[derive(Debug)]
pub struct NewBar {
    pub dt: DT,
    pub(crate) freq: Freq,
    pub(crate) open: f32,
    pub(crate) close: f32,
    pub high: f32,
    pub low: f32,
    pub(crate) vol: f32,
    pub(crate) amount: f32,
    // cache 用户缓存，一个最常见的场景是缓存技术指标计算结果
    pub(crate) cache: GenericCache,
    pub raw_bars: Vec<Rc<RefCell<Bar>>>, // 存入具有包含关系的原始K线
}

impl Default for NewBar {
    fn default() -> Self {
        Self {
            dt: Local::now().fixed_offset(),
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

    pub fn positive_dea_sum(&self) -> f32 {
        self.raw_bars
            .iter()
            .map(|e| e.borrow().macd_4_9_9.2.max(0.0))
            .sum()
    }

    pub fn negative_dea_sum(&self) -> f32 {
        self.raw_bars
            .iter()
            .map(|e| e.borrow().macd_4_9_9.2.min(0.0))
            .sum()
    }
}

#[derive(Debug)]
pub struct FX {
    pub dt: DT,
    pub(crate) mark: Mark,
    pub high: f32,
    pub(crate) low: f32,
    pub(crate) fx: f32,
    pub elements: Vec<Rc<NewBar>>,
    pub(crate) cache: GenericCache,
}

#[derive(Debug)]
pub struct BI {
    // 笔开始的分型
    pub fx_a: Rc<FX>,
    // 笔结束的分型
    pub fx_b: Rc<FX>,
    // 笔内部的分型列表
    pub fxs: Vec<Rc<FX>>,
    pub direction: Direction,
    pub bars: Vec<Rc<NewBar>>,
    pub cache: GenericCache,
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

    pub fn iter(&self) -> impl Iterator<Item = &Rc<NewBar>> {
        self.bars.get(1..self.bars.len() - 1).unwrap().iter()
    }

    pub fn diff(&self) -> f32 {
        self.bars
            .iter()
            .rev()
            .skip(1)
            .next()
            .unwrap()
            .raw_bars
            .last()
            .map(|x| x.borrow().macd_4_9_9.0)
            .unwrap_or(0.0f32)
    }

    pub fn max_diff_bar(&self) -> Option<Rc<RefCell<Bar>>> {
        let mut bar = None;
        let mut max = f32::MIN;
        for n in self.iter() {
            for b in &n.raw_bars {
                if b.borrow().macd_4_9_9.2 > max {
                    bar = Some(b.clone());
                    max = b.borrow().macd_4_9_9.2;
                }
            }
        }
        bar
    }

    pub fn min_diff_bar(&self) -> Option<Rc<RefCell<Bar>>> {
        let mut bar = None;
        let mut min = f32::MAX;
        for n in self.iter() {
            for b in &n.raw_bars {
                if b.borrow().macd_4_9_9.2 < min {
                    bar = Some(b.clone());
                    min = b.borrow().macd_4_9_9.2;
                }
            }
        }
        bar
    }
}
