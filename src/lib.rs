use crate::alpha_mul_div::RustAlphaMulDiv;
use crate::image_view::Image;
use crate::pil_image_wrapper::PilImageWrapper;
use crate::resizer::{RustResizeOptions, RustResizer};
use crate::thread_pool::ResizerThreadPool;
use pyo3::prelude::*;

#[macro_use]
mod utils;

mod alpha_mul_div;
mod image_view;
mod pil_image_wrapper;
mod resizer;
mod thread_pool;

/// This module is a python module implemented in Rust.
#[pymodule]
fn rust_lib(_py: Python, m: &Bound<PyModule>) -> PyResult<()> {
    // "Disable" global rayon's thread-pool
    rayon::ThreadPoolBuilder::new()
        .num_threads(1)
        .use_current_thread()
        .build_global()
        .unwrap();

    m.add_class::<ResizerThreadPool>()?;
    m.add_class::<Image>()?;
    m.add_class::<PilImageWrapper>()?;
    m.add_class::<RustResizeOptions>()?;
    m.add_class::<RustResizer>()?;
    m.add_class::<RustAlphaMulDiv>()?;

    Ok(())
}
