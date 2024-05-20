use std::collections::HashMap;
use zen_core::objects::enums::Freq;

struct BuySellPoint {}

impl BuySellPoint {
    pub fn next_freq() -> HashMap<Freq, Freq> {
        HashMap::from([
            (Freq::W, Freq::D),
            (Freq::D, Freq::F60),
            (Freq::F60, Freq::F15),
            (Freq::F15, Freq::F3),
            (Freq::F5, Freq::F1),
        ])
    }
}
