use pyo3::prelude::*;

use crate::alpha_mul_div::RustAlphaMulDiv;
use crate::image_view::ImageView;
use crate::pil_image_view::PilImageView;
use crate::resizer::RustResizer;

#[macro_use]
mod utils;

mod alpha_mul_div;
mod image_view;
mod pil_image_view;
mod resizer;

/// This module is a python module implemented in Rust.
#[pymodule]
fn rust_lib(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<ImageView>()?;
    m.add_class::<PilImageView>()?;
    m.add_class::<RustResizer>()?;
    m.add_class::<RustAlphaMulDiv>()?;

    Ok(())
}
