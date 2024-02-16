use std::cell::RefCell;
use std::cmp::Ordering;
use std::fmt::Display;
use std::ops::Deref;
use std::rc::Rc;

use tracing::error;

use crate::objects::chan::{Bar, BI, FX, NewBar, Symbol};
use crate::objects::enums::{Direction, Freq, Mark};
use crate::settings::{BiType, Settings};

struct CZSC {
    bars_raw: Vec<Rc<RefCell<Bar>>>,
    // 原始K线序列
    bars_ubi: Vec<Rc<NewBar>>,
    //未完成笔的无包含K线序列
    bi_list: Vec<BI>,
    symbol: Symbol,
    freq: Freq,
    settings: Settings,
}

impl Display for CZSC {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "<{}-{:?}>", self.symbol, self.freq)
    }
}

impl CZSC {
    pub fn update(&mut self, bar_: Bar) {
        let last_bar =
            if self.bars_raw.len() == 0 || bar_.dt != self.bars_raw.last().unwrap().borrow().dt {
                self.bars_raw.push(Rc::new(RefCell::new(bar_)));
                vec![self.bars_raw.last().unwrap().clone()]
            } else {
                let len = self.bars_raw.len();
                *self.bars_raw[len - 1].borrow_mut() = bar_;
                let last_bars = self.bars_ubi.pop().expect("must non-empty");
                last_bars.raw_bars.iter().map(|x| x.clone()).collect()
            };

        let bars_ubi = &mut self.bars_ubi;
        for bar in last_bar {
            if bars_ubi.len() < 2 {
                bars_ubi.push(Rc::new(NewBar {
                    symbol: bar.borrow().symbol.clone(),
                    id: bar.borrow().id,
                    dt: bar.borrow().dt,
                    freq: bar.borrow().freq,
                    open: bar.borrow().open,
                    close: bar.borrow().close,
                    high: bar.borrow().high,
                    low: bar.borrow().low,
                    vol: bar.borrow().vol,
                    amount: bar.borrow().amount,
                    cache: Default::default(),
                    raw_bars: vec![bar.clone()],
                }))
            } else {
                let mut iter = bars_ubi.iter().rev().take(2);
                let (k1, k2) = (iter.next(), iter.next());
                let (has_include, k3) = remove_include(k1.unwrap(), k2.unwrap(), bar.clone());
                if has_include {
                    let len = bars_ubi.len();
                    bars_ubi[len - 1] = Rc::new(k3);
                } else {
                    bars_ubi.push(Rc::new(k3));
                }
            }
        }

        self.update_bi();

        let _ = self.bi_list.drain(0..self.settings.max_bi_num);
        {
            let sdt = self.bi_list[0].fx_a.elements[0].dt;
            let mut s_index = 0;
            for (i, bar) in self.bars_raw.iter().enumerate() {
                if bar.borrow().dt >= sdt {
                    s_index = i;
                    break;
                }
            }
            let _ = self.bars_raw.drain(0..s_index);
        }
    }

    fn update_bi(&mut self) {
        if self.bi_list.is_empty() {
            let fxs = check_fxs(&self.bars_ubi);
            if fxs.is_empty() {
                return;
            }
            let fx_a = match fxs[0].mark {
                Mark::D => fxs
                    .iter()
                    .filter(|x| x.mark == Mark::D)
                    .min_by(|x, y| x.low.partial_cmp(&y.low).unwrap()),
                Mark::G => fxs
                    .iter()
                    .filter(|x| x.mark == Mark::G)
                    .max_by(|x, y| x.high.partial_cmp(&y.high).unwrap()),
            }
            .unwrap();
            self.bars_ubi.retain(|x| x.dt >= fx_a.elements[0].dt);
        }

        let benchmark = if self.settings.bi_change_threshold > 0.0f32 && self.bi_list.len() >= 5 {
            Some(
                self.bi_list
                    .iter()
                    .rev()
                    .take(5)
                    .map(|x| x.power_price())
                    .sum::<f32>()
                    / 5.0,
            )
        } else {
            None
        };

        let bi = check_bi(&mut self.bars_ubi, None, &self.settings);
        if bi.is_some() {
            self.bi_list.push(bi.unwrap());
        }

        if let Some(last_bi) = self.bi_list.last_mut() {
            if (last_bi.direction == Direction::Up
                && self.bars_ubi.last().unwrap().high > last_bi.high())
                || (last_bi.direction == Direction::Down
                    && self.bars_ubi.last().unwrap().low < last_bi.low())
            {
                let len = last_bi.bars.len();
                let mut new_ubi = vec![];
                for i in 0..last_bi.bars.len() - 2 {
                    new_ubi.push(last_bi.bars[i].clone());
                }
                for x in &self.bars_ubi {
                    if x.dt > last_bi.bars[len - 2].dt {
                        new_ubi.push(x.clone());
                    }
                }
                self.bars_ubi = new_ubi;
            }
        }
    }
}

fn remove_include(k1: &NewBar, k2: &NewBar, k3: Rc<RefCell<Bar>>) -> (bool, NewBar) {
    let k4 = k3;
    let direction = if k1.high < k2.high {
        Direction::Up
    } else if k1.high < k2.high {
        Direction::Down
    } else {
        let k3 = k4.borrow();
        let k4 = NewBar {
            symbol: k3.symbol.clone(),
            id: k3.id,
            freq: k3.freq,
            dt: k3.dt,
            open: k3.open,
            close: k3.close,
            high: k3.high,
            low: k3.low,
            vol: k3.vol,
            amount: k3.amount,
            cache: Default::default(),
            raw_bars: vec![k4.clone()],
        };
        return (false, k4);
    };

    return if (k2.high <= k4.borrow().high && k2.low >= k4.borrow().low)
        || (k2.high >= k4.borrow().high && k2.low <= k4.borrow().low)
    {
        let k3 = k4.borrow();

        let (high, low) = if direction == Direction::Up {
            (k2.high.max(k3.high), k2.low.max(k3.low))
        } else {
            (k2.high.min(k3.high), k2.low.min(k3.low))
        };
        let dt = if direction == Direction::Up {
            if k2.high > k3.high {
                k2.dt
            } else {
                k3.dt
            }
        } else {
            if k2.low < k3.low {
                k2.dt
            } else {
                k3.dt
            }
        };

        let (open, close) = if k3.open > k3.close {
            (high, low)
        } else {
            (low, high)
        };
        let vol = k2.vol + k3.vol;
        let amount = k2.amount + k3.amount;
        let mut elements = vec![];
        for x in &k2.raw_bars {
            if x.borrow().dt != k3.dt {
                elements.push(x.clone());
            }
            if elements.len() > 100 {
                break;
            }
        }
        let symbol = k3.symbol.clone();
        elements.push(k4.clone());
        let k4 = NewBar {
            symbol,
            id: k2.id,
            freq: k2.freq,
            dt,
            open,
            close,
            high,
            low,
            vol,
            amount,
            cache: Default::default(),
            raw_bars: elements,
        };
        (true, k4)
    } else {
        let k3 = k4.borrow();
        let k4 = NewBar {
            symbol: k3.symbol.clone(),
            id: k3.id,
            freq: k3.freq,
            dt: k3.dt,
            open: k3.open,
            close: k3.close,
            high: k3.high,
            low: k3.low,
            vol: k3.vol,
            amount: k3.amount,
            raw_bars: vec![k4.clone()],
            cache: Default::default(),
        };
        (false, k4)
    };
}

fn check_fx(k1: Rc<NewBar>, k2: Rc<NewBar>, k3: Rc<NewBar>) -> Option<FX> {
    let mut fx = None;
    if (k1.high < k2.high && k2.high > k3.high) && (k1.low < k2.low && k2.low > k3.low) {
        fx = Some(FX {
            symbol: k1.symbol.clone(),
            dt: k2.dt,
            mark: Mark::G,
            high: k2.high,
            low: k2.low,
            fx: k2.high,
            elements: vec![k1, k2, k3],
            cache: Default::default(),
        })
    } else if (k1.low > k2.low && k2.low < k3.low) && (k1.high > k2.high && k2.high < k3.high) {
        fx = Some(FX {
            symbol: k1.symbol.clone(),
            dt: k2.dt,
            mark: Mark::D,
            high: k2.high,
            low: k2.low,
            fx: k2.low,
            elements: vec![k1, k2, k3],
            cache: Default::default(),
        })
    }
    return fx;
}

fn check_fxs(bars: &Vec<Rc<NewBar>>) -> Vec<Rc<FX>> {
    let mut fxs: Vec<Rc<FX>> = vec![];
    for i in 1..bars.len() - 1 {
        let fx_ = check_fx(bars[i - 1].clone(), bars[i].clone(), bars[i + 1].clone());
        if let Some(fx) = fx_ {
            if fxs.len() >= 2 && fx.mark == fxs[fxs.len() - 1].mark {
                error!(
                    "check_fxs错误: {}，{:?}，{:?}",
                    bars[i].dt,
                    fx.mark,
                    fxs[fxs.len() - 1].mark
                );
            } else {
                fxs.push(Rc::new(fx))
            }
        }
    }
    return fxs;
}

fn check_bi(bars: &mut Vec<Rc<NewBar>>, benchmark: Option<f32>, settings: &Settings) -> Option<BI> {
    let mut fxs = check_fxs(&bars);
    if fxs.len() < 2 {
        return None;
    }

    let fx_a = fxs[0].clone();
    let (direction, fx_b) = match fx_a.mark {
        Mark::D => (
            Direction::Up,
            fxs.iter()
                .rev()
                .filter(|x| x.mark == Mark::G && x.dt > fx_a.dt && x.fx > fx_a.fx)
                .max_by(|a, b| a.high.partial_cmp(&b.high).unwrap()),
        ),
        Mark::G => (
            Direction::Down,
            fxs.iter()
                .filter(|x| x.mark == Mark::D && x.dt > fx_a.dt && x.fx < fx_a.fx)
                .min_by(|a, b| a.low.partial_cmp(&b.low).unwrap()),
        ),
    };
    if fx_b.is_none() {
        return None;
    }
    let fx_b = fx_b.unwrap().clone();
    let ab_include = (fx_a.high > fx_b.high && fx_a.low < fx_b.low)
        || (fx_a.high < fx_b.high && fx_a.low > fx_b.low);
    let power_enough = benchmark.is_none()
        || (fx_a.fx - fx_b.fx).abs() > benchmark.unwrap() * settings.bi_change_threshold;
    let bars_a: Vec<_> = bars
        .iter()
        .filter(|x| x.dt >= fx_a.elements[0].dt && x.dt <= fx_b.elements[2].dt)
        .map(|x| x.clone())
        .collect();
    let length_enough = match settings.bi_type {
        BiType::Modern => {
            bars_a.len() >= 5
                || (bars_a.len() == 4
                    && bars_a.iter().map(|x| x.raw_bars.len() as i32).sum::<i32>() >= 5)
        }
        BiType::Legacy => bars_a.len() >= 5,
        BiType::FourK => bars_a.len() >= 4,
    };
    if (!ab_include) && (length_enough || power_enough) {
        bars.retain(|x| x.dt >= fx_b.elements[0].dt);
        fxs.retain(|x| x.dt >= fx_a.elements[0].dt && x.dt <= fx_b.elements[2].dt);
        let bi = Some(BI {
            symbol: fx_a.symbol.clone(),
            fx_a,
            fx_b,
            fxs,
            direction,
            bars: bars_a,
            cache: Default::default(),
        });
        return bi;
    }
    return None;
}
