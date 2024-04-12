use crate::calculate::r#trait::Processor;
use crate::utils::notify::Notify;
use cached::proc_macro::cached;
use lru::LruCache;
use notify_rust::Notification;
use serde::Serialize;
use std::cell::RefCell;
use std::num::NonZeroUsize;
use std::rc::Rc;
use time::{format_description, OffsetDateTime};
use tracing::debug;
use zen_core::objects::chan::{NewBar, BI};
use zen_core::objects::enums::Direction;
use zen_core::objects::trade::{Signal, ZS};
use zen_core::{Bar, CZSC};

#[derive(Debug, Clone, Serialize, Eq, PartialEq)]
pub(crate) struct Range {
    pub left_dt: i64,
    pub right_dt: i64,
}

#[derive(Serialize, Debug, Clone)]
pub(crate) struct BeichiInfo {
    direction: Direction,
    pub start: Range,
    pub end: Range,
    high: f32,
    low: f32,
    r#type: String,
    macd_a_dt: i64,
    macd_a_val: f32,
    macd_b_dt: i64,
    macd_b_val: f32,
}
pub(crate) struct MacdArea {
    dindex: usize,
    threshold: usize,
    pub beichi_tracker: Vec<BeichiInfo>,
    last_bi_start_dt: OffsetDateTime,
}

impl Processor for MacdArea {
    fn process(&mut self, czsc: &CZSC, is_new: bool) -> Vec<Signal> {
        let mut result = vec![];
        if czsc
            .bi_list
            .last()
            .map(|b| b.fx_b.dt)
            .unwrap_or(OffsetDateTime::now_utc())
            == self.last_bi_start_dt
        {
            self.beichi_tracker
                .retain(|bc| bc.end.left_dt != self.last_bi_start_dt.unix_timestamp());
        }
        for use_fake in vec![false, true] {
            result.append(&mut self.bc_single(&czsc, 0, use_fake));
        }

        czsc.bi_list
            .last()
            .map(|b| self.last_bi_start_dt = b.fx_a.dt);
        self.beichi_tracker
            .dedup_by(|a, b| a.start == b.start && a.end == b.end);

        if self.beichi_tracker.len() > 100 {
            self.beichi_tracker = self
                .beichi_tracker
                .split_off(self.beichi_tracker.len() - 100);
        }
        return result;
    }
}

impl MacdArea {
    pub fn new(dindex: usize) -> Self {
        Self {
            dindex,
            threshold: 90,
            beichi_tracker: vec![],
            last_bi_start_dt: OffsetDateTime::now_utc(),
        }
    }
    fn bc_single(&mut self, czsc: &CZSC, dindex: usize, use_fake: bool) -> Vec<Signal> {
        if czsc.bi_list.len() < 7 {
            return vec![];
        }
        const LEFT_RIGHT: i32 = 2;
        const LEFT: i32 = 1;
        if !use_fake && czsc.bars_ubi.len() as i32 - LEFT_RIGHT > 4 {
            return vec![];
        }
        if use_fake && czsc.bars_ubi.len() as i32 - LEFT < 4 {
            return vec![];
        }
        let mut result = vec![];

        let extra_offset = if use_fake { 1 } else { 0 };
        for n in vec![3, 5, 7, 9] {
            let n = n - extra_offset;
            let len = czsc.bi_list.len();
            if len < dindex + n || n == 2 {
                continue;
            }

            let slice = czsc.bi_list.get((len - dindex - n)..(len - dindex));
            let zs = slice.map(|x| ZS::new(x.get(1..(x.len() - 1 + extra_offset)).unwrap()));
            if zs.is_none() {
                continue;
            }
            let zs = zs.unwrap();
            if !zs.is_valid() {
                continue;
            }
            let bi_first = slice.unwrap().first().unwrap();
            let bi_last = slice.unwrap().last().unwrap(); // !use_fake
            let mut min_low = slice
                .unwrap()
                .iter()
                .map(|b| b.low())
                .min_by(|a, b| a.partial_cmp(b).unwrap())
                .unwrap();
            let mut max_high = slice
                .unwrap()
                .iter()
                .map(|b| b.high())
                .max_by(|a, b| a.partial_cmp(b).unwrap())
                .unwrap();
            let mut bi_last_low = bi_last.low();
            let mut bi_last_high = bi_last.high();

            let bi_first_diff = bi_first
                .bars
                .iter()
                .rev()
                .skip(1)
                .next()
                .unwrap()
                .raw_bars
                .last()
                .map(|x| x.borrow().macd_4_9_9.0)
                .unwrap_or(0.0f32);

            let mut bi_last_diff = bi_last
                .bars
                .iter()
                .rev()
                .skip(1)
                .next()
                .unwrap()
                .raw_bars
                .last()
                .map(|x| x.borrow().macd_4_9_9.0)
                .unwrap_or(0.0f32);
            if use_fake {
                let high = czsc.bars_ubi[1]
                    .high
                    .max(czsc.bars_ubi.last().map(|x| x.high).unwrap_or(0.0f32));
                max_high = max_high.max(high);
                let low = czsc.bars_ubi[1]
                    .low
                    .min(czsc.bars_ubi.last().map(|x| x.low).unwrap_or(1e10f32));
                min_low = min_low.min(low);
                bi_last_low = low;
                bi_last_high = high;
                let all_high = czsc
                    .bars_ubi
                    .iter()
                    .skip(1)
                    .map(|x| x.high)
                    .max_by(|a, b| a.partial_cmp(b).unwrap());
                let all_low = czsc
                    .bars_ubi
                    .iter()
                    .skip(1)
                    .map(|x| x.low)
                    .min_by(|a, b| a.partial_cmp(b).unwrap());
                if high != all_high.unwrap_or(-1.0) || low != all_low.unwrap_or(-1.0) {
                    continue;
                }
                bi_last_diff = czsc
                    .bars_ubi
                    .last()
                    .unwrap()
                    .raw_bars
                    .last()
                    .map(|x| x.borrow().macd_4_9_9.0)
                    .unwrap_or(0.0f32);
            }

            let summer = |x: &Rc<NewBar>| -> f32 {
                if bi_first.direction == Direction::Up {
                    x.raw_bars
                        .iter()
                        .map(|e| e.borrow().macd_4_9_9.2.max(0.0))
                        .sum()
                } else {
                    x.raw_bars
                        .iter()
                        .map(|e| e.borrow().macd_4_9_9.2.min(0.0))
                        .sum()
                }
            };
            let first_macd_area: f32 = bi_first.iter().map(summer).sum();
            let last_macd_area: f32 = if !use_fake {
                bi_last.iter().map(summer).sum()
            } else {
                czsc.bars_ubi.iter().skip(1).map(summer).sum()
            };

            let dif_zero = if bi_first.direction == Direction::Up {
                let mut diff = f32::MAX;
                for b in zs.bis {
                    for e in &b.fx_b.elements {
                        for bar in &e.raw_bars {
                            diff = diff.min(bar.borrow().macd_4_9_9.0);
                        }
                    }
                }
                diff
            } else {
                let mut diff = f32::MIN;
                for b in zs.bis {
                    for e in &b.fx_b.elements {
                        for bar in &e.raw_bars {
                            diff = diff.max(bar.borrow().macd_4_9_9.0);
                        }
                    }
                }
                diff
            };

            if false {
                debug!(
                    "{:?} {}-{} => {}-{} : low {} {}, high {} {}, diff {} {} {}, area {} {}",
                    czsc.freq,
                    bi_first.fx_a.dt,
                    bi_first.fx_b.dt,
                    bi_last.fx_a.dt,
                    bi_last.fx_b.dt,
                    bi_first.low(),
                    min_low,
                    bi_last_high,
                    max_high,
                    dif_zero,
                    bi_first_diff,
                    bi_last_diff,
                    first_macd_area,
                    last_macd_area
                );
            }
            if last_macd_area.abs() > first_macd_area.abs() * self.threshold as f32 / 100.0 {
                continue;
            }

            let diff_threshold = 80.0 / 100.0;

            if bi_first.direction == Direction::Up
                && (bi_first.low() - min_low).abs() < f32::EPSILON
                && (bi_last_high - max_high).abs() < f32::EPSILON
                && (dif_zero < 0.00001 || dif_zero.abs() < diff_threshold * bi_first_diff)
                && (bi_first_diff > 0.0 && bi_last_diff > 0.0)
            {
                let score = if bi_first_diff > bi_last_diff && bi_last_diff > 0.0 {
                    100
                } else {
                    80
                };
                if !use_fake {
                    let bi_max_fn = |bi: &BI| {
                        let mut bar = None;
                        let mut max = f32::MIN;
                        for n in bi.iter() {
                            for b in &n.raw_bars {
                                if b.borrow().macd_4_9_9.2 > max {
                                    bar = Some(b.clone());
                                    max = b.borrow().macd_4_9_9.2;
                                }
                            }
                        }
                        bar
                    };
                    let macd_a = bi_max_fn(bi_first);
                    let macd_b = bi_max_fn(bi_last);

                    if n > 3 {
                        self.beichi_tracker.push(BeichiInfo {
                            direction: Direction::Up,
                            start: Range {
                                left_dt: bi_first.fx_a.dt.unix_timestamp(),
                                right_dt: bi_first.fx_b.dt.unix_timestamp(),
                            },
                            end: Range {
                                left_dt: bi_last.fx_a.dt.unix_timestamp(),
                                right_dt: bi_last.fx_b.dt.unix_timestamp(),
                            },
                            high: bi_first.high().max(bi_last_high),
                            low: bi_first.low().min(bi_last_low),
                            r#type: if score == 100 {
                                "area_with_diff".to_string()
                            } else {
                                "area".to_string()
                            },
                            macd_a_dt: macd_a
                                .as_ref()
                                .map(|x| x.borrow().dt.unix_timestamp())
                                .unwrap_or(0),
                            macd_a_val: macd_a.map(|x| x.borrow().macd_4_9_9.2).unwrap_or(0.0),
                            macd_b_dt: macd_b
                                .as_ref()
                                .map(|x| x.borrow().dt.unix_timestamp())
                                .unwrap_or(0),
                            macd_b_val: macd_b.map(|x| x.borrow().macd_4_9_9.2).unwrap_or(0.0),
                        });
                    }
                }
                let signal = Signal {
                    key: (
                        format!("{:?}", czsc.freq),
                        format!("D{}-MACD面积背驰", dindex + 1),
                        if use_fake {
                            "推笔".to_string()
                        } else {
                            "BS".to_string()
                        },
                    ),
                    value: ("顶".to_string(), format!("{}笔", n), "other".to_string()),
                    score,
                };
                if n > 3 {
                    result.push(signal.clone());
                }
                if !use_fake {
                    Notify::notify_signal(bi_last.fx_b.dt, signal);
                }
            }

            if bi_first.direction == Direction::Down
                && (bi_first.high() - max_high).abs() < f32::EPSILON
                && (bi_last_low - min_low).abs() < f32::EPSILON
                && (dif_zero > -0.00001 || dif_zero.abs() < diff_threshold * bi_first_diff)
                && (bi_first_diff < 0.0 && bi_last_diff < 0.0)
            {
                let score = if bi_first_diff < bi_last_diff && bi_last_diff < 0.0 {
                    100
                } else {
                    80
                };
                if !use_fake {
                    let bi_min_fn = |b: &BI| {
                        let mut bar = None;
                        let mut min = f32::MAX;
                        for n in b.iter() {
                            for b in &n.raw_bars {
                                if b.borrow().macd_4_9_9.2 < min {
                                    bar = Some(b.clone());
                                    min = b.borrow().macd_4_9_9.2;
                                }
                            }
                        }
                        bar
                    };
                    let macd_a = bi_min_fn(bi_first);
                    let macd_b = bi_min_fn(bi_last);

                    if n > 3 {
                        self.beichi_tracker.push(BeichiInfo {
                            direction: Direction::Down,
                            start: Range {
                                left_dt: bi_first.fx_a.dt.unix_timestamp(),
                                right_dt: bi_first.fx_b.dt.unix_timestamp(),
                            },
                            end: Range {
                                left_dt: bi_last.fx_a.dt.unix_timestamp(),
                                right_dt: bi_last.fx_b.dt.unix_timestamp(),
                            },
                            high: bi_first.high().max(bi_last_high),
                            low: bi_first.low().min(bi_last_low),
                            r#type: if score == 100 {
                                "area_with_diff".to_string()
                            } else {
                                "area".to_string()
                            },
                            macd_a_dt: macd_a
                                .as_ref()
                                .map(|x| x.borrow().dt.unix_timestamp())
                                .unwrap_or(0),
                            macd_a_val: macd_a.map(|x| x.borrow().macd_4_9_9.2).unwrap_or(0.0),
                            macd_b_dt: macd_b
                                .as_ref()
                                .map(|x| x.borrow().dt.unix_timestamp())
                                .unwrap_or(0),
                            macd_b_val: macd_b.map(|x| x.borrow().macd_4_9_9.2).unwrap_or(0.0),
                        });
                    }
                }
                let signal = Signal {
                    key: (
                        format!("{:?}", czsc.freq),
                        format!("D{}-MACD面积背驰", dindex + 1),
                        if use_fake {
                            "推笔".to_string()
                        } else {
                            "BS".to_string()
                        },
                    ),
                    value: ("底".to_string(), format!("{}笔", n), "other".to_string()),
                    score,
                };
                if n > 3 {
                    result.push(signal.clone());
                }
                if !use_fake {
                    Notify::notify_signal(bi_last.fx_b.dt, signal);
                }
            }
        }
        return result;
    }
}
