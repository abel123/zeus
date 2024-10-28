use notify_rust::{get_bundle_identifier_or_default, set_application};
#[allow(dead_code)]

use pyo3::prelude::*;
use crate::calculate::beichi::buy_sell_point::BSPoint;

mod element;
mod setting;
mod analyze;
mod calculate;
mod talipp;
mod utils;
mod store;

#[pyfunction]
fn init() {
    let safari_id = get_bundle_identifier_or_default("iTerm 2");
    set_application(&safari_id).expect("install iterm");
}
#[pymodule]
fn zen_core(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<element::chan::Bar>()?;
    m.add_class::<element::enums::Freq>()?;
    m.add_class::<element::event::Signal>()?;
    m.add_class::<store::Zen>()?;
    m.add_class::<store::ZenBiDetail>()?;
    m.add_class::<BSPoint>()?;
    m.add_function(wrap_pyfunction!(init, m)?)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {}
}
