use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use time::OffsetDateTime;

use super::enums::{Direction, Freq, Mark};

pub type GenericCache = HashMap<String, String>; //Box<dyn std::any::Any>>;

pub type Symbol = String;

//原始K线元素
#[derive(Debug)]
pub struct Bar {
    pub id: usize,
    pub dt: OffsetDateTime,
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

// 去除包含关系后的K线元素
#[derive(Debug, Clone)]
pub struct NewBar {
    pub(crate) id: usize,
    pub dt: OffsetDateTime,
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
            id: 0,
            dt: OffsetDateTime::now_utc(),
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
    pub dt: OffsetDateTime,
    pub(crate) mark: Mark,
    pub(crate) high: f32,
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
