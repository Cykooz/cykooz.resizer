use std::sync::{Arc, Mutex};

use fast_image_resize as fr;
use fast_image_resize::{ResizeOptions, SrcCropping};
use pyo3::prelude::*;

use crate::image_view::Image;
use crate::pil_image_wrapper::PilImageWrapper;
use crate::utils::{cpu_extensions_from_u8, cpu_extensions_to_u8, result2pyresult};

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
pub struct RustResizeOptions(ResizeOptions);

#[pymethods]
impl RustResizeOptions {
    #[new]
    fn new() -> Self {
        Self(ResizeOptions::new())
    }

    fn copy(&self) -> Self {
        Self(self.0)
    }

    /// get_algorithm() -> Tuple[int, int, int]
    /// --
    ///
    /// Returns resize algorithm.
    ///
    /// :rtype: Tuple[int, int, int]
    fn get_resize_alg(&self) -> (u8, u8, u8) {
        let (algorithm, filter_type, multiplicity) = match self.0.algorithm {
            fr::ResizeAlg::Nearest => (1u8, 0u8, 2u8),
            fr::ResizeAlg::Convolution(filter_type) => (2u8, filter_type_as_u8(filter_type), 2u8),
            fr::ResizeAlg::SuperSampling(filter_type, multiplicity) => {
                (3u8, filter_type_as_u8(filter_type), multiplicity)
            }
            _ => (0u8, 0u8, 2u8),
        };

        // let algorithm = algorithm.to_object(py);
        // let filter_type = filter_type.to_object(py);
        // let multiplicity = multiplicity.to_object(py);

        // let res: PyObject = PyTuple::new_bound(py, &[algorithm, filter_type, multiplicity]).into();
        (algorithm, filter_type, multiplicity)
    }

    /// Set resize algorithm.
    #[pyo3(signature = (algorithm, filter_type, multiplicity))]
    fn set_resize_alg(&mut self, algorithm: u8, filter_type: u8, multiplicity: u8) -> Self {
        let resizer_alg = match algorithm {
            1 => fr::ResizeAlg::Nearest,
            2 => fr::ResizeAlg::Convolution(filter_type_from_u8(filter_type)),
            3 => fr::ResizeAlg::SuperSampling(filter_type_from_u8(filter_type), multiplicity),
            _ => fr::ResizeAlg::Nearest,
        };
        Self(self.0.resize_alg(resizer_alg))
    }

    /// Set crop box for source image.
    fn get_crop_box(&self) -> Option<(f64, f64, f64, f64)> {
        match self.0.cropping {
            SrcCropping::Crop(crop_box) => {
                Some((crop_box.left, crop_box.top, crop_box.width, crop_box.height))
            }
            _ => None,
        }
    }

    /// Set crop box for source image.
    #[pyo3(signature = (left, top, width, height))]
    fn set_crop_box(&self, left: f64, top: f64, width: f64, height: f64) -> Self {
        Self(self.0.crop(left, top, width, height))
    }

    fn get_fit_into_destination_centering(&self) -> Option<(f64, f64)> {
        match self.0.cropping {
            SrcCropping::FitIntoDestination(centering) => Some(centering),
            _ => None,
        }
    }

    /// Fit source image into the aspect ratio of destination image without distortions.
    #[pyo3(signature = (centering=None))]
    fn set_fit_into_destination(&self, centering: Option<(f64, f64)>) -> Self {
        Self(self.0.fit_into_destination(centering))
    }

    fn get_use_alpha(&self) -> bool {
        self.0.mul_div_alpha
    }

    /// Enable or disable consideration of the alpha channel when resizing.
    #[pyo3(signature = (v))]
    fn set_use_alpha(&self, v: bool) -> Self {
        Self(self.0.use_alpha(v))
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
    fn get_cpu_extensions(&self, py: Python) -> PyResult<PyObject> {
        let resizer_mutex = self.resizer.clone();
        let resizer = result2pyresult(resizer_mutex.lock())?;
        let cpu_extensions = cpu_extensions_to_u8(resizer.cpu_extensions());
        Ok(cpu_extensions.into_py(py))
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
        py.allow_threads(move || {
            let fir_options = options.map(|o| &o.0);
            let src_image_view = src_image.src_image_view();
            let dst_image_view = dst_image.dst_image_view();
            let mut resizer = result2pyresult(resizer_mutex.lock())?;
            result2pyresult(resizer.resize(src_image_view, dst_image_view, fir_options))?;
            Ok(())
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
        py.allow_threads(move || {
            let fir_options = options.map(|o| &o.0);
            let mut resizer = result2pyresult(resizer_mutex.lock())?;
            result2pyresult(resizer.resize(src_image, dst_image, fir_options))?;
            Ok(())
        })
    }
}
