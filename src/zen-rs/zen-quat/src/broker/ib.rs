use cached::CachedAsync;
use std::cell::RefCell;
use std::cmp::max;
use std::collections::hash_map::Entry;
use std::collections::{HashMap, VecDeque};
use std::fs;
use std::num::NonZeroUsize;
use std::ops::Deref;
use std::rc::Rc;

use futures_util::StreamExt;
use lru::LruCache;
use notify_rust::Notification;
use time::macros::offset;
use time::{format_description, Duration, OffsetDateTime, UtcOffset};
use tokio::select;
use tokio::sync::oneshot::{channel, Sender};
use tokio::sync::RwLock;
use tokio::task::spawn_local;
use tokio::time::sleep;
use tokio_util::sync::CancellationToken;
use tracing::{debug, error, info, warn};

use crate::broker::r#trait::Broker;
use crate::broker::zen::{Store, Zen};
use crate::calculate::macd_area::MacdArea;
use crate::calculate::r#trait::Processor;
use crate::calculate::sma_tracker::SMATracker;
use crate::db::models::Symbol;
use crate::utils::notify::Notify;
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

pub(crate) struct IB {
    pub client: RwLock<Rc<RefCell<Option<ClientRef>>>>,
    pub store: Rc<RefCell<Store>>,
}

impl Broker for IB {
    async fn try_subscribe(
        &self,
        contract: &Contract,
        freq: Freq,
        from: i64,
        to: i64,
        cout_back: isize,
        non_realtime: bool,
    ) -> Result<(), Error> {
        todo!()
    }

    fn get_czsc(&self, contract: &Contract, freq: Freq) -> Rc<RwLock<Zen>> {
        todo!()
    }
}
impl IB {
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
        let mut store = self.store.clone();
        spawn_local(async move {
            let callback = move |m| {
                store.borrow_mut().onerror(m);
            };
            client.blocking_process(callback).await?;
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
    pub fn get_czsc(&self, contract: &Contract, freq: Freq) -> Rc<RwLock<Zen>> {
        { self.store.borrow_mut().get_czsc(contract, freq) }.clone()
    }
    pub async fn try_subscribe(
        mgr: Rc<RefCell<Self>>,
        contract: &Contract,
        freq: Freq,
        from: i64,
        to: i64,
        replay: bool,
    ) -> Result<(), Error> {
        let c = contract.clone();
        let subscribe = {
            let zen = { mgr.borrow().get_czsc(contract, freq) };
            let zen = zen.read().await;
            let x = zen.need_subscribe(from, to, replay);
            x
        };
        if subscribe {
            let (send, recv) = channel::<()>();
            spawn_local(async move {
                mgr.borrow()
                    .subscribe_with(&c, freq, from, to, replay, send)
                    .await
                    .expect("TODO: panic message");
            });
            return recv
                .await
                .map_err(|e| Error::Simple("subscribe error".to_string()));
        }
        Ok(())
    }
    pub async fn subscribe_with(
        &self,
        contract: &Contract,
        freq: Freq,
        from: i64,
        to: i64,
        replay: bool,
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

        let token = CancellationToken::new();
        let cloned_token = token.clone();

        let mut stream = {
            let mut zen = { self.store.borrow_mut().get_czsc(contract, freq) };
            let mut zen = zen.write().await;
            if !zen.need_subscribe(from, to, replay) {
                sender.send(()).unwrap();
                return Ok(());
            }

            if zen.realtime {
                self.cancel_historical_data(zen.request_id).await?;
            }

            zen.token.take().map(|t| t.cancel());

            let mut keep_up = OffsetDateTime::now_utc() - OffsetDateTime::from_unix_timestamp(to)?
                < Duration::days(365);
            if freq == Freq::D || freq == Freq::M || freq == Freq::S || freq == Freq::Y {
                keep_up = OffsetDateTime::now_utc() - OffsetDateTime::from_unix_timestamp(to)?
                    < Duration::days(365 * 4);
            }
            keep_up = keep_up && !replay;
            let mut to = to;
            if replay && !zen.realtime {
                to = max(to, zen.czsc.end().map(|e| e.unix_timestamp()).unwrap_or(0));
            }
            debug!(
                "need_subscribe {:?} {:?} {} {} {:?}-{:?} required {:?}-{:?} {} {} {:?}",
                zen.contract.symbol,
                zen.freq,
                zen.subscribed,
                zen.realtime,
                zen.czsc.start(),
                zen.czsc.end(),
                OffsetDateTime::from_unix_timestamp(from),
                OffsetDateTime::from_unix_timestamp(to),
                keep_up,
                replay,
                if keep_up {
                    OffsetDateTime::now_utc().unix_timestamp()
                } else {
                    to
                } - from
            );
            let (bars, mut stream) = historical_data(
                client,
                &contract,
                if !keep_up {
                    Some(OffsetDateTime::from_unix_timestamp(to).unwrap())
                } else {
                    None
                },
                timedelta_to_duration(
                    Duration::seconds(
                        if keep_up {
                            OffsetDateTime::now_utc().unix_timestamp()
                        } else {
                            to
                        } - from,
                    )
                    .max(Duration::days(1)),
                ),
                to_barsize(freq),
                Some(WhatToShow::Trades),
                true,
                keep_up,
            )
            .await?;
            //info!("cost {:?}, bars: {:?}", now.elapsed(), &bars.bars);
            let symbol = contract.symbol.clone();
            zen.reset();

            for e in &bars.bars {
                let signals = zen.update(e.to_bar(freq));
                {
                    self.store
                        .borrow_mut()
                        .signal_tracker
                        .insert((contract.clone(), freq), signals)
                };
            }

            zen.subscribed = true;
            zen.realtime = keep_up;
            zen.request_id = bars.request_id.unwrap();
            zen.token = Some(token);

            stream
        };

        {
            self.store.borrow().process(contract).await
        };
        sender.send(()).unwrap();

        loop {
            select! {
                Some(Ok(e)) = stream.next() =>{
                        let e: historical::Bar = e;
                        let mut zen = {self.store.borrow_mut().get_czsc(contract, freq)};
                    {

                        let mut zen = zen.write().await;
                        let signals = zen.update(e.to_bar(freq));
                        {
                            self.store
                                .borrow_mut()
                                .signal_tracker
                                .insert((contract.clone(), freq), signals)
                        };
                    }
                        self.store.borrow_mut().process(contract).await;
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

fn to_barsize(freq: Freq) -> BarSize {
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

pub fn timedelta_to_duration(duration: Duration) -> TWSDuration {
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
