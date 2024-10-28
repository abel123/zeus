use crate::broker::zen::Zen;
use std::rc::Rc;
use tokio::sync::RwLock;
use tws_rs::contracts::Contract;
use tws_rs::Error;
use zen_core::objects::enums::Freq;

pub trait Broker {
    async fn try_subscribe(
        &self,
        contract: &Contract,
        freq: Freq,
        from: i64,
        to: i64,
        cout_back: isize,
        non_realtime: bool,
    ) -> Result<(), Error>;
    fn get_czsc(&self, contract: &Contract, freq: Freq) -> Rc<RwLock<Zen>>;
}
