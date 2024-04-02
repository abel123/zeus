use std::collections::HashMap;
use std::fmt::{Debug, Formatter, Write};

use serde::ser::{SerializeTuple, SerializeTupleStruct};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde_yaml::Error;
use time::OffsetDateTime;

use crate::objects::chan::{GenericCache, BI};

#[derive(Deserialize, Serialize, Clone)]
pub struct Signal {
    #[serde(serialize_with = "Signal::ser_key_val")]
    #[serde(deserialize_with = "Signal::deser_key_val")]
    pub key: (String, String, String),
    #[serde(serialize_with = "Signal::ser_key_val")]
    #[serde(deserialize_with = "Signal::deser_key_val")]
    pub value: (String, String, String),
    pub score: u32,
}

impl Default for Signal {
    fn default() -> Self {
        Self {
            key: ("".to_string(), "other".to_string(), "other".to_string()),
            value: ("".to_string(), "other".to_string(), "other".to_string()),
            score: 0,
        }
    }
}

impl Debug for Signal {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{}：{}", self.key(), self.value()))
    }
}

impl Signal {
    pub fn is_default(kv: &(String, String)) -> bool {
        kv.1 == "other"
    }
    pub fn ser_key_val<S>(key: &(String, String, String), serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let content = if key.1 == "other" {
            format!("{}", key.0)
        } else if key.2 == "other" {
            format!("{}_{}", key.0, key.1)
        } else {
            format!("{}_{}_{}", key.0, key.1, key.2)
        };
        serializer.serialize_str(content.as_str())
    }

    pub fn deser_key_val<'de, D>(deserializer: D) -> Result<(String, String, String), D::Error>
    where
        D: Deserializer<'de>,
    {
        String::deserialize(deserializer).and_then(|s| {
            let parts: Vec<&str> = s.split("_").collect();
            let mut res = (
                parts[0].to_string(),
                "other".to_string(),
                "other".to_string(),
            );
            if parts.len() > 1 {
                res.1 = parts[1].to_string();
            }
            if parts.len() > 2 {
                res.2 = parts[2].to_string();
            }
            Ok(res)
        })
    }
    pub fn key(&self) -> String {
        return if self.key.1 == "other" {
            format!("{}", self.key.0)
        } else if self.key.2 == "other" {
            format!("{}_{}", self.key.0, self.key.1)
        } else {
            format!("{}_{}_{}", self.key.0, self.key.1, self.key.2)
        };
    }

    pub fn value(&self) -> String {
        return if self.value.1 == "other" {
            format!("{}", self.value.0)
        } else if self.value.2 == "other" {
            format!("{}_{}", self.value.0, self.value.1)
        } else {
            format!("{}_{}_{}", self.value.0, self.value.1, self.value.2)
        };
    }

    pub fn is_match(&self, other: &Self) -> bool {
        if other.score >= self.score {
            if other.value.0 == self.value.0 || self.key.0 == "other" {
                if other.value.1 == self.value.1 || self.key.1 == "other" {
                    if other.value.2 == self.value.2 || self.key.2 == "other" {
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
            .min_by(|a, b| a.partial_cmp(b).unwrap())
            .unwrap();
    }

    pub fn dd(&self) -> f32 {
        // 中枢最低点
        return self
            .bis
            .iter()
            .map(|x| x.low())
            .min_by(|a, b| a.partial_cmp(b).unwrap())
            .unwrap();
    }
    pub fn zd(&self) -> f32 {
        // 中枢下沿
        return self
            .bis
            .iter()
            .take(3)
            .map(|x| x.low())
            .max_by(|a, b| a.partial_cmp(b).unwrap())
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

#[derive(Deserialize, Serialize, Debug)]
pub struct Factor {
    // signals_all 必须全部满足的信号，至少需要设定一个信号
    pub signals_all: Vec<Signal>,
    // signals_any 满足其中任一信号，允许为空
    #[serde(skip_serializing_if = "Option::is_none")]
    signals_any: Option<Vec<Signal>>,
    // signals_not 不能满足其中任一信号，允许为空
    #[serde(skip_serializing_if = "Option::is_none")]
    signals_not: Option<Vec<Signal>>,
}

impl Factor {
    pub(super) fn is_match(&self, k_v: &HashMap<String, Signal>) -> bool {
        for s in &self.signals_all {
            if let Some(v) = k_v.get(&s.key()) {
                if !s.is_match(v) {
                    return false;
                }
            } else {
                return false;
            }
        }
        if let Some(signals_any) = &self.signals_any {
            let mut any = false;
            for s in signals_any {
                if let Some(v) = k_v.get(&s.key()) {
                    if s.is_match(v) {
                        any = true;
                        break;
                    }
                }
            }
            if !any {
                return false;
            }
        }

        if let Some(signals_not) = &self.signals_not {
            for s in signals_not {
                if let Some(v) = k_v.get(&s.key()) {
                    if s.is_match(v) {
                        return false;
                    }
                }
            }
        }
        return true;
    }
}
#[derive(Deserialize, Serialize, Debug)]
pub enum Operate {
    HL, // Hold Long
    HS, // Hold Short
    HO, // Hold Other
    LO, // Long Open
    LE, // Long Exit
    SO, // Short Open
    SE, // Short Exit
}

#[derive(Deserialize, Serialize, Debug)]
pub struct Event {
    pub name: String,
    // 多个信号组成一个因子，多个因子组成一个事件。
    // 单个事件是一系列同类型因子的集合，事件中的任一因子满足，则事件为真。
    factors: Vec<Factor>,
    // signals_all 必须全部满足的信号，允许为空
    #[serde(skip_serializing_if = "Option::is_none")]
    signals_all: Option<Vec<Signal>>,
    // signals_any 满足其中任一信号，允许为空
    #[serde(skip_serializing_if = "Option::is_none")]
    signals_any: Option<Vec<Signal>>,
    // signals_not 不能满足其中任一信号，允许为空
    #[serde(skip_serializing_if = "Option::is_none")]
    signals_not: Option<Vec<Signal>>,
    operate: Operate,
    pub enable_notify: bool,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct Matcher(Vec<Event>);

impl Matcher {
    pub fn is_match(&self, signals: Vec<Signal>) -> Option<(&Event, &Factor)> {
        let mut k_v: HashMap<String, Signal> = HashMap::new();

        for s in signals {
            k_v.insert(s.key(), s.clone());
        }

        for event in &self.0 {
            let mut factor_matched = None;
            for f in &event.factors {
                if f.is_match(&k_v) {
                    factor_matched = Some(f);
                    break;
                }
            }
            if !factor_matched.is_some() {
                continue;
            }

            let mut br = false;
            if let Some(signals_all) = &event.signals_all {
                for s in signals_all {
                    if let Some(v) = k_v.get(&s.key()) {
                        if !s.is_match(v) {
                            br = true;
                            break;
                        }
                    } else {
                        br = true;
                        break;
                    }
                }
            }
            if br {
                continue;
            }

            if let Some(signals_any) = &event.signals_any {
                let mut any = false;
                for s in signals_any {
                    if let Some(v) = k_v.get(&s.key()) {
                        if s.is_match(v) {
                            any = true;
                            break;
                        }
                    }
                }
                if !any {
                    continue;
                }
            }

            if let Some(signals_not) = &event.signals_not {
                for s in signals_not {
                    if let Some(v) = k_v.get(&s.key()) {
                        if s.is_match(v) {
                            br = true;
                            break;
                        }
                    }
                }
            }
            if br {
                continue;
            }
            return Some((event, factor_matched.unwrap()));
        }
        return None;
    }

    pub fn from(content: &str) -> Result<Self, Error> {
        let events: Vec<Event> = serde_yaml::from_str(content)?;
        Ok(Matcher(events))
    }
}

#[cfg(test)]
mod tests {
    use tracing::debug;
    use tracing_test::traced_test;

    use crate::objects::trade::{Event, Factor, Operate, Signal};

    #[traced_test]
    #[test]
    fn it_works() {
        let mut events = vec![];
        events.push(Event {
            name: "event a".to_string(),
            factors: vec![Factor {
                signals_all: vec![Signal {
                    key: ("k1".to_string(), "k2".to_string(), "k3".to_string()),
                    value: ("v1".to_string(), "v2".to_string(), "v3".to_string()),
                    score: 70,
                }],
                signals_any: None,
                signals_not: None,
                name: "".to_string(),
                enable_notify: false,
            }],
            signals_all: None,
            signals_any: None,
            signals_not: None,
            operate: Operate::HL,
            enable_notify: false,
        });

        let yml = serde_yaml::to_string(&events);
        let yml = yml.unwrap();
        debug!("yml \n{}", yml.clone());

        let events: Vec<Event> = serde_yaml::from_str(yml.clone().as_str()).unwrap();
        debug!("events {:?}", events);
    }
}
