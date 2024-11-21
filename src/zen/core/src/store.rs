use crate::analyze::{Symbol, CZSC};
use crate::calculate::beichi::buy_sell_point::{BSPoint, BuySellPoint};
use crate::calculate::others;
use crate::calculate::others::sma_tracker::SMATracker;
use crate::element::chan::{Bar, BI};
use crate::element::enums::{Direction, Freq};
use crate::element::event::Signal;
use crate::setting::Settings;
use dict_derive::{FromPyObject, IntoPyObject};
use pyo3::{pyclass, pymethods};
use serde::Serialize;
use serde_json::json;
use std::fmt::format;

#[pyclass(unsendable)]
pub(crate) struct Zen {
    pub czsc: CZSC,
    pub(crate) beichi_processor: BuySellPoint,
}

#[derive(Serialize, Debug)]
#[pyclass]
pub(super) struct ZenBiDetail {
    pub direction: String,
    pub end: f32,
    pub end_ts: i64,
    pub start: f32,
    pub start_ts: i64,
}

#[pymethods]
impl ZenBiDetail {
    fn __str__(&self) -> String {
        serde_json::to_string(&self).unwrap()
    }
    fn __repr__(&self) -> String {
        format!("{:?}", self)
    }
}

#[pymethods]
impl Zen {
    #[new]
    pub fn new(sym: Symbol, freq: Freq) -> Self {
        let mut ret = Self {
            czsc: CZSC::new(sym, freq, Settings::new().unwrap()),
            beichi_processor: BuySellPoint::new(),
        };

        ret.czsc
            .cache
            .insert(SMATracker::new(vec![15, 30, 60, 120, 200]));
        ret
    }

    pub fn append(&mut self, bar: Bar, skip_process: bool) -> Vec<Signal> {
        let is_new = self.czsc.update(bar);
        if !skip_process {
            let signals = self.beichi_processor.process(&mut self.czsc, is_new, None);
            others::sma_tracker::process(&mut self.czsc, is_new, None);
            signals
        } else {
            vec![]
        }
    }

    pub fn bi_info(&self) -> Vec<ZenBiDetail> {
        let mut ret = vec![];
        for bi in &self.czsc.bi_list {
            ret.push(ZenBiDetail {
                direction: String::from(bi.direction.as_str()),
                end: if bi.direction == Direction::Down {
                    bi.low()
                } else {
                    bi.high()
                },
                end_ts: bi.fx_b.dt.timestamp(),
                start: if bi.direction == Direction::Down {
                    bi.high()
                } else {
                    bi.low()
                },
                start_ts: bi.fx_a.dt.timestamp(),
            })
        }

        ret
    }

    pub fn bc_info(&self) -> Vec<BSPoint> {
        self.beichi_processor.beichi_tracker.clone()
    }

    pub fn json(&self) -> String {
        let last_dir = self
            .czsc
            .bi_list
            .last()
            .map(|b| b.direction.clone())
            .unwrap_or(Direction::Up);
        let unfinished = match last_dir {
            Direction::Up => {
                let bar = self
                    .czsc
                    .bars_ubi
                    .iter()
                    .skip(1)
                    .min_by(|a, b| a.low.partial_cmp(&b.low).unwrap())
                    .map(|a| a.clone())
                    .unwrap();
                ZenBiDetail {
                    direction: String::from(Direction::Down.as_str()),
                    end: bar.low,
                    end_ts: bar.dt.timestamp(),
                    start: self.czsc.bars_ubi[1].high,
                    start_ts: self.czsc.bars_ubi[1].dt.timestamp(),
                }
            }
            Direction::Down => {
                let bar = self
                    .czsc
                    .bars_ubi
                    .iter()
                    .skip(1)
                    .max_by(|a, b| a.high.partial_cmp(&b.high).unwrap())
                    .map(|a| a.clone())
                    .unwrap();
                ZenBiDetail {
                    direction: String::from(Direction::Up.as_str()),
                    end: bar.high,
                    end_ts: bar.dt.timestamp(),
                    start: self.czsc.bars_ubi[1].low,
                    start_ts: self.czsc.bars_ubi[1].dt.timestamp(),
                }
            }
        };
        json!(
                {
        "bi": {"finished": self.bi_info(), "unfinished": [unfinished]},
                "beichi": [self.bc_info()]
        })
        .to_string()
    }
}
