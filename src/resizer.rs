use std::ops::DerefMut;
use std::sync::{Arc, Mutex};

use crate::image_view::Image;
use crate::pil_image_wrapper::PilImageWrapper;
use crate::thread_pool::ResizerThreadPool;
use crate::utils::{cpu_extensions_from_u8, cpu_extensions_to_u8, result2pyresult};
use fast_image_resize as fr;
use pyo3::prelude::*;
use pyo3::types::PyInt;

fn filter_type_from_u8(filter: u8) -> fr::FilterType {
    match filter {
        1 => fr::FilterType::Box,
        2 => fr::FilterType::Bilinear,
        3 => fr::FilterType::CatmullRom,
        4 => fr::FilterType::Mitchell,
        5 => fr::FilterType::Lanczos3,
        6 => fr::FilterType::Gaussian,
        _ => fr::FilterType::Box,
    }
}

fn filter_type_as_u8(filter_type: fr::FilterType) -> u8 {
    match filter_type {
        fr::FilterType::Box => 1,
        fr::FilterType::Bilinear => 2,
        fr::FilterType::CatmullRom => 3,
        fr::FilterType::Mitchell => 4,
        fr::FilterType::Lanczos3 => 5,
        fr::FilterType::Gaussian => 6,
        _ => 0,
    }
}

#[pyclass]
pub struct RustResizeOptions {
    fir_options: fr::ResizeOptions,
    thread_pool: Option<ResizerThreadPool>,
}

#[pymethods]
impl RustResizeOptions {
    #[new]
    fn new() -> Self {
        Self {
            fir_options: fr::ResizeOptions::new(),
            thread_pool: None,
        }
    }

    fn copy(&self) -> Self {
        Self {
            fir_options: self.fir_options,
            thread_pool: self.thread_pool.clone(),
        }
    }

    /// get_algorithm() -> Tuple[int, int, int]
    /// --
    ///
    /// Returns resize algorithm.
    ///
    /// :rtype: Tuple[int, int, int]
    fn get_resize_alg(&self) -> (u8, u8, u8) {
        let (algorithm, filter_type, multiplicity) = match self.fir_options.algorithm {
            fr::ResizeAlg::Nearest => (1u8, 0u8, 2u8),
            fr::ResizeAlg::Convolution(filter_type) => (2u8, filter_type_as_u8(filter_type), 2u8),
            fr::ResizeAlg::Interpolation(filter_type) => (3u8, filter_type_as_u8(filter_type), 2u8),
            fr::ResizeAlg::SuperSampling(filter_type, multiplicity) => {
                (4u8, filter_type_as_u8(filter_type), multiplicity)
            }
            _ => (0u8, 0u8, 2u8),
        };
        (algorithm, filter_type, multiplicity)
    }

    /// Set resize algorithm.
    #[pyo3(signature = (algorithm, filter_type, multiplicity))]
    fn set_resize_alg(&mut self, algorithm: u8, filter_type: u8, multiplicity: u8) -> Self {
        let resizer_alg = match algorithm {
            1 => fr::ResizeAlg::Nearest,
            2 => fr::ResizeAlg::Convolution(filter_type_from_u8(filter_type)),
            3 => fr::ResizeAlg::Interpolation(filter_type_from_u8(filter_type)),
            4 => fr::ResizeAlg::SuperSampling(filter_type_from_u8(filter_type), multiplicity),
            _ => fr::ResizeAlg::Nearest,
        };
        Self {
            fir_options: self.fir_options.resize_alg(resizer_alg),
            thread_pool: self.thread_pool.clone(),
        }
    }

    /// Set crop box for source image.
    fn get_crop_box(&self) -> Option<(f64, f64, f64, f64)> {
        match self.fir_options.cropping {
            fr::SrcCropping::Crop(crop_box) => {
                Some((crop_box.left, crop_box.top, crop_box.width, crop_box.height))
            }
            _ => None,
        }
    }

    /// Set crop box for source image.
    #[pyo3(signature = (left, top, width, height))]
    fn set_crop_box(&self, left: f64, top: f64, width: f64, height: f64) -> Self {
        Self {
            fir_options: self.fir_options.crop(left, top, width, height),
            thread_pool: self.thread_pool.clone(),
        }
    }

    fn get_fit_into_destination_centering(&self) -> Option<(f64, f64)> {
        match self.fir_options.cropping {
            fr::SrcCropping::FitIntoDestination(centering) => Some(centering),
            _ => None,
        }
    }

    /// Fit the source image into the aspect ratio of the destination image without distortions.
    #[pyo3(signature = (centering=None))]
    fn set_fit_into_destination(&self, centering: Option<(f64, f64)>) -> Self {
        Self {
            fir_options: self.fir_options.fit_into_destination(centering),
            thread_pool: self.thread_pool.clone(),
        }
    }

    fn get_use_alpha(&self) -> bool {
        self.fir_options.mul_div_alpha
    }

    /// Enable or disable consideration of the alpha channel when resizing.
    #[pyo3(signature = (v))]
    fn set_use_alpha(&self, v: bool) -> Self {
        Self {
            fir_options: self.fir_options.use_alpha(v),
            thread_pool: self.thread_pool.clone(),
        }
    }

    /// get_thread_pool() -> Option<ThreadPool>
    /// --
    ///
    /// Returns thread pool.
    ///
    /// :rtype: Option<ThreadPool>
    fn get_thread_pool(&self) -> Option<ResizerThreadPool> {
        self.thread_pool.clone()
    }

    /// Set a thread pool.
    #[pyo3(signature = (thread_pool))]
    fn set_thread_pool(&mut self, thread_pool: Option<ResizerThreadPool>) -> Self {
        Self {
            fir_options: self.fir_options,
            thread_pool,
        }
    }
}

#[pyclass]
pub struct RustResizer {
    resizer: Arc<Mutex<fr::Resizer>>,
}

#[pymethods]
impl RustResizer {
    #[new]
    fn new() -> Self {
        Self {
            resizer: Arc::new(Mutex::new(fr::Resizer::new())),
        }
    }

    /// Returns CPU extensions.
    #[pyo3(text_signature = "($self)")]
    fn get_cpu_extensions<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyInt>> {
        let resizer_mutex = self.resizer.clone();
        let resizer = result2pyresult(resizer_mutex.lock())?;
        let cpu_extensions = cpu_extensions_to_u8(resizer.cpu_extensions());
        Ok(cpu_extensions.into_pyobject(py)?)
    }

    /// Set CPU extensions.
    #[pyo3(text_signature = "($self, extensions)")]
    fn set_cpu_extensions(&mut self, extensions: u8) -> PyResult<()> {
        let cpu_extensions = cpu_extensions_from_u8(extensions);
        let resizer_mutex = self.resizer.clone();
        let mut resizer = result2pyresult(resizer_mutex.lock())?;
        unsafe {
            resizer.set_cpu_extensions(cpu_extensions);
        }
        Ok(())
    }

    /// Resize source image into destination image.
    #[pyo3(signature = (src_image, dst_image, options=None))]
    fn resize(
        &self,
        py: Python,
        src_image: &Image,
        dst_image: &mut Image,
        options: Option<&RustResizeOptions>,
    ) -> PyResult<()> {
        let resizer_mutex = self.resizer.clone();
        py.detach(move || {
            let fir_options = options.map(|o| &o.fir_options);
            let src_image_view = src_image.src_image_view();
            let dst_image_view = dst_image.dst_image_view();
            let mut resizer_guard = result2pyresult(resizer_mutex.lock())?;
            let resizer = resizer_guard.deref_mut();
            if let Some(thread_pool) = options.and_then(|o| o.thread_pool.as_ref()) {
                if fir_options
                    // Don't process in thread-pool if resize alg is nearest
                    .map(|o| o.algorithm != fr::ResizeAlg::Nearest)
                    .unwrap_or(true)
                {
                    return thread_pool.run_within(|| {
                        result2pyresult(resizer.resize(src_image_view, dst_image_view, fir_options))
                    });
                }
            }
            result2pyresult(resizer.resize(src_image_view, dst_image_view, fir_options))
        })
    }

    /// Resize source image into destination image.
    #[pyo3(signature = (src_image, dst_image, options=None))]
    fn resize_pil(
        &self,
        py: Python,
        src_image: &PilImageWrapper,
        dst_image: &mut PilImageWrapper,
        options: Option<&RustResizeOptions>,
    ) -> PyResult<()> {
        let resizer_mutex = self.resizer.clone();
        py.detach(move || {
            let fir_options = options.map(|o| &o.fir_options);
            let mut resizer_guard = result2pyresult(resizer_mutex.lock())?;
            let resizer = resizer_guard.deref_mut();
            if let Some(thread_pool) = options.and_then(|o| o.thread_pool.as_ref()) {
                if fir_options
                    // Don't process in thread-pool if resize alg is nearest
                    .map(|o| o.algorithm != fr::ResizeAlg::Nearest)
                    .unwrap_or(true)
                {
                    return thread_pool.run_within(|| {
                        result2pyresult(resizer.resize(src_image, dst_image, fir_options))
                    });
                }
            }
            result2pyresult(resizer.resize(src_image, dst_image, fir_options))
        })
    }
}
