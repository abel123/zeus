use crate::client::market_data::realtime::{TickOptionComputation, TickType};
use crate::messages::ResponseMessage;
use crate::Error;
use std::ops::Sub;

pub fn decode_tick_option_computation_msg(
    message: &mut ResponseMessage,
) -> Result<TickOptionComputation, Error> {
    let version = message.next_int()?;
    let req_id = message.next_int()?;
    let tick_type = message.next_int()?;
    let mut implied_vol = message.next_double()?;
    if (implied_vol - (-1.0f64)).abs() < f64::EPSILON {
        // -1 is the "not yet computed" indicator
        implied_vol = f64::MAX;
    }

    let mut delta = message.next_double()?;
    if (delta - (-2.0f64)).abs() < f64::EPSILON {
        // -2 is the "not yet computed" indicator
        delta = f64::MAX;
    }
    let mut opt_price = f64::MAX;
    let mut pv_dividend = f64::MAX;
    let mut gamma = f64::MAX;
    let mut vega = f64::MAX;
    let mut theta = f64::MAX;
    let mut und_price = f64::MAX;
    if version >= 6
        || tick_type == TickType::MODEL_OPTION as i32
        || tick_type == TickType::DELAYED_MODEL_OPTION as i32
    {
        // introduced in version == 5
        opt_price = message.next_double()?;
        if opt_price.sub(-1.0f64).abs() < f64::EPSILON {
            // -1 is the "not yet computed" indicator
            opt_price = f64::MAX;
        }
        pv_dividend = message.next_double()?;
        if pv_dividend.sub(-1.0f64).abs() < f64::EPSILON {
            // -1 is the "not yet computed" indicator
            pv_dividend = f64::MAX;
        }
    }
    if version >= 6 {
        gamma = message.next_double()?;
        if gamma.sub(-2.0f64).abs() < f64::EPSILON {
            // -2 is the "not yet computed" indicator
            gamma = f64::MAX;
        }
        vega = message.next_double()?;
        if vega.sub(-2.0f64).abs() < f64::EPSILON {
            // -2 is the "not yet computed" indicator
            vega = f64::MAX;
        }
        theta = message.next_double()?;
        if theta.sub(-2.0f64).abs() < f64::EPSILON {
            // -2 is the "not yet computed" indicator
            theta = f64::MAX;
        }
        und_price = message.next_double()?;
        if und_price.sub(-1.0f64).abs() < f64::EPSILON {
            // -1 is the "not yet computed" indicator
            und_price = f64::MAX;
        }
    }

    Ok(TickOptionComputation {
        tick_type: TickType::from(tick_type),
        implied_vol,
        delta,
        opt_price,
        pv_dividend,
        gamma,
        vega,
        theta,
        und_price,
    })
}
