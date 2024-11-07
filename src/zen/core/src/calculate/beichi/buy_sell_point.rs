use crate::analyze::CZSC;
use crate::element::chan::Bar;
use crate::element::chan::{NewBar, DT};
use crate::element::enums::Direction;
use crate::element::event::{Signal, ZS};
use crate::utils::notify::Notify;
use chrono::{FixedOffset, Local, TimeZone, Utc};
use chrono_tz::Tz;
use dict_derive::{FromPyObject, IntoPyObject};
use pyo3::{pyclass, pymethods};
use serde::Serialize;
use std::cmp::PartialEq;
use std::ops::Sub;
use std::rc::Rc;
use tracing::debug;

#[derive(Eq, PartialEq, Serialize, Debug, Clone)]
enum PointType {
    None,
    FirstBuy,
    SecondBuy,
    ThirdBuy,
    FirstSell,
    SecondSell,
    ThirdSell,
}

#[derive(Serialize, Debug, Clone, Eq, PartialEq)]
#[pyclass]
enum BeichiType {
    Area,
    Diff,
    ZsZs,
    ZsLzs,
}

#[derive(Serialize, Debug, Clone)]
pub struct ZSInfo {
    left: i64,
    pub(crate) right: i64,
    high: f32,
    low: f32,
    bi_count: u32,
}

#[derive(Serialize, Debug, Clone)]
#[pyclass]
pub struct BSPoint {
    direction: Direction,
    r#type: PointType,
    bc_type: Vec<BeichiType>,
    pub(crate) zs2: ZSInfo,
    zs1: Option<ZSInfo>,
    fake_bi: bool,
    macd_a_dt: i64,
    macd_a_val: f32,
    pub(crate) macd_b_dt: i64,
    macd_b_val: f32,
    dt: i64,
    price: f32,
}

#[pymethods]
impl BSPoint {
    fn __str__(&self) -> String {
        serde_json::to_string(&self).unwrap()
    }
}
pub struct BuySellPoint {
    pub beichi_tracker: Vec<BSPoint>,
    last_bi_start_dt: DT,
}

impl BuySellPoint {
    pub fn new() -> Self {
        Self {
            beichi_tracker: vec![],
            last_bi_start_dt: Local::now().fixed_offset(),
        }
    }

    pub fn process(
        &mut self,
        czsc: &mut CZSC,
        is_new: bool,
        start: Option<(Bar, Direction)>,
    ) -> Vec<Signal> {
        let mut result = vec![];
        if czsc
            .bi_list
            .last()
            .map(|b| b.fx_b.dt)
            .unwrap_or(Local::now().fixed_offset())
            == self.last_bi_start_dt
        {
            self.beichi_tracker
                .retain(|bc| bc.zs2.right != self.last_bi_start_dt.timestamp());
        }

        let bs = self.calculate(czsc, 0);
        if bs.is_some() {
            let bs = bs.unwrap();
            let signal = Signal {
                key: (
                    format!("{:?}", czsc.freq),
                    format!("D{}-MACD面积背驰", 0 + 1),
                    if bs.fake_bi {
                        "推笔".to_string()
                    } else {
                        "BS".to_string()
                    },
                ),
                value: (
                    if bs.direction == Direction::Up {
                        "顶"
                    } else {
                        "底"
                    }
                        .to_string(),
                    format!("{}笔", bs.zs2.bi_count + 2),
                    "other".to_string(),
                ),
                dt: Some(
                    Utc.timestamp_opt(bs.dt, 0)
                        .unwrap()
                        .fixed_offset()
                        .with_timezone(&FixedOffset::east_opt(8 * 3600).unwrap()),
                ),
                figure: if bs.bc_type.contains(&BeichiType::Diff) {
                    100.0
                } else {
                    80.0
                },
                figure_max: None,
            };
            if !bs.fake_bi {
                //Notify::notify_signal(&czsc.symbol, signal.dt.unwrap(), signal.clone());
            }
            result.push(signal);
            self.beichi_tracker.push(bs);
        }
        czsc.bi_list
            .last()
            .map(|b| self.last_bi_start_dt = b.fx_a.dt);
        self.beichi_tracker
            .dedup_by(|a, b| a.zs2.left == b.zs2.left && a.zs2.right == b.zs2.right);
        if self.beichi_tracker.len() > 100 {
            self.beichi_tracker = self
                .beichi_tracker
                .split_off(self.beichi_tracker.len() - 100);
        }
        //debug!("tracker {:?}", self.beichi_tracker);
        result
    }

    fn zs<'a>(&'a self, czsc: &'a CZSC, dindex: usize, use_fake: bool) -> Option<ZS> {
        const LEFT: i32 = 1;
        if !use_fake && czsc.bars_ubi.len() as i32 - LEFT > 4 {
            return None;
        }
        if use_fake && czsc.bars_ubi.len() as i32 - LEFT < 4 {
            return None;
        }

        let extra_offset = if use_fake { 1 } else { 0 };
        for n in vec![9, 7, 5, 3] {
            let n = n - extra_offset;
            let len = czsc.bi_list.len();
            if len < dindex + n {
                continue;
            }

            let slice = czsc.bi_list.get((len - dindex - n)..(len - dindex));
            let zs = slice.map(|x| ZS::new(x.get(0..(x.len() - 1 + extra_offset)).unwrap()));
            if zs.as_ref().map(|z| z.is_valid().clone()).unwrap_or(false) == false {
                continue;
            }

            let zs = zs.unwrap();

            let bi_first = slice.unwrap().first().unwrap();
            let bi_last = slice.unwrap().last().unwrap(); // !use_fake

            if bi_first.direction == Direction::Up {
                let last_high = if use_fake {
                    if czsc
                        .fake_bi_high()
                        .sub(czsc.fake_max_high().unwrap_or(0.0))
                        .abs()
                        > f32::EPSILON
                    {
                        continue;
                    }
                    czsc.fake_bi_high()
                } else {
                    bi_last.high()
                };

                if bi_first.low() >= zs.dd() || last_high <= zs.gg() {
                    continue;
                }
            } else {
                let last_low = if use_fake {
                    if czsc
                        .fake_bi_low()
                        .sub(czsc.fake_min_low().unwrap_or(0.0))
                        .abs()
                        < f32::EPSILON
                    {
                        continue;
                    }
                    czsc.fake_bi_low()
                } else {
                    bi_last.low()
                };
                if bi_first.high() <= zs.gg() || last_low >= zs.dd() {
                    continue;
                }
            }

            return Some(zs);
        }

        None
    }

    pub fn calculate(&self, czsc: &mut CZSC, dindex: usize) -> Option<BSPoint> {
        let mut result: Vec<BSPoint> = vec![];

        for fake in if dindex == 0 {
            vec![false, true]
        } else {
            vec![false]
        } {
            if !fake && czsc.bi_list.len() > 3 {
                if let Some(val) = czsc
                    .bi_list
                    .last_mut()
                    .unwrap()
                    .cache
                    .get::<Option<BSPoint>>()
                {
                    if let Some(bs) = val {
                        return Some(bs.clone());
                    }
                } else {
                    continue;
                }
            }
            let zs2 = self.zs(czsc, dindex, fake);
            if zs2.is_none() {
                continue;
            }
            let zs2 = zs2.unwrap();

            let zs1 = self.zs(czsc, dindex + zs2.bis.len() + 1, fake);

            let bi_first = czsc
                .bi_list
                .iter()
                .rev()
                .skip(dindex + if fake { 0 } else { 1 } + zs2.bis.len())
                .next()
                .unwrap();
            let bi_last = czsc.bi_list.iter().rev().skip(dindex).next().unwrap();
            let direction = bi_first.direction.clone();

            let mut zs1_exist =
                zs1.is_some() && zs1.as_ref().map(|z| z.bis.len()).unwrap_or(0) >= 3;
            if direction == Direction::Down {
                zs1_exist =
                    zs1_exist && zs1.as_ref().map(|z| z.zd()).unwrap_or(f32::MAX) > zs2.zg();
            } else {
                zs1_exist =
                    zs1_exist && zs1.as_ref().map(|z| z.zg()).unwrap_or(f32::MAX) < zs2.zd();
            }

            let summer = |x: &Rc<NewBar>| -> f32 {
                if direction == Direction::Up {
                    x.positive_dea_sum()
                } else {
                    x.negative_dea_sum()
                }
            };

            let first_macd_area: f32 = bi_first.bars.iter().map(summer).sum();
            let last_macd_area: f32 = if !fake {
                bi_last.bars.iter().map(summer).sum()
            } else {
                czsc.bars_ubi.iter().skip(1).map(summer).sum()
            };

            let bi_first_diff = bi_first.diff();
            let bi_last_diff = if !fake {
                bi_last.diff()
            } else {
                czsc.fake_bi_diff()
            };

            let dif_zero = if direction == Direction::Up {
                zs2.min_diff()
            } else {
                zs2.max_diff()
            };

            if false {
                debug!(
                    "first: {}, end: {}, macd_area: {} {}",
                    bi_first.fx_a.dt,
                    bi_last.fx_a.dt,
                    first_macd_area.abs(),
                    last_macd_area.abs()
                );
            }
            if first_macd_area.abs() <= last_macd_area.abs() {
                continue;
            }

            let diff_threshold = 0.7;

            if direction == Direction::Up {
                let macd_a = bi_first.max_diff_bar();
                let macd_b = if !fake { bi_last.max_diff_bar() } else { None };
                let mut res = BSPoint {
                    direction,
                    r#type: if !zs1_exist {
                        PointType::None
                    } else {
                        PointType::FirstSell
                    },
                    bc_type: vec![
                        BeichiType::Area,
                        if zs2.bis.len() == 1 {
                            BeichiType::ZsLzs
                        } else {
                            BeichiType::ZsZs
                        },
                    ],
                    zs2: ZSInfo {
                        left: zs2.sdt().timestamp(),
                        right: zs2.edt().timestamp(),
                        high: zs2.zg(),
                        low: zs2.zd(),
                        bi_count: zs2.bis.len() as u32,
                    },
                    zs1: zs1.map(|z| ZSInfo {
                        left: z.sdt().timestamp(),
                        right: z.edt().timestamp(),
                        high: z.zg(),
                        low: z.zd(),
                        bi_count: z.bis.len() as u32,
                    }),
                    fake_bi: fake,
                    macd_a_dt: macd_a
                        .as_ref()
                        .map(|x| x.borrow().dt.timestamp())
                        .unwrap_or(0),
                    macd_a_val: macd_a.map(|x| x.borrow().macd_4_9_9.2).unwrap_or(0.0),
                    macd_b_dt: macd_b
                        .as_ref()
                        .map(|x| x.borrow().dt.timestamp())
                        .unwrap_or(0),
                    macd_b_val: macd_b.map(|x| x.borrow().macd_4_9_9.2).unwrap_or(0.0),
                    dt: if fake {
                        czsc.bars_ubi.last().map(|x| x.dt.timestamp()).unwrap_or(0)
                    } else {
                        bi_last.fx_b.dt.timestamp()
                    },
                    price: if fake {
                        czsc.bars_ubi.last().map(|x| x.high).unwrap_or(0.0)
                    } else {
                        bi_last.high()
                    },
                };
                if dif_zero > (bi_first_diff * diff_threshold).abs() {
                    continue;
                }
                if bi_first_diff > bi_last_diff && bi_last_diff > 0.0 {
                    res.bc_type.push(BeichiType::Diff);
                }
                if !fake && res.r#type != PointType::None {
                    czsc.bi_list
                        .last_mut()
                        .unwrap()
                        .cache
                        .insert(Some(res.clone()));
                    return Some(res);
                } else {
                    result.push(res);
                }
            } else {
                let macd_a = bi_first.min_diff_bar();
                let macd_b = if !fake { bi_last.min_diff_bar() } else { None };
                let mut res = BSPoint {
                    direction,
                    r#type: if !zs1_exist {
                        PointType::None
                    } else {
                        PointType::FirstBuy
                    },
                    bc_type: vec![
                        BeichiType::Area,
                        if zs2.bis.len() == 1 {
                            BeichiType::ZsLzs
                        } else {
                            BeichiType::ZsZs
                        },
                    ],
                    zs2: ZSInfo {
                        left: zs2.sdt().timestamp(),
                        right: zs2.edt().timestamp(),
                        high: zs2.zg(),
                        low: zs2.zd(),
                        bi_count: zs2.bis.len() as u32,
                    },
                    zs1: zs1.map(|z| ZSInfo {
                        left: z.sdt().timestamp(),
                        right: z.edt().timestamp(),
                        high: z.zg(),
                        low: z.zd(),
                        bi_count: z.bis.len() as u32,
                    }),
                    fake_bi: fake,
                    macd_a_dt: macd_a
                        .as_ref()
                        .map(|x| x.borrow().dt.timestamp())
                        .unwrap_or(0),
                    macd_a_val: macd_a.map(|x| x.borrow().macd_4_9_9.2).unwrap_or(0.0),
                    macd_b_dt: macd_b
                        .as_ref()
                        .map(|x| x.borrow().dt.timestamp())
                        .unwrap_or(0),
                    macd_b_val: macd_b.map(|x| x.borrow().macd_4_9_9.2).unwrap_or(0.0),
                    dt: if fake {
                        czsc.bars_ubi.last().map(|x| x.dt.timestamp()).unwrap_or(0)
                    } else {
                        bi_last.fx_b.dt.timestamp()
                    },
                    price: if fake {
                        czsc.bars_ubi.last().map(|x| x.low).unwrap_or(0.0)
                    } else {
                        bi_last.low()
                    },
                };
                if dif_zero < -(bi_first_diff * diff_threshold).abs() {
                    continue;
                }
                if bi_first_diff < bi_last_diff && bi_last_diff < 0.0 {
                    res.bc_type.push(BeichiType::Diff);
                }
                if !fake && res.r#type != PointType::None {
                    czsc.bi_list
                        .last_mut()
                        .unwrap()
                        .cache
                        .insert(Some(res.clone()));
                    return Some(res);
                } else {
                    result.push(res);
                }
            }
        }

        if result.len() > 0 {
            //debug!("rest beichi {:?}", result);
        }
        if czsc.bi_list.len() > 0 {
            czsc.bi_list
                .last_mut()
                .unwrap()
                .cache
                .insert::<Option<BSPoint>>(None);
        }
        // 非盘整背驰
        if let Some(bc) = result
            .iter()
            .filter(|bc| bc.fake_bi == false && bc.r#type != PointType::None)
            .next()
        {
            return Some(bc.clone());
        }

        // 盘整背驰
        if let Some(bc) = result
            .iter()
            .filter(|bc| bc.fake_bi == false && bc.r#type == PointType::None)
            .next()
        {
            return Some(bc.clone());
        }
        None
    }
}
