use std::ops::Sub;
use crate::element::enums::Direction;
use crate::element::event::Signal;
use crate::analyze::CZSC;

// 黄金分割位置，比例力度
pub fn length_percentage(czsc: &CZSC, dindex: usize, use_fake: bool) -> Vec<Signal> {
    let key = ("长度比例D".to_string(), "值".to_string(), "".to_string());
    if czsc.bi_list.len() < 2 {
        return vec![];
    }

    if dindex == 0 && use_fake {
        if czsc.bi_list.last().unwrap().direction == Direction::Up {
            if czsc
                .fake_bi_low()
                .sub(czsc.fake_min_low().unwrap_or(0.0))
                .abs()
                > f32::EPSILON
            {
                return vec![];
            }
        } else {
            if czsc
                .fake_bi_high()
                .sub(czsc.fake_max_high().unwrap_or(0.0))
                .abs()
                > f32::EPSILON
            {
                return vec![];
            }
        }
        let percent = (czsc.fake_min_low().unwrap() - czsc.fake_max_high().unwrap())
            / czsc.bi_list.last().unwrap().power_price().abs();
        return vec![Signal {
            key,
            value: (dindex.to_string(), percent.to_string(), "".to_string()),
            dt: None,
            figure: 0.0,
            figure_max: None,
        }];
    }
    let percent = czsc
        .bi_list
        .iter()
        .rev()
        .skip(1)
        .next()
        .unwrap()
        .power_price()
        / czsc.bi_list.last().unwrap().power_price();
    return vec![Signal {
        key,
        value: (dindex.to_string(), percent.to_string(), "".to_string()),
        dt: None,
        figure: 0.0,
        figure_max: None,
    }];
}

// 量比，量价背离程度（日级别以上）
