use crate::calculate::beichi::buy_sell_point::BuySellPoint;
use crate::calculate::others;
use crate::calculate::others::sma_tracker::SMATracker;
use crate::calculate::r#trait::Processor;
use crate::utils::notify::Notify;
use std::collections::HashMap;
use std::rc::Rc;
use talipp::indicator::sma;
use time::OffsetDateTime;
use tokio::sync::RwLock;
use tokio_util::sync::CancellationToken;
use tracing::debug;
use tws_rs::contracts::Contract;
use tws_rs::messages::ResponseMessage;
use zen_core::objects::enums::Freq;
use zen_core::objects::trade::Signal;
use zen_core::{Bar, Settings, CZSC};

pub(crate) struct Zen {
    pub czsc: CZSC,
    pub(crate) contract: Contract,
    pub(crate) freq: Freq,
    pub(crate) last_time: OffsetDateTime,
    pub subscribed: bool,
    pub realtime: bool,
    setting: Settings,
    pub(crate) token: Option<CancellationToken>,
    pub(crate) request_id: i32,
    pub(crate) beichi_processor: BuySellPoint,
}

impl Drop for Zen {
    fn drop(&mut self) {
        self.token.take().map(|t| t.cancel());
    }
}
impl Zen {
    pub fn new(sym: Contract, freq: Freq, setting: Settings) -> Self {
        let mut res = Self {
            czsc: CZSC::new(sym.symbol.clone(), freq, setting.clone()),
            contract: sym,
            freq,
            last_time: OffsetDateTime::now_utc(),
            subscribed: false,
            realtime: false,
            setting,
            token: None,
            request_id: 0,
            beichi_processor: BuySellPoint::new(),
        };
        res.czsc
            .cache
            .insert(SMATracker::new(vec![15, 30, 60, 120, 200]));
        res
    }

    pub fn reset(&mut self) {
        self.token.take().map(|t| t.cancel());

        self.czsc = CZSC::new(
            self.contract.symbol.clone(),
            self.freq,
            self.setting.clone(),
        );
        self.subscribed = false;
        self.realtime = false;
        self.last_time = OffsetDateTime::now_utc();
        self.token = None;
        self.czsc
            .cache
            .insert(SMATracker::new(vec![15, 30, 60, 120, 200]));
        self.beichi_processor.beichi_tracker.clear();
    }

    pub fn update(&mut self, bar: Bar) -> Vec<Signal> {
        let is_new = self.czsc.update(bar);
        //let signals = self.bc_processor.process(&self.czsc, is_new);
        let signals = self.beichi_processor.process(&mut self.czsc, is_new, None);
        others::sma_tracker::process(&mut self.czsc, is_new, None);
        return signals;
    }
    pub fn need_subscribe(&self, from: i64, to: i64, replay: bool) -> bool {
        if !self.subscribed {
            return true;
        }
        if !self.realtime {
            if self.czsc.start().is_none() || self.czsc.end().is_none() {
                return true;
            }
            if self.czsc.start().unwrap().unix_timestamp() <= from
                && to <= self.czsc.end().unwrap().unix_timestamp()
            {
                return false;
            } else {
                return true;
            }
        }
        if replay {
            return true;
        }
        //else {
        if self.czsc.start().is_none() {
            return true;
        }
        return self.czsc.start().unwrap().unix_timestamp() > from;
    }
}
pub struct Store {
    store: HashMap<(Contract, Freq), Rc<RwLock<Zen>>>,
    pub(crate) signal_tracker: HashMap<(Contract, Freq), Vec<Signal>>,
    setting: Settings,
}

impl Store {
    pub fn new() -> Self {
        Self {
            store: Default::default(),
            signal_tracker: Default::default(),
            setting: Settings::new().expect("config init error"),
        }
    }

    pub(crate) fn onerror(&mut self, rsp: ResponseMessage) {
        debug!("onerror {:?}", rsp);
        match rsp.fields[3].as_str() {
            "1100"| "2103" | "2106" => {
                self.store.clear();
            }
            _ => {}
        }
    }
    pub fn get_czsc(&mut self, sym: &Contract, freq: Freq) -> Rc<RwLock<Zen>> {
        self.store
            .entry((sym.clone(), freq))
            .or_insert_with(|| {
                Rc::new(RwLock::new(Zen::new(
                    sym.clone(),
                    freq,
                    self.setting.clone(),
                )))
            })
            .clone()
    }

    pub async fn process(&self, sym: &Contract) {
        let mut signals = vec![];
        for x in &self.signal_tracker {
            if x.0 .0.symbol == sym.symbol {
                signals.append(x.1.clone().as_mut())
            }
        }
        if signals.len() > 0 {
            //debug!("{}, signals {:?}", dt, signals);
        }
        self.setting.matcher.as_ref().and_then(|m| {
            let event = m.is_match(signals);
            if event.is_some() {
                //debug!("event: {:?}", event);
                if let Some((ev, factor, dt)) = event {
                    if ev.enable_notify {
                        Notify::notify_event(sym, dt, ev, factor);
                    }
                }
            }
            Some(())
        });
    }

    pub async fn offline_process(&self, sym: &Contract) {}
}
