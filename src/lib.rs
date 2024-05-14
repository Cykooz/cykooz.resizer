use pyo3::prelude::*;

use crate::alpha_mul_div::RustAlphaMulDiv;
use crate::image_view::Image;
use crate::pil_image_wrapper::PilImageWrapper;
use crate::resizer::{RustResizeOptions, RustResizer};

#[macro_use]
mod utils;

mod alpha_mul_div;
mod image_view;
mod pil_image_wrapper;
mod resizer;

/// This module is a python module implemented in Rust.
#[pymodule]
fn rust_lib(_py: Python, m: &Bound<PyModule>) -> PyResult<()> {
    m.add_class::<Image>()?;
    m.add_class::<PilImageWrapper>()?;
    m.add_class::<RustResizeOptions>()?;
    m.add_class::<RustResizer>()?;
    m.add_class::<RustAlphaMulDiv>()?;

    Ok(())
}
