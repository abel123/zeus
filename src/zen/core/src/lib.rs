#[allow(dead_code)]

use pyo3::prelude::*;

mod element;
mod setting;
mod analyze;

#[pymodule]
fn zen_core(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<element::chan::Bar>();
    m.add_class::<element::enums::Freq>();
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
    }
}
