use crate::broker::ib::IB;
use crate::broker::local_db::LocalDB;
use crate::broker::r#trait::Broker;
use crate::broker::zen::Zen;
use std::rc::Rc;
use tokio::sync::RwLock;
use tws_rs::contracts::Contract;
use tws_rs::Error;
use zen_core::objects::enums::Freq;

pub struct Mixed {
    local_db: LocalDB,
    ib: IB,
}

impl Mixed {
    pub fn new() -> Self {
        Self {
            local_db: LocalDB::new(),
            ib: IB::new(),
        }
    }

    async fn try_subscribe(
        &mut self,
        local: bool,
        contract: &Contract,
        freq: Freq,
        from: i64,
        to: i64,
        cout_back: isize,
        non_realtime: bool,
    ) -> Result<(), Error> {
        todo!()
    }
    fn get_czsc(&self, local: bool, contract: &Contract, freq: Freq) -> Rc<RwLock<Zen>> {
        todo!()
    }
}
