use pyo3::prelude::*;
use std::rc::Rc;
use zen_core::analyze::check_bi;
use zen_core::objects::chan::{NewBar, BI};
use zen_core::settings::BiType;
use zen_core::Settings;

#[pyfunction]
fn check_bi_py(bars: Vec<NewBar>, benchmark: Option<f32>) -> (Vec<NewBar>, Option<BI>) {
    let bars = &mut bars.into_iter().map(|x| Rc::new(x)).collect();
    let res = check_bi(
        bars,
        benchmark,
        &Settings {
            debug: false,
            bi_type: BiType::FourK,
            bi_change_threshold: 0.0,
            max_bi_num: 0,
            event_matcher_file: "".to_string(),
            matcher: None,
        },
    );
    (
        bars.into_iter()
            .map(|x| x.clone().as_ref().clone())
            .collect(),
        res,
    )
}

/// A Python module implemented in Rust.
#[pymodule]
fn zen_core_py(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<NewBar>()?;
    m.add_class::<BI>()?;
    m.add_function(wrap_pyfunction!(check_bi_py, m)?)?;
    Ok(())
}
