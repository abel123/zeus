use crate::calculate::r#trait::Processor;
use std::collections::HashMap;
use talipp::indicator::sma::SMA;
use talipp::indicator::Indicator;
use zen_core::objects::trade::Signal;
use zen_core::CZSC;

pub struct SMATracker {
    periods: Vec<isize>,
    pub store: HashMap<isize, SMA>,
}

impl SMATracker {
    pub fn new(periods: Vec<isize>) -> Self {
        let mut tracker = SMATracker {
            periods,
            store: Default::default(),
        };
        for p in &tracker.periods {
            tracker.store.insert(*p, SMA::new(*p));
        }
        tracker
    }
}

impl Processor for SMATracker {
    fn process(&mut self, czsc: &CZSC, is_new: bool) -> Vec<Signal> {
        let last_price = czsc.bars_raw.last().unwrap().borrow().close;
        for p in &self.periods {
            self.store.get_mut(p).and_then(|sma| {
                if is_new {
                    sma.next(last_price);
                } else {
                    sma.update(last_price);
                }
                Some(())
            });
        }
        vec![]
    }
}
