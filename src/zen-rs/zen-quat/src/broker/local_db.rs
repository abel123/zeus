use crate::broker::r#trait::Broker;
use crate::broker::zen::{Store, Zen};
use crate::db::establish_connection;
use crate::db::models::BarHistory;
use diesel::{ExpressionMethods, QueryDsl, RunQueryDsl, SelectableHelper, SqliteConnection};
use diesel_logger::LoggingConnection;
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
    conn: RefCell<LoggingConnection<SqliteConnection>>,
}

impl LocalDB {
    pub fn new() -> Self {
        Self {
            store: Rc::new(RefCell::new(Store::new())),
            conn: RefCell::new(LoggingConnection::new(establish_connection())),
        }
    }
}

impl Broker for LocalDB {
    async fn try_subscribe(
        &self,
        contract: &Contract,
        freq: Freq,
        from: i64,
        to: i64,
        cout_back: isize,
        _: bool,
    ) -> Result<(), Error> {
        let subscribed = {
            let zen = { self.get_czsc(contract, freq.clone()) };
            let zen = zen.read().await;
            zen.subscribed
        };
        if subscribed {
            return Ok(());
        }

        let mut zen = { self.get_czsc(contract, freq) };
        let mut zen = zen.write().await;
        if zen.subscribed {
            return Ok(());
        }
        zen.reset();

        let freq_bak = freq;
        {
            use crate::schema::bar_history::dsl::*;
            let contract = contract.clone();

            let bars = bar_history
                .filter(symbol.eq(contract.symbol.clone()))
                .filter(freq.eq(freq_bak.as_str()))
                .order(dt.desc())
                .limit(1500)
                .select(BarHistory::as_select())
                .load(&mut *self.conn.borrow_mut())
                .expect("TODO: panic message");

            for bar in bars.iter().rev() {
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
