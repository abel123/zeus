use crate::calculate::r#trait::Processor;
use crate::calculate::zen_cache::SMATrackerCache;
use std::collections::HashMap;
use talipp::indicator::sma::SMA;
use talipp::indicator::Indicator;
use zen_core::objects::enums::Direction;
use zen_core::objects::trade::Signal;
use zen_core::{Bar, CZSC};

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

pub fn process(czsc: &mut CZSC, is_new: bool, start: Option<(Bar, Direction)>) -> Vec<Signal> {
    let smas = czsc.cache.get_mut::<SMATrackerCache>().unwrap();
    let last_price = czsc.bars_raw.last().unwrap().borrow().close;
    for p in &smas.periods {
        smas.store.get_mut(p).and_then(|sma| {
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
