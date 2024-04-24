use crate::broker::r#trait::Broker;
use crate::broker::zen::Zen;
use std::rc::Rc;
use tokio::sync::RwLock;
use tws_rs::contracts::Contract;
use tws_rs::Error;
use zen_core::objects::enums::Freq;

pub struct LocalDB {}

impl Broker for LocalDB {
    async fn try_subscribe(
        &mut self,
        contract: &Contract,
        freq: Freq,
        from: i64,
        to: i64,
        non_realtime: bool,
    ) -> Result<(), Error> {
        todo!()
    }

    fn get_czsc(&self, contract: &Contract, freq: Freq) -> Rc<RwLock<Zen>> {
        todo!()
    }
}
