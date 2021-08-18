use std::num::NonZeroU32;

use fast_image_resize::{CropBox, DstImageView, PixelType, SrcImageView};
use pyo3::prelude::*;

use crate::utils::{pixel_type_from_u8, result2pyresult};
use pyo3::types::PyBytes;

#[pyclass]
pub struct ImageView {
    pixels: Vec<u32>,
    pixel_type: PixelType,
    width: NonZeroU32,
    height: NonZeroU32,
    crop_box: Option<CropBox>,
}

#[pymethods]
impl ImageView {
    #[new]
    fn new(width: u32, height: u32, pixel_type: u8, buffer: Option<&[u8]>) -> PyResult<Self> {
        let width = into_non_zero!(width)?;
        let height = into_non_zero!(height)?;
        let pixels = if let Some(buffer) = buffer {
            let buffer_size = (width.get() * height.get()) as usize * 4;
            if buffer.len() < buffer_size {
                return result2pyresult(Err(format!(
                    "Size of 'buffer' must be greater or equal to {} bytes",
                    buffer_size
                )));
            }
            buffer
                .chunks_exact(4)
                .map(|p| u32::from_le_bytes([p[0], p[1], p[2], p[3]]))
                .collect()
        } else {
            let pixels_size = (width.get() * height.get()) as usize;
            vec![0; pixels_size]
        };

        Ok(Self {
            pixels,
            pixel_type: pixel_type_from_u8(pixel_type),
            width,
            height,
            crop_box: None,
        })
    }

    fn set_crop_box(&mut self, left: u32, top: u32, width: u32, height: u32) -> PyResult<()> {
        self.crop_box = Some(CropBox {
            left,
            top,
            width: into_non_zero!(width)?,
            height: into_non_zero!(height)?,
        });
        Ok(())
    }

    fn width(&self) -> u32 {
        self.width.get()
    }

    fn height(&self) -> u32 {
        self.height.get()
    }

    fn buffer(&self, py: Python) -> PyResult<PyObject> {
        let res_buffer_size = (self.width.get() * self.height.get() * 4) as usize;
        PyBytes::new_with(py, res_buffer_size, |dst_pixels| {
            let (_, src_buffer, _) = unsafe { &self.pixels.align_to::<u8>() };
            dst_pixels.copy_from_slice(src_buffer);
            Ok(())
        })
        .map(|bytes| bytes.to_object(py))
    }
}

impl ImageView {
    pub(crate) fn src_image_view(&self) -> PyResult<SrcImageView> {
        let mut src_image_view = result2pyresult(SrcImageView::from_pixels(
            self.width,
            self.height,
            &self.pixels,
            self.pixel_type,
        ))?;
        if let Some(crop_box) = self.crop_box {
            result2pyresult(src_image_view.set_crop_box(crop_box))?;
        }
        Ok(src_image_view)
    }

    pub(crate) fn dst_image_view(&mut self) -> PyResult<DstImageView> {
        let rows = self
            .pixels
            .chunks_exact_mut(self.width.get() as usize)
            .collect();
        result2pyresult(DstImageView::from_rows(
            self.width,
            self.height,
            rows,
            self.pixel_type,
        ))
    }
}
