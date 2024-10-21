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

#[pymodule]
fn zen_core(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<element::chan::Bar>()?;
    m.add_class::<element::enums::Freq>()?;
    m.add_class::<element::event::Signal>()?;
    m.add_class::<store::Zen>()?;
    m.add_class::<store::ZenBiDetail>()?;
    m.add_class::<BSPoint>()?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {}
}
