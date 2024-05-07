use crate::broker::r#trait::Broker;
use crate::broker::zen::{Store, Zen};
use crate::db::establish_connection;
use crate::db::models::{BarHistory, Symbol};
use diesel::{ExpressionMethods, QueryDsl, RunQueryDsl, SelectableHelper, SqliteConnection};
use std::cell::RefCell;
use std::rc::Rc;
use time::OffsetDateTime;
use tokio::sync::RwLock;
use tws_rs::contracts::Contract;
use tws_rs::Error;
use zen_core::objects::enums::Freq;
use zen_core::Bar;

pub struct LocalDB {
    store: Rc<RefCell<Store>>,
    conn: SqliteConnection,
}

impl LocalDB {
    pub fn new() -> Self {
        Self {
            store: Rc::new(RefCell::new(Store::new())),
            conn: establish_connection(),
        }
    }
}

impl Broker for LocalDB {
    async fn try_subscribe(
        &mut self,
        contract: &Contract,
        freq: Freq,
        from: i64,
        to: i64,
        cout_back: isize,
        _: bool,
    ) -> Result<(), Error> {
        let subscribe = {
            let zen = { self.get_czsc(contract, freq.clone()) };
            let zen = zen.read().await;
            let x = zen.need_subscribe(from, to, true);
            x
        };
        if !subscribe {
            return Ok(());
        }

        let mut zen = { self.get_czsc(contract, freq) };
        let mut zen = zen.write().await;
        if !zen.need_subscribe(from, to, true) {
            return Ok(());
        }
        zen.reset();

        let freq_bak = freq;
        {
            use crate::schema::bar_history::dsl::*;
            let contract = contract.clone();

            let bars = if cout_back > 0 {
                bar_history
                    .filter(symbol.eq(contract.symbol.clone()))
                    .filter(dt.le(to as i32))
                    .limit(cout_back as i64)
                    .order(dt.asc())
                    .select(BarHistory::as_select())
                    .load(&mut self.conn)
                    .expect("TODO: panic message")
            } else {
                bar_history
                    .filter(symbol.eq(contract.symbol.clone()))
                    .filter(dt.le(to as i32))
                    .filter(dt.gt(from as i32))
                    .order(dt.asc())
                    .select(BarHistory::as_select())
                    .load(&mut self.conn)
                    .expect("TODO: panic message")
            };
            for bar in &bars {
                let signals = zen.update(Bar {
                    id: 0,
                    dt: OffsetDateTime::from_unix_timestamp(bar.dt as i64).unwrap(),
                    freq: freq_bak,
                    open: bar.open.unwrap_or(0.0),
                    close: bar.close.unwrap_or(0.0),
                    high: bar.high.unwrap_or(0.0),
                    low: bar.low.unwrap_or(0.0),
                    vol: bar.volume.unwrap_or(0) as f32,
                    amount: 0.0,
                    cache: Default::default(),
                    macd_4_9_9: (0.0, 0.0, 0.0),
                });
                {
                    self.store
                        .borrow_mut()
                        .signal_tracker
                        .insert((contract.clone(), freq_bak), signals)
                };
            }

            zen.subscribed = true;
            zen.realtime = false;
        }

        Ok(())
    }

    fn get_czsc(&self, contract: &Contract, freq: Freq) -> Rc<RwLock<Zen>> {
        { self.store.borrow_mut().get_czsc(contract, freq) }.clone()
    }
}
