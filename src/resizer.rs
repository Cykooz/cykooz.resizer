use std::sync::{Arc, Mutex};

use fast_image_resize as fr;
use pyo3::prelude::*;
use pyo3::types::PyTuple;

use crate::image_view::ImageView;
use crate::pil_image_view::PilImageView;
use crate::utils::{cpu_extensions_from_u8, cpu_extensions_to_u8, result2pyresult};

fn filter_type_from_u8(filter: u8) -> fr::FilterType {
    match filter {
        1 => fr::FilterType::Box,
        2 => fr::FilterType::Bilinear,
        3 => fr::FilterType::CatmullRom,
        4 => fr::FilterType::Mitchell,
        5 => fr::FilterType::Lanczos3,
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
        _ => 0,
    }
}

#[pyclass]
pub struct RustResizer {
    resizer: Arc<Mutex<fr::Resizer>>,
}

#[pymethods]
impl RustResizer {
    #[new]
    fn new(algorithm: u8, filter_type: u8, multiplicity: u8) -> PyResult<Self> {
        let mut resizer = Self {
            resizer: Arc::new(Mutex::new(fr::Resizer::new(fr::ResizeAlg::Nearest))),
        };
        resizer.set_algorithm(algorithm, filter_type, multiplicity)?;
        Ok(resizer)
    }

    /// get_algorithm() -> Tuple[int, int, int]
    /// --
    ///
    /// Returns resize algorithm.
    ///
    /// :rtype: Tuple[int, int, int]
    fn get_algorithm(&self, py: Python) -> PyResult<PyObject> {
        let resizer_mutex = self.resizer.clone();
        let resizer = result2pyresult(resizer_mutex.lock())?;

        let (algorithm, filter_type, multiplicity) = match resizer.algorithm {
            fr::ResizeAlg::Nearest => (1u8, 0u8, 2u8),
            fr::ResizeAlg::Convolution(filter_type) => (2u8, filter_type_as_u8(filter_type), 2u8),
            fr::ResizeAlg::SuperSampling(filter_type, multiplicity) => {
                (3u8, filter_type_as_u8(filter_type), multiplicity)
            }
            _ => (0u8, 0u8, 2u8),
        };

        let algorithm = algorithm.to_object(py);
        let filter_type = filter_type.to_object(py);
        let multiplicity = multiplicity.to_object(py);

        let res: PyObject = PyTuple::new(py, &[algorithm, filter_type, multiplicity]).into();
        Ok(res)
    }

    /// Set resize algorithm.
    #[pyo3(text_signature = "($self, algorithm, filter_type, multiplicity)")]
    fn set_algorithm(&mut self, algorithm: u8, filter_type: u8, multiplicity: u8) -> PyResult<()> {
        let resizer_alg = match algorithm {
            1 => fr::ResizeAlg::Nearest,
            2 => fr::ResizeAlg::Convolution(filter_type_from_u8(filter_type)),
            3 => fr::ResizeAlg::SuperSampling(filter_type_from_u8(filter_type), multiplicity),
            _ => fr::ResizeAlg::Nearest,
        };
        let resizer_mutex = self.resizer.clone();
        let mut resizer = result2pyresult(resizer_mutex.lock())?;
        resizer.algorithm = resizer_alg;
        Ok(())
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
    #[pyo3(text_signature = "($self, src_image, dst_image)")]
    fn resize(&self, py: Python, src_image: &ImageView, dst_image: &mut ImageView) -> PyResult<()> {
        let resizer_mutex = self.resizer.clone();
        py.allow_threads(move || {
            let src_image_view = src_image.src_image_view()?;
            let mut dst_image_view = dst_image.dst_image_view();
            let mut resizer = result2pyresult(resizer_mutex.lock())?;
            result2pyresult(resizer.resize(&src_image_view, &mut dst_image_view))?;
            Ok(())
        })
    }

    /// Resize source image into destination image.
    #[pyo3(text_signature = "($self, src_image, dst_image)")]
    fn resize_pil(
        &self,
        py: Python,
        src_image: &PilImageView,
        dst_image: &mut PilImageView,
    ) -> PyResult<()> {
        let resizer_mutex = self.resizer.clone();
        py.allow_threads(move || {
            let src_image_view = src_image.src_image_view()?;
            let mut dst_image_view = dst_image.dst_image_view()?;
            let mut resizer = result2pyresult(resizer_mutex.lock())?;
            result2pyresult(resizer.resize(&src_image_view, &mut dst_image_view))?;
            Ok(())
        })
    }
}
