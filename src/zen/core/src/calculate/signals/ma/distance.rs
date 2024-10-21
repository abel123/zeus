use crate::element::enums::Direction;
use crate::element::event::Signal;
use crate::element::chan::{Bar};
use crate::analyze::CZSC;

// 均线是很强的参考
// 均线触及（接近程度）计算
pub fn ma_distance(czsc: &mut CZSC, is_new: bool, start: Option<(Bar, Direction)>) -> Vec<Signal> {
    let smas = czsc
        .cache
        .get::<crate::calculate::zen_cache::SMATrackerCache>()
        .unwrap();

    let mut result = vec![];
    for (k, v) in &smas.store {
        result.push(Signal {
            key: ("MA".to_string(), "distance".to_string(), "".to_string()),
            value: ("D1".to_string(), format!("MA{}", k), "".to_string()),
            dt: None,
            figure: czsc
                .bars_ubi
                .last()
                .map(|x| x.raw_bars.last().map(|b| b.borrow().close).unwrap_or(0.0))
                .unwrap_or(0.0)
                / v.ma(),
            figure_max: None,
        })
    }

    result
}

// 均线密集程度
