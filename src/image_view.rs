use fast_image_resize::images::Image as FirImage;
use pyo3::prelude::*;
use pyo3::types::PyBytes;

use crate::utils::{pixel_type_from_u8, result2pyresult};

#[pyclass]
pub struct Image(FirImage<'static>);

#[pymethods]
impl Image {
    #[new]
    fn new(width: u32, height: u32, pixel_type: u8, buffer: Option<&[u8]>) -> PyResult<Self> {
        let pixel_type = pixel_type_from_u8(pixel_type);
        let pixel_size = pixel_type.size();
        let image = if let Some(buffer) = buffer {
            let buffer_size = (width * height) as usize * pixel_size;
            if buffer.len() < buffer_size {
                return result2pyresult(Err(format!(
                    "Size of 'buffer' must be greater or equal to {} bytes",
                    buffer_size
                )));
            }
            result2pyresult(FirImage::from_vec_u8(
                width,
                height,
                buffer.to_vec(),
                pixel_type,
            ))?
        } else {
            FirImage::new(width, height, pixel_type)
        };

        Ok(Self(image))
    }

    fn width(&self) -> u32 {
        self.0.width()
    }

    fn height(&self) -> u32 {
        self.0.height()
    }

    fn buffer(&self, py: Python) -> PyResult<PyObject> {
        let image_buffer = self.0.buffer();
        PyBytes::new_bound_with(py, image_buffer.len(), |dst_buffer| {
            dst_buffer.copy_from_slice(image_buffer);
            Ok(())
        })
        .map(|bytes| bytes.to_object(py))
    }
}

impl Image {
    pub(crate) fn src_image_view(&self) -> &FirImage<'static> {
        &self.0
    }

    pub(crate) fn dst_image_view(&mut self) -> &mut FirImage<'static> {
        &mut self.0
    }
}
