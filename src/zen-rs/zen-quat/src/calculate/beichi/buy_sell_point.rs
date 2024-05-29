use crate::calculate::others::macd_area::BeichiInfo;
use serde::Serialize;
use std::cmp::PartialEq;
use std::ops::Sub;
use std::rc::Rc;
use time::OffsetDateTime;
use tracing::debug;
use tws_rs::Error;
use zen_core::objects::chan::NewBar;
use zen_core::objects::enums::Direction;
use zen_core::objects::trade::ZS;
use zen_core::CZSC;

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

#[derive(Serialize, Debug, Clone)]
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

pub struct BuySellPoint {
    pub beichi_tracker: Vec<BSPoint>,
    last_bi_start_dt: OffsetDateTime,
}

impl BuySellPoint {
    pub fn new() -> Self {
        Self {
            beichi_tracker: vec![],
            last_bi_start_dt: OffsetDateTime::now_utc(),
        }
    }

    pub fn process(&mut self, czsc: &CZSC, is_new: bool) {
        if czsc
            .bi_list
            .last()
            .map(|b| b.fx_b.dt)
            .unwrap_or(OffsetDateTime::now_utc())
            == self.last_bi_start_dt
        {
            self.beichi_tracker
                .retain(|bc| bc.zs2.right != self.last_bi_start_dt.unix_timestamp());
        }

        let bs = self.calculate(czsc, 0);
        if bs.is_some() {
            self.beichi_tracker.push(bs.unwrap());
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
            let zs = slice.map(|x| ZS::new(x.get(1..(x.len() - 1 + extra_offset)).unwrap()));
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

    pub fn calculate(&self, czsc: &CZSC, dindex: usize) -> Option<BSPoint> {
        let mut result: Vec<BSPoint> = vec![];

        for fake in [false, true] {
            let zs2 = self.zs(czsc, dindex, fake);
            if zs2.is_none() {
                continue;
            }
            let zs2 = zs2.unwrap();

            let zs1 = self.zs(czsc, dindex + zs2.bis.len() + 1, fake);
            let mut zs1_exist = zs1.is_some()
                && zs1.as_ref().map(|z| z.zd()).unwrap_or(f32::MAX) > zs2.zg()
                && zs1.as_ref().map(|z| z.bis.len()).unwrap_or(0) >= 3;

            let bi_first = czsc
                .bi_list
                .iter()
                .rev()
                .skip(dindex + if fake { 0 } else { 1 } + zs2.bis.len())
                .next()
                .unwrap();
            let bi_last = czsc.bi_list.iter().rev().skip(dindex).next().unwrap();

            let direction = bi_first.direction.clone();
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
                let macd_b = bi_last.max_diff_bar();
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
                        left: zs2.sdt().unix_timestamp(),
                        right: zs2.edt().unix_timestamp(),
                        high: zs2.zg(),
                        low: zs2.zd(),
                        bi_count: zs2.bis.len() as u32,
                    },
                    zs1: zs1.map(|z| ZSInfo {
                        left: z.sdt().unix_timestamp(),
                        right: z.edt().unix_timestamp(),
                        high: z.zg(),
                        low: z.zd(),
                        bi_count: z.bis.len() as u32,
                    }),
                    fake_bi: fake,
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
                    dt: if fake {
                        czsc.bars_ubi
                            .last()
                            .map(|x| x.dt.unix_timestamp())
                            .unwrap_or(0)
                    } else {
                        bi_last.fx_b.dt.unix_timestamp()
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
                    return Some(res);
                } else {
                    result.push(res);
                }
            } else {
                let macd_a = bi_first.min_diff_bar();
                let macd_b = bi_last.min_diff_bar();
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
                        left: zs2.sdt().unix_timestamp(),
                        right: zs2.edt().unix_timestamp(),
                        high: zs2.zg(),
                        low: zs2.zd(),
                        bi_count: zs2.bis.len() as u32,
                    },
                    zs1: zs1.map(|z| ZSInfo {
                        left: z.sdt().unix_timestamp(),
                        right: z.edt().unix_timestamp(),
                        high: z.zg(),
                        low: z.zd(),
                        bi_count: z.bis.len() as u32,
                    }),
                    fake_bi: fake,
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
                    dt: if fake {
                        czsc.bars_ubi
                            .last()
                            .map(|x| x.dt.unix_timestamp())
                            .unwrap_or(0)
                    } else {
                        bi_last.fx_b.dt.unix_timestamp()
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
                    return Some(res);
                } else {
                    result.push(res);
                }
            }
        }

        if result.len() > 0 {
            //debug!("rest beichi {:?}", result);
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
