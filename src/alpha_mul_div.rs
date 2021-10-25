use std::sync::{Arc, Mutex};

use fast_image_resize as fir;
use pyo3::prelude::*;

use crate::image_view::ImageView;
use crate::pil_image_view::{PilImageView, RgbMode};
use crate::utils::{cpu_extensions_from_u8, cpu_extensions_to_u8, result2pyresult};

#[pyclass]
pub struct RustAlphaMulDiv {
    mul_div: Arc<Mutex<fir::MulDiv>>,
}

#[pymethods]
impl RustAlphaMulDiv {
    #[new]
    fn new() -> Self {
        Self {
            mul_div: Arc::new(Mutex::new(Default::default())),
        }
    }

    /// Returns CPU extensions.
    #[pyo3(text_signature = "($self)")]
    fn get_cpu_extensions(&self, py: Python) -> PyResult<PyObject> {
        let mul_div_mutex = self.mul_div.clone();
        let mul_div = result2pyresult(mul_div_mutex.lock())?;
        let cpu_extensions = cpu_extensions_to_u8(mul_div.cpu_extensions());
        Ok(cpu_extensions.into_py(py))
    }

    /// Set CPU extensions.
    #[pyo3(text_signature = "($self, extensions)")]
    fn set_cpu_extensions(&mut self, extensions: u8) -> PyResult<()> {
        let cpu_extensions = cpu_extensions_from_u8(extensions);
        let mul_div_mutex = self.mul_div.clone();
        let mut mul_div = result2pyresult(mul_div_mutex.lock())?;
        unsafe {
            mul_div.set_cpu_extensions(cpu_extensions);
        }
        Ok(())
    }

    /// Multiplies RGB-channels of source image by alpha-channel and store
    /// result into destination image.
    /// The images are represented as instances of ImageView.
    #[pyo3(text_signature = "($self, src_image, dst_image)")]
    fn multiply_alpha(
        &self,
        py: Python,
        src_image: &ImageView,
        dst_image: &mut ImageView,
    ) -> PyResult<()> {
        let mul_div_mutex = self.mul_div.clone();
        py.allow_threads(move || {
            let src_image_view = src_image.src_image_view()?;
            let mut dst_image_view = dst_image.dst_image_view();
            let mul_div = result2pyresult(mul_div_mutex.lock())?;
            result2pyresult(mul_div.multiply_alpha(&src_image_view, &mut dst_image_view))
        })
    }

    /// Multiplies RGB-channels of image by alpha-channel inplace.
    /// The image is represented as instances of ImageView.
    #[pyo3(text_signature = "($self, image)")]
    fn multiply_alpha_inplace(&self, py: Python, image: &mut ImageView) -> PyResult<()> {
        let mul_div_mutex = self.mul_div.clone();
        py.allow_threads(move || {
            let mut dst_image_view = image.dst_image_view();
            let mul_div = result2pyresult(mul_div_mutex.lock())?;
            result2pyresult(mul_div.multiply_alpha_inplace(&mut dst_image_view))
        })
    }

    /// Multiplies RGB-channels of source image by alpha-channel and store
    /// result into destination image.
    /// The images are represented as instances of PilImageView.
    #[pyo3(text_signature = "($self, src_image, dst_image)")]
    fn multiply_alpha_pil(
        &self,
        py: Python,
        src_image: &PilImageView,
        dst_image: &mut PilImageView,
    ) -> PyResult<()> {
        if !src_image.is_rgb_mode(py)? {
            return result2pyresult(Err("Invalid mode of source PIL image"));
        }
        if !dst_image.is_rgb_mode(py)? {
            return result2pyresult(Err("Invalid mode of destination PIL image"));
        }
        let mul_div_mutex = self.mul_div.clone();
        py.allow_threads(move || {
            let src_image_view = src_image.src_image_view()?;
            let mut dst_image_view = dst_image.dst_image_view()?;
            let mul_div = result2pyresult(mul_div_mutex.lock())?;
            result2pyresult(mul_div.multiply_alpha(&src_image_view, &mut dst_image_view))
        })
    }

    /// Multiplies RGB-channels of image by alpha-channel inplace.
    /// The image is represented as instance of PilImageView.
    #[pyo3(text_signature = "($self, image)")]
    fn multiply_alpha_pil_inplace(&self, py: Python, image: &mut PilImageView) -> PyResult<()> {
        if !image.is_rgb_mode(py)? {
            return result2pyresult(Err("Invalid mode of PIL image"));
        }
        let mul_div_mutex = self.mul_div.clone();
        py.allow_threads(|| {
            let mut dst_image_view = image.dst_image_view()?;
            let mul_div = result2pyresult(mul_div_mutex.lock())?;
            result2pyresult(mul_div.multiply_alpha_inplace(&mut dst_image_view))
        })?;
        image.set_rgb_mode(py, RgbMode::Rgba)
    }

    /// Divides RGB-channels of source image by alpha-channel and store
    /// result into destination image.
    /// The images are represented as instances of ImageView.
    #[pyo3(text_signature = "($self, src_image, dst_image)")]
    fn divide_alpha(
        &self,
        py: Python,
        src_image: &ImageView,
        dst_image: &mut ImageView,
    ) -> PyResult<()> {
        let mul_div_mutex = self.mul_div.clone();
        py.allow_threads(move || {
            let src_image_view = src_image.src_image_view()?;
            let mut dst_image_view = dst_image.dst_image_view();
            let mul_div = result2pyresult(mul_div_mutex.lock())?;
            result2pyresult(mul_div.divide_alpha(&src_image_view, &mut dst_image_view))
        })
    }

    /// Divides RGB-channels of image by alpha-channel inplace.
    /// The image is represented as instances of ImageView.
    #[pyo3(text_signature = "($self, image)")]
    fn divide_alpha_inplace(&self, py: Python, image: &mut ImageView) -> PyResult<()> {
        let mul_div_mutex = self.mul_div.clone();
        py.allow_threads(move || {
            let mut dst_image_view = image.dst_image_view();
            let mul_div = result2pyresult(mul_div_mutex.lock())?;
            result2pyresult(mul_div.divide_alpha_inplace(&mut dst_image_view))
        })
    }

    /// Divides RGB-channels of source image by alpha-channel and store
    /// result into destination image.
    /// The images are represented as instances of PilImageView.
    #[pyo3(text_signature = "($self, src_image, dst_image)")]
    fn divide_alpha_pil(
        &self,
        py: Python,
        src_image: &PilImageView,
        dst_image: &mut PilImageView,
    ) -> PyResult<()> {
        if !src_image.is_rgb_mode(py)? {
            return result2pyresult(Err("Invalid mode of source PIL image"));
        }
        if !dst_image.is_rgb_mode(py)? {
            return result2pyresult(Err("Invalid mode of destination PIL image"));
        }
        let mul_div_mutex = self.mul_div.clone();
        py.allow_threads(move || {
            let src_image_view = src_image.src_image_view()?;
            let mut dst_image_view = dst_image.dst_image_view()?;
            let mul_div = result2pyresult(mul_div_mutex.lock())?;
            result2pyresult(mul_div.divide_alpha(&src_image_view, &mut dst_image_view))
        })
    }

    /// Divides RGB-channels of image by alpha-channel inplace.
    /// The image is represented as instance of PilImageView.
    #[pyo3(text_signature = "($self, image)")]
    fn divide_alpha_pil_inplace(&self, py: Python, image: &mut PilImageView) -> PyResult<()> {
        if !image.is_rgb_mode(py)? {
            return result2pyresult(Err("Invalid mode of PIL image"));
        }
        let mul_div_mutex = self.mul_div.clone();
        py.allow_threads(|| {
            let mut dst_image_view = image.dst_image_view()?;
            let mul_div = result2pyresult(mul_div_mutex.lock())?;
            result2pyresult(mul_div.divide_alpha_inplace(&mut dst_image_view))
        })?;
        image.set_rgb_mode(py, RgbMode::RgbA)
    }
}
