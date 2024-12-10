use std::ops::Deref;
use std::sync::{Arc, Mutex};

use crate::image_view::Image;
use crate::pil_image_wrapper::{PilImageWrapper, RgbMode};
use crate::thread_pool::ResizerThreadPool;
use crate::utils::{cpu_extensions_from_u8, cpu_extensions_to_u8, result2pyresult};
use fast_image_resize as fir;
use pyo3::prelude::*;
use pyo3::types::PyInt;

#[pyclass]
pub struct RustAlphaMulDiv {
    mul_div: Arc<Mutex<fir::MulDiv>>,
}

#[pymethods]
impl RustAlphaMulDiv {
    #[new]
    fn new() -> Self {
        Self {
            mul_div: Arc::new(Mutex::new(fir::MulDiv::new())),
        }
    }

    /// Returns CPU extensions.
    #[pyo3(signature = ())]
    fn get_cpu_extensions<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyInt>> {
        let mul_div_mutex = self.mul_div.clone();
        let mul_div = result2pyresult(mul_div_mutex.lock())?;
        let cpu_extensions = cpu_extensions_to_u8(mul_div.cpu_extensions());
        Ok(cpu_extensions.into_pyobject(py)?)
    }

    /// Set CPU extensions.
    #[pyo3(signature = (extensions))]
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
    #[pyo3(signature = (src_image, dst_image, thread_pool=None))]
    fn multiply_alpha(
        &self,
        py: Python,
        src_image: &Image,
        dst_image: &mut Image,
        thread_pool: Option<ResizerThreadPool>,
    ) -> PyResult<()> {
        let mul_div_mutex = self.mul_div.clone();
        py.allow_threads(move || {
            let src_image_view = src_image.src_image_view();
            let dst_image_view = dst_image.dst_image_view();
            let mul_div_guard = result2pyresult(mul_div_mutex.lock())?;
            let mul_div = mul_div_guard.deref();
            if let Some(thread_pool) = thread_pool {
                return thread_pool.run_within(|| {
                    result2pyresult(mul_div.multiply_alpha(src_image_view, dst_image_view))
                });
            }
            result2pyresult(mul_div.multiply_alpha(src_image_view, dst_image_view))
        })
    }

    /// Multiplies RGB-channels of image by alpha-channel inplace.
    /// The image is represented as instances of ImageView.
    #[pyo3(signature = (image, thread_pool=None))]
    fn multiply_alpha_inplace(
        &self,
        py: Python,
        image: &mut Image,
        thread_pool: Option<ResizerThreadPool>,
    ) -> PyResult<()> {
        let mul_div_mutex = self.mul_div.clone();
        py.allow_threads(move || {
            let dst_image_view = image.dst_image_view();
            let mul_div_guard = result2pyresult(mul_div_mutex.lock())?;
            let mul_div = mul_div_guard.deref();
            if let Some(thread_pool) = thread_pool {
                return thread_pool.run_within(|| {
                    result2pyresult(mul_div.multiply_alpha_inplace(dst_image_view))
                });
            }
            result2pyresult(mul_div.multiply_alpha_inplace(dst_image_view))
        })
    }

    /// Multiplies RGB-channels of source image by alpha-channel and store
    /// result into destination image.
    /// The images are represented as instances of PilImageView.
    #[pyo3(signature = (src_image, dst_image, thread_pool=None))]
    fn multiply_alpha_pil(
        &self,
        py: Python,
        src_image: &PilImageWrapper,
        dst_image: &mut PilImageWrapper,
        thread_pool: Option<ResizerThreadPool>,
    ) -> PyResult<()> {
        if !src_image.is_rgb_mode(py)? {
            return result2pyresult(Err("Invalid mode of source PIL image"));
        }
        if !dst_image.is_rgb_mode(py)? {
            return result2pyresult(Err("Invalid mode of destination PIL image"));
        }
        let mul_div_mutex = self.mul_div.clone();
        py.allow_threads(move || {
            let mul_div_guard = result2pyresult(mul_div_mutex.lock())?;
            let mul_div = mul_div_guard.deref();
            if let Some(thread_pool) = thread_pool {
                return thread_pool
                    .run_within(|| result2pyresult(mul_div.multiply_alpha(src_image, dst_image)));
            }
            result2pyresult(mul_div.multiply_alpha(src_image, dst_image))
        })
    }

    /// Multiplies RGB-channels of image by alpha-channel inplace.
    /// The image is represented as instance of PilImageView.
    #[pyo3(signature = (image, thread_pool=None))]
    fn multiply_alpha_pil_inplace(
        &self,
        py: Python,
        image: &mut PilImageWrapper,
        thread_pool: Option<ResizerThreadPool>,
    ) -> PyResult<()> {
        if !image.is_rgb_mode(py)? {
            return result2pyresult(Err("Invalid mode of PIL image"));
        }
        let mul_div_mutex = self.mul_div.clone();
        py.allow_threads(|| {
            let mul_div_guard = result2pyresult(mul_div_mutex.lock())?;
            let mul_div = mul_div_guard.deref();
            if let Some(thread_pool) = thread_pool {
                thread_pool.run_within(|| result2pyresult(mul_div.multiply_alpha_inplace(image)))
            } else {
                result2pyresult(mul_div.multiply_alpha_inplace(image))
            }
        })?;
        image.set_rgb_mode(py, RgbMode::Rgba)
    }

    /// Divides RGB-channels of source image by alpha-channel and store
    /// result into destination image.
    /// The images are represented as instances of ImageView.
    #[pyo3(signature = (src_image, dst_image, thread_pool=None))]
    fn divide_alpha(
        &self,
        py: Python,
        src_image: &Image,
        dst_image: &mut Image,
        thread_pool: Option<ResizerThreadPool>,
    ) -> PyResult<()> {
        let mul_div_mutex = self.mul_div.clone();
        py.allow_threads(move || {
            let src_image_view = src_image.src_image_view();
            let dst_image_view = dst_image.dst_image_view();
            let mul_div_guard = result2pyresult(mul_div_mutex.lock())?;
            let mul_div = mul_div_guard.deref();
            if let Some(thread_pool) = thread_pool {
                return thread_pool.run_within(|| {
                    result2pyresult(mul_div.divide_alpha(src_image_view, dst_image_view))
                });
            }
            result2pyresult(mul_div.divide_alpha(src_image_view, dst_image_view))
        })
    }

    /// Divides RGB-channels of image by alpha-channel inplace.
    /// The image is represented as instances of ImageView.
    #[pyo3(signature = (image, thread_pool=None))]
    fn divide_alpha_inplace(
        &self,
        py: Python,
        image: &mut Image,
        thread_pool: Option<ResizerThreadPool>,
    ) -> PyResult<()> {
        let mul_div_mutex = self.mul_div.clone();
        py.allow_threads(move || {
            let dst_image_view = image.dst_image_view();
            let mul_div_guard = result2pyresult(mul_div_mutex.lock())?;
            let mul_div = mul_div_guard.deref();
            if let Some(thread_pool) = thread_pool {
                return thread_pool
                    .run_within(|| result2pyresult(mul_div.divide_alpha_inplace(dst_image_view)));
            }
            result2pyresult(mul_div.divide_alpha_inplace(dst_image_view))
        })
    }

    /// Divides RGB-channels of source image by alpha-channel and store
    /// result into destination image.
    /// The images are represented as instances of PilImageView.
    #[pyo3(signature = (src_image, dst_image, thread_pool=None))]
    fn divide_alpha_pil(
        &self,
        py: Python,
        src_image: &PilImageWrapper,
        dst_image: &mut PilImageWrapper,
        thread_pool: Option<ResizerThreadPool>,
    ) -> PyResult<()> {
        if !src_image.is_rgb_mode(py)? {
            return result2pyresult(Err("Invalid mode of source PIL image"));
        }
        if !dst_image.is_rgb_mode(py)? {
            return result2pyresult(Err("Invalid mode of destination PIL image"));
        }
        let mul_div_mutex = self.mul_div.clone();
        py.allow_threads(move || {
            let mul_div_guard = result2pyresult(mul_div_mutex.lock())?;
            let mul_div = mul_div_guard.deref();
            if let Some(thread_pool) = thread_pool {
                return thread_pool
                    .run_within(|| result2pyresult(mul_div.divide_alpha(src_image, dst_image)));
            }
            result2pyresult(mul_div.divide_alpha(src_image, dst_image))
        })
    }

    /// Divides RGB-channels of image by alpha-channel inplace.
    /// The image is represented as instance of PilImageView.
    #[pyo3(signature = (image, thread_pool=None))]
    fn divide_alpha_pil_inplace(
        &self,
        py: Python,
        image: &mut PilImageWrapper,
        thread_pool: Option<ResizerThreadPool>,
    ) -> PyResult<()> {
        if !image.is_rgb_mode(py)? {
            return result2pyresult(Err("Invalid mode of PIL image"));
        }
        let mul_div_mutex = self.mul_div.clone();
        py.allow_threads(|| {
            let mul_div_guard = result2pyresult(mul_div_mutex.lock())?;
            let mul_div = mul_div_guard.deref();
            if let Some(thread_pool) = thread_pool {
                thread_pool.run_within(|| result2pyresult(mul_div.divide_alpha_inplace(image)))
            } else {
                result2pyresult(mul_div.divide_alpha_inplace(image))
            }
        })?;
        image.set_rgb_mode(py, RgbMode::RgbA)
    }
}
