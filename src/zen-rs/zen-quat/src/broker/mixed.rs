use crate::broker::ib::IB;
use crate::broker::local_db::LocalDB;
use crate::broker::r#trait::Broker;
use crate::broker::zen::Zen;
use std::cell::RefCell;
use std::rc::Rc;
use tokio::sync::RwLock;
use tws_rs::contracts::Contract;
use tws_rs::Error;
use zen_core::objects::enums::Freq;

pub struct Mixed {
    local_db: LocalDB,
    pub ib: Rc<RefCell<IB>>,
}

pub(crate) type MixedBroker = Rc<RefCell<Mixed>>;

impl Mixed {
    pub fn new() -> Self {
        Self {
            local_db: LocalDB::new(),
            ib: Rc::new(RefCell::new(IB::new())),
        }
    }

    pub async fn try_subscribe(
        &self,
        local: bool,
        contract: &Contract,
        freq: Freq,
        from: i64,
        to: i64,
        cout_back: isize,
        non_realtime: bool,
    ) -> Result<(), Error> {
        if local {
            return self
                .local_db
                .try_subscribe(contract, freq, from, to, cout_back, non_realtime)
                .await;
        } else {
            return IB::try_subscribe(self.ib.clone(), contract, freq, from, to, non_realtime)
                .await;
        }
    }

    pub fn get_czsc(&self, local: bool, contract: &Contract, freq: Freq) -> Rc<RwLock<Zen>> {
        if local {
            return self.local_db.get_czsc(contract, freq);
        } else {
            return self.ib.borrow().get_czsc(contract, freq);
        }
    }
}
