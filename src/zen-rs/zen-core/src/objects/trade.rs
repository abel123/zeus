use crate::objects::chan::{GenericCache, BI};
use serde::Serialize;
use time::OffsetDateTime;

#[derive(Debug, Serialize)]
pub struct Signal {
    pub kv1: (String, String),
    pub kv2: (String, String),
    pub kv3: (String, String),
    pub score: u32,
}

impl Default for Signal {
    fn default() -> Self {
        Self {
            kv1: ("".to_string(), "".to_string()),
            kv2: ("other".to_string(), "other".to_string()),
            kv3: ("other".to_string(), "other".to_string()),
            score: 0,
        }
    }
}

impl Signal {
    pub fn key(&self) -> String {
        return if self.kv2.0 == "other" {
            format!("{}", self.kv1.0)
        } else if self.kv3.0 == "other" {
            format!("{}_{}", self.kv1.0, self.kv2.0)
        } else {
            format!("{}_{}_{}", self.kv1.0, self.kv2.0, self.kv3.0)
        };
    }

    pub fn value(&self) -> String {
        format!(
            "{}_{}_{}_{}",
            self.kv1.0, self.kv2.0, self.kv3.0, self.score
        )
    }

    pub fn is_match(&self, other: &str) -> bool {
        let parts: Vec<&str> = other.split('_').collect();
        let v1 = parts[0];
        let v2 = parts[1];
        let v3 = parts[2];
        let score: u32 = parts[3].parse().unwrap();
        if score >= self.score {
            if v1 == self.kv1.1 || self.kv1.0 == "other" {
                if v2 == self.kv2.1 || self.kv2.0 == "other" {
                    if v3 == self.kv3.1 || self.kv3.0 == "other" {
                        return true;
                    }
                }
            }
        }
        false
    }
}

pub struct ZS<'a> {
    pub bis: &'a [BI],
    cache: GenericCache,
}

impl<'a> ZS<'a> {
    pub fn new(bis: &'a [BI]) -> Self {
        Self {
            bis,
            cache: Default::default(),
        }
    }

    pub fn sdt(&self) -> OffsetDateTime {
        self.bis.first().unwrap().fx_a.dt
    }

    pub fn edt(&self) -> OffsetDateTime {
        self.bis.last().unwrap().fx_b.dt
    }

    pub fn zz(&self) -> f32 {
        self.zd() + (self.zg() - self.zd()) / 2.0
    }

    pub fn gg(&self) -> f32 {
        // 中枢最高点
        return self
            .bis
            .iter()
            .map(|x| x.high())
            .max_by(|a, b| a.partial_cmp(b).unwrap())
            .unwrap();
    }

    pub fn zg(&self) -> f32 {
        // 中枢上沿
        return self
            .bis
            .iter()
            .take(3)
            .map(|x| x.high())
            .max_by(|a, b| a.partial_cmp(b).unwrap())
            .unwrap();
    }

    pub fn dd(&self) -> f32 {
        // 中枢最低点
        return self
            .bis
            .iter()
            .map(|x| x.high())
            .min_by(|a, b| a.partial_cmp(b).unwrap())
            .unwrap();
    }
    pub fn zd(&self) -> f32 {
        // 中枢下沿
        return self
            .bis
            .iter()
            .take(3)
            .map(|x| x.high())
            .min_by(|a, b| a.partial_cmp(b).unwrap())
            .unwrap();
    }

    pub fn is_valid(&self) -> bool {
        if self.zg() < self.zd() {
            return false;
        }
        for bi in self.bis {
            if (self.zg() >= bi.high() && bi.high() >= self.zd())
                || (self.zg() >= bi.low() && bi.low() >= self.zd())
                || (bi.high() >= self.zg() && bi.low() <= self.zd())
            {
                continue;
            } else {
                return false;
            }
        }
        true
    }
}

#[derive(Serialize)]
pub struct Factor {
    // signals_all 必须全部满足的信号，至少需要设定一个信号
    signals_all: Vec<Signal>,
    // signals_any 满足其中任一信号，允许为空
    signals_any: Vec<Signal>,
    // signals_not 不能满足其中任一信号，允许为空
    signals_not: Vec<Signal>,
    name: String,
    enable_notify: bool,
}

#[derive(Serialize)]
pub enum Operate {
    HL, // Hold Long
    HS, // Hold Short
    HO, // Hold Other
    LO, // Long Open
    LE, // Long Exit
    SO, // Short Open
    SE, // Short Exit
}

#[derive(Serialize)]
pub struct Event {
    pub name: String,
    // 多个信号组成一个因子，多个因子组成一个事件。
    // 单个事件是一系列同类型因子的集合，事件中的任一因子满足，则事件为真。
    factors: Vec<Factor>,
    // signals_all 必须全部满足的信号，允许为空
    signals_all: Vec<Signal>,
    // signals_any 满足其中任一信号，允许为空
    signals_any: Vec<Signal>,
    // signals_not 不能满足其中任一信号，允许为空
    signals_not: Vec<Signal>,
    operate: Operate,
}
