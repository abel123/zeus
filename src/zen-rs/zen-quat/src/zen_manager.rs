use std::cell::RefCell;
use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::fs;
use std::num::NonZeroUsize;
use std::ops::Deref;
use std::rc::Rc;

use futures_util::StreamExt;
use lru::LruCache;
use notify_rust::Notification;
use time::{format_description, Duration, OffsetDateTime};
use tokio::select;
use tokio::sync::oneshot::{channel, Sender};
use tokio::sync::RwLock;
use tokio::task::spawn_local;
use tokio::time::sleep;
use tokio_util::sync::CancellationToken;
use tracing::{debug, error, info};

use tws_rs::client::market_data::historical;
use tws_rs::client::market_data::historical::{
    cancel_historical_data, historical_data, BarSize, TWSDuration, WhatToShow,
};
use tws_rs::contracts::Contract;
use tws_rs::messages::ResponseMessage;
use tws_rs::{Client, ClientRef, Error};
use zen_core::objects::enums::Freq;
use zen_core::objects::trade::{Matcher, Signal};
use zen_core::{Bar, Settings, CZSC};

use crate::calculate::macd_area::MacdArea;
use crate::calculate::r#trait::Processor;
use crate::calculate::sma_tracker::SMATracker;

pub(crate) struct Zen {
    pub czsc: CZSC,
    contract: Contract,
    freq: Freq,
    pub subscribed: bool,
    pub realtime: bool,
    setting: Settings,
    token: Option<CancellationToken>,
    pub(crate) request_id: i32,
    pub(crate) bc_processor: MacdArea,
    pub(crate) tracker: SMATracker,
    signals: Vec<Signal>,
}

impl Drop for Zen {
    fn drop(&mut self) {
        self.token.take().map(|t| t.cancel());
    }
}
impl Zen {
    pub fn new(sym: Contract, freq: Freq, setting: Settings) -> Self {
        Self {
            czsc: CZSC::new(sym.symbol.clone(), freq, setting.clone()),
            contract: sym,
            freq,
            subscribed: false,
            realtime: false,
            setting,
            token: None,
            request_id: 0,
            bc_processor: MacdArea::new(1),
            tracker: SMATracker::new(vec![15, 30, 60, 120, 200]),
            signals: vec![],
        }
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
        self.token = None;
        self.bc_processor.beichi_tracker.clear();
        self.tracker = SMATracker::new(vec![15, 30, 60, 120, 200]);
    }

    pub fn update(&mut self, bar: Bar) {
        let is_new = self.czsc.update(bar);
        self.signals = self.bc_processor.process(&self.czsc, is_new);
        self.tracker.process(&self.czsc, is_new);
    }
    pub fn need_subscribe(&self, from: i64, to: i64) -> bool {
        if false {
            debug!(
                "need_subscribe {:?} {:?} {} {} {:?}-{:?} required {:?}-{:?}",
                self.contract.symbol,
                self.freq,
                self.subscribed,
                self.realtime,
                self.czsc.start(),
                self.czsc.end(),
                OffsetDateTime::from_unix_timestamp(from),
                OffsetDateTime::from_unix_timestamp(to)
            );
        }
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
        //else {
        if self.czsc.start().is_none() {
            return true;
        }
        return self.czsc.start().unwrap().unix_timestamp() > from;
    }
}
pub(crate) struct Store {
    store: HashMap<(Contract, Freq), Rc<RefCell<Zen>>>,
    setting: Settings,
    pub lock: HashMap<(Contract, Freq), Rc<RwLock<()>>>,
    notify_dedup: LruCache<String, bool>,
}

impl Store {
    pub fn new() -> Self {
        Self {
            store: Default::default(),
            setting: Settings::new().expect("config init error"),
            lock: Default::default(),
            notify_dedup: LruCache::new(NonZeroUsize::new(1000).unwrap()),
        }
    }

    fn onerror(&mut self, rsp: ResponseMessage) {
        debug!("onerror {:?}", rsp);
        match rsp.fields[3].as_str() {
            "1100" => self.store.iter().for_each(|(k, v)| {
                v.borrow_mut().reset();
            }),
            _ => {}
        }
    }
    pub fn get_czsc(&mut self, sym: &Contract, freq: Freq) -> Rc<RefCell<Zen>> {
        match self.store.entry((sym.clone(), freq)) {
            Entry::Occupied(o) => o.get().clone(),
            Entry::Vacant(v) => {
                self.lock
                    .insert((sym.clone(), freq), Rc::new(RwLock::new(())));
                v.insert(Rc::new(RefCell::new(Zen::new(
                    sym.clone(),
                    freq,
                    self.setting.clone(),
                ))))
                .clone()
            }
        }
    }

    pub fn get_or_insert_czsc_lock(&mut self, sym: &Contract, freq: Freq) -> Rc<RwLock<()>> {
        match self.lock.entry((sym.clone(), freq)) {
            Entry::Occupied(o) => self.get_czsc_lock(sym, freq),
            Entry::Vacant(v) => {
                self.lock
                    .insert((sym.clone(), freq), Rc::new(RwLock::new(())));
                self.get_czsc_lock(sym, freq)
            }
        }
    }

    pub fn get_czsc_lock(&self, sym: &Contract, freq: Freq) -> Rc<RwLock<()>> {
        self.lock.get(&(sym.clone(), freq)).unwrap().clone()
    }
    pub fn process(&mut self, sym: &Contract, dt: OffsetDateTime) {
        let mut signals = vec![];
        self.store.iter().for_each(|x| {
            if x.0 .0.symbol == sym.symbol {
                signals.append(x.1.borrow().signals.clone().as_mut())
            }
        });
        if signals.len() > 0 {
            //debug!("{}, signals {:?}", dt, signals);
        }
        self.setting.matcher.as_ref().and_then(|m| {
            let event = m.is_match(signals);
            if event.is_some() {
                //debug!("event: {:?}", event);
                if let Some((ev, factor)) = event {
                    if ev.enable_notify && factor.enable_notify {
                        let key = format!(
                            "{:?}{:?}{:?}",
                            ev,
                            factor,
                            dt.format(
                                &format_description::parse("[year]-[month]-[day] [hour]:[minute]",)
                                    .unwrap()
                            )
                            .unwrap()
                        );
                        if self.notify_dedup.get(key.as_str()).is_none() {
                            Notification::new()
                                .summary(
                                    format!(
                                        "{} - {}",
                                        //ev.name,
                                        factor.name,
                                        dt.format(
                                            &format_description::parse(
                                                "[month]-[day] [hour]:[minute]:[second]",
                                            )
                                            .unwrap()
                                        )
                                        .unwrap()
                                    )
                                    .as_str(),
                                )
                                .subtitle("✅")
                                .body(
                                    factor
                                        .signals_all
                                        .iter()
                                        .map(|x| format!("{:?}", x))
                                        .collect::<Vec<_>>()
                                        .join("\n")
                                        .as_str(),
                                )
                                .show()
                                .unwrap();
                            self.notify_dedup.push(key, true);
                        }
                    }
                }
            }
            Some(())
        });
    }
}
pub(crate) struct ZenManager {
    pub client: RwLock<Rc<RefCell<Option<ClientRef>>>>,
    pub store: Rc<RefCell<Store>>,
}

pub(crate) type AppZenMgr = Rc<RefCell<ZenManager>>;
impl ZenManager {
    pub fn new() -> Self {
        Self {
            client: RwLock::new(Rc::new(RefCell::new(None))),
            store: Rc::new(RefCell::new(Store::new())),
        }
    }

    pub async fn connect(&self) -> Result<(), Error> {
        if self.client.read().await.borrow().is_some() {
            return Ok(());
        }

        let cref = self.client.write().await;
        let mut client = Client::new("127.0.0.1:14001", 4322);
        info!("connecting to TWS");
        let client_ref = client.connect().await?;
        info!("connected");
        let store = self.store.clone();
        spawn_local(async move {
            client
                .blocking_process(move |m| {
                    store.borrow_mut().onerror(m);
                })
                .await?;
            Ok::<(), Error>(())
        });
        *cref.borrow_mut() = Some(client_ref);
        Ok(())
    }
    pub fn freq_map() -> HashMap<String, Freq> {
        HashMap::from([
            ("1D".to_string(), Freq::D),
            ("1M".to_string(), Freq::M),
            ("1W".to_string(), Freq::W),
            ("240".to_string(), Freq::F240),
            ("480".to_string(), Freq::F480),
            ("120".to_string(), Freq::F120),
            ("60".to_string(), Freq::F60),
            ("30".to_string(), Freq::F30),
            ("20".to_string(), Freq::F20),
            ("15".to_string(), Freq::F15),
            ("10".to_string(), Freq::F10),
            ("5".to_string(), Freq::F5),
            ("3".to_string(), Freq::F3),
            ("2".to_string(), Freq::F2),
            ("1".to_string(), Freq::F1),
        ])
    }
    pub async fn cancel_historical_data(&self, request_id: i32) -> Result<(), Error> {
        let _ = self.connect().await;
        let client = { self.client.read().await.clone() };
        let client = client.borrow();
        let client = client.as_ref().unwrap();
        cancel_historical_data(client, request_id).await?;
        Ok(())
    }
    pub fn get_czsc(&self, contract: &Contract, freq: Freq) -> Rc<RefCell<Zen>> {
        self.store.borrow_mut().get_czsc(contract, freq)
    }
    pub async fn try_subscribe(
        mgr: Rc<RefCell<Self>>,
        contract: &Contract,
        freq: Freq,
        from: i64,
        to: i64,
    ) -> Result<(), Error> {
        let c = contract.clone();
        let subscribe = {
            {
                let _ = mgr
                    .borrow()
                    .store
                    .borrow_mut()
                    .get_or_insert_czsc_lock(contract, freq);
            }
            let lock = mgr.borrow().store.borrow().get_czsc_lock(contract, freq);
            let _ = lock.read().await;
            let zen = mgr.borrow().get_czsc(contract, freq);
            let x = zen.borrow().need_subscribe(from, to);
            x
        };
        if subscribe {
            let lock = mgr.borrow().store.borrow().get_czsc_lock(contract, freq);
            let _ = lock.write().await;
            let zen = mgr.borrow().get_czsc(contract, freq);

            if zen.borrow().need_subscribe(from, to) {
                debug!(
                    "need_subscribe {:?} {:?} {} {} {:?}-{:?} required {:?}-{:?}",
                    zen.borrow().contract.symbol,
                    zen.borrow().freq,
                    zen.borrow().subscribed,
                    zen.borrow().realtime,
                    zen.borrow().czsc.start(),
                    zen.borrow().czsc.end(),
                    OffsetDateTime::from_unix_timestamp(from),
                    OffsetDateTime::from_unix_timestamp(to)
                );
                let (send, recv) = channel::<()>();
                spawn_local(async move {
                    mgr.borrow()
                        .subscribe_with(&c, freq, from, to, send)
                        .await
                        .expect("TODO: panic message");
                });
                return recv
                    .await
                    .map_err(|e| Error::Simple("subscribe error".to_string()));
            }
        }
        Ok(())
    }
    pub async fn subscribe_with(
        &self,
        contract: &Contract,
        freq: Freq,
        from: i64,
        to: i64,
        sender: Sender<()>,
    ) -> Result<(), Error> {
        let e = self.connect().await;
        if e.is_err() {
            error!("connect error {:?}", e);
            return e;
        }
        let client = { self.client.read().await.clone() };
        let client = client.borrow();
        let client = client.as_ref().unwrap();

        let zen = self.store.borrow_mut().get_czsc(contract, freq);
        self.cancel_historical_data(zen.borrow().request_id).await?;

        zen.borrow_mut().token.take().map(|t| t.cancel());

        let mut keep_up = OffsetDateTime::now_utc() - OffsetDateTime::from_unix_timestamp(to)?
            < Duration::days(365);
        if freq == Freq::D || freq == Freq::M || freq == Freq::S || freq == Freq::Y {
            keep_up = OffsetDateTime::now_utc() - OffsetDateTime::from_unix_timestamp(to)?
                < Duration::days(365 * 4);
        }
        let (bars, mut stream) = historical_data(
            client,
            &contract,
            None,
            timedelta_to_duration(Duration::seconds(
                if keep_up {
                    OffsetDateTime::now_utc().unix_timestamp()
                } else {
                    to
                } - from,
            )),
            freq_convert(freq),
            Some(WhatToShow::Trades),
            true,
            keep_up,
        )
        .await?;
        //info!("cost {:?}, bars: {:?}", now.elapsed(), &bars.bars);
        let symbol = contract.symbol.clone();
        zen.borrow_mut().reset();

        for e in &bars.bars {
            zen.borrow_mut().update(Bar {
                id: 0,
                dt: e.date,
                freq,
                open: e.open as f32,
                high: e.high as f32,
                low: e.low as f32,
                vol: e.volume as f32,
                amount: 0.0,
                close: e.close as f32,
                cache: Default::default(),
                macd_4_9_9: (0.0, 0.0, 0.0),
            });
            self.store.borrow_mut().process(contract, e.date);
        }

        zen.borrow_mut().subscribed = true;
        zen.borrow_mut().realtime = keep_up;
        zen.borrow_mut().request_id = bars.request_id.unwrap();
        let token = CancellationToken::new();
        let cloned_token = token.clone();
        zen.borrow_mut().token = Some(token);

        sender.send(()).unwrap();

        loop {
            select! {
                Some(Ok(e)) = stream.next() =>{
                            let e: historical::Bar = e;
                        zen.borrow_mut().update(Bar {
                            id: 0,
                            dt: e.date,
                            freq,
                            open: e.open as f32,
                            high: e.high as f32,
                            low: e.low as f32,
                            vol: e.volume as f32,
                            amount: 0.0,
                            close: e.close as f32,
                            cache: Default::default(),
                            macd_4_9_9: (0.0,0.0,0.0)
                        });
                        self.store.borrow_mut().process(contract, e.date);

                        }
                _ = cloned_token.cancelled() => {
                    break;
                }
                else =>{
                    break;
                }
            }
        }

        Ok(())
    }
}

fn freq_convert(freq: Freq) -> BarSize {
    match freq {
        Freq::Tick => BarSize::Sec,
        Freq::F1 => BarSize::Min,
        Freq::F2 => BarSize::Min2,
        Freq::F3 => BarSize::Min3,
        Freq::F4 => {
            unreachable!()
        }
        Freq::F5 => BarSize::Min5,
        Freq::F6 => {
            unreachable!()
        }
        Freq::F10 => {
            unreachable!()
        }
        Freq::F12 => {
            unreachable!()
        }
        Freq::F15 => BarSize::Min15,
        Freq::F20 => {
            unreachable!()
        }
        Freq::F30 => BarSize::Sec30,
        Freq::F60 => BarSize::Hour,
        Freq::F120 => BarSize::Hour2,
        Freq::F240 => BarSize::Hour4,
        Freq::F480 => {
            unreachable!()
        }
        Freq::D => BarSize::Day,
        Freq::W => BarSize::Week,
        Freq::M => BarSize::Month,
        Freq::S => {
            unreachable!()
        }
        Freq::Y => {
            unreachable!()
        }
    }
}

fn timedelta_to_duration(duration: Duration) -> TWSDuration {
    if duration.as_seconds_f32() >= Duration::days(360).as_seconds_f32() {
        return TWSDuration::years(
            (duration.as_seconds_f32() / Duration::days(365).as_seconds_f32()).ceil() as i32,
        );
    } else if duration.as_seconds_f32() >= Duration::days(36).as_seconds_f32() {
        return TWSDuration::months(
            (duration.as_seconds_f32() / Duration::days(30).as_seconds_f32()).ceil() as i32,
        );
    } else if duration.as_seconds_f32() >= Duration::days(7).as_seconds_f32() {
        return TWSDuration::months(
            (duration.as_seconds_f32() / Duration::days(7).as_seconds_f32()).ceil() as i32,
        );
    } else if duration.as_seconds_f32() >= Duration::days(1).as_seconds_f32() {
        return TWSDuration::days(
            (duration.as_seconds_f32() / Duration::days(1).as_seconds_f32()).ceil() as i32 + 1,
        );
    } else {
        return TWSDuration::seconds(duration.as_seconds_f32().ceil() as i32);
    }
}
