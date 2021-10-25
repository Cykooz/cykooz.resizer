use fast_image_resize as fir;
use fast_image_resize::PixelType;
use pyo3::prelude::*;
use pyo3::types::PyBytes;

use crate::utils::{pixel_type_from_u8, result2pyresult};

#[pyclass]
pub struct ImageView {
    image: fir::Image<'static>,
    crop_box: Option<fir::CropBox>,
}

#[pymethods]
impl ImageView {
    #[new]
    fn new(width: u32, height: u32, pixel_type: u8, buffer: Option<&[u8]>) -> PyResult<Self> {
        let width = into_non_zero!(width)?;
        let height = into_non_zero!(height)?;
        let pixel_type = pixel_type_from_u8(pixel_type);
        let pixel_size = match pixel_type {
            PixelType::U8 => 1,
            _ => 4,
        };
        let image = if let Some(buffer) = buffer {
            let buffer_size = (width.get() * height.get()) as usize * pixel_size;
            if buffer.len() < buffer_size {
                return result2pyresult(Err(format!(
                    "Size of 'buffer' must be greater or equal to {} bytes",
                    buffer_size
                )));
            }
            result2pyresult(fir::Image::from_vec_u8(
                width,
                height,
                buffer.to_vec(),
                pixel_type,
            ))?
        } else {
            fir::Image::new(width, height, pixel_type)
        };

        Ok(Self {
            image,
            crop_box: None,
        })
    }

    fn set_crop_box(&mut self, left: u32, top: u32, width: u32, height: u32) -> PyResult<()> {
        self.crop_box = Some(fir::CropBox {
            left,
            top,
            width: into_non_zero!(width)?,
            height: into_non_zero!(height)?,
        });
        Ok(())
    }

    fn width(&self) -> u32 {
        self.image.width().get()
    }

    fn height(&self) -> u32 {
        self.image.height().get()
    }

    fn buffer(&self, py: Python) -> PyResult<PyObject> {
        let image_buffer = self.image.buffer();
        PyBytes::new_with(py, image_buffer.len(), |dst_buffer| {
            dst_buffer.copy_from_slice(image_buffer);
            Ok(())
        })
        .map(|bytes| bytes.to_object(py))
    }
}

impl ImageView {
    pub(crate) fn src_image_view(&self) -> PyResult<fir::ImageView> {
        let mut src_image_view = self.image.view();
        if let Some(crop_box) = self.crop_box {
            result2pyresult(src_image_view.set_crop_box(crop_box))?;
        }
        Ok(src_image_view)
    }

    pub(crate) fn dst_image_view(&mut self) -> fir::ImageViewMut {
        self.image.view_mut()
    }
}
