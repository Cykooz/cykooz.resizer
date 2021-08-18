use std::num::NonZeroU32;
use std::slice;

use fast_image_resize::{CropBox, DstImageView, PixelType, SrcImageView};
use pyo3::prelude::*;
use pyo3::{PyGCProtocol, PyTraverseError, PyVisit};

use crate::utils::result2pyresult;

#[pyclass(gc)]
pub struct PilImageView {
    pil_image: Option<PyObject>,
    pixel_type: PixelType,
    width: NonZeroU32,
    height: NonZeroU32,
    rows_ptr: Option<u64>,
    crop_box: Option<CropBox>,
}

#[pyproto]
impl PyGCProtocol for PilImageView {
    fn __traverse__(&self, visit: PyVisit) -> Result<(), PyTraverseError> {
        if let Some(obj) = &self.pil_image {
            visit.call(obj)?
        }
        Ok(())
    }

    fn __clear__(&mut self) {
        // Clear reference, this decrements ref counter.
        self.pil_image = None;
        self.rows_ptr = None;
    }
}

#[pymethods]
impl PilImageView {
    #[new]
    fn new(py: Python, pil_image: PyObject) -> PyResult<Self> {
        let py_mode = pil_image.getattr(py, "mode")?;
        let mode: String = py_mode.extract(py)?;
        let pixel_type = match mode.as_str() {
            "RGB" | "RGBA" | "RGBa" | "CMYK" | "YCbCr" | "Lab" => PixelType::U8x4,
            "I" => PixelType::I32,
            "F" => PixelType::F32,
            _ => return result2pyresult(Err("Not supported mode of PIL image")),
        };

        let py_size = pil_image.getattr(py, "size")?;
        let (width, height): (u32, u32) = py_size.extract(py)?;
        let width =
            result2pyresult(NonZeroU32::new(width).ok_or("Image width must be greater than zero"))?;
        let height = result2pyresult(
            NonZeroU32::new(height).ok_or("Image height must be greater than zero"),
        )?;

        pil_image.call_method0(py, "load")?;
        let py_unsafe_ptrs = pil_image.getattr(py, "im")?.getattr(py, "unsafe_ptrs")?;
        let unsafe_ptrs: Vec<(String, u64)> = py_unsafe_ptrs.extract(py)?;
        let rows_ptr = result2pyresult(
            unsafe_ptrs
                .into_iter()
                .find(|(name, _)| name == "image32")
                .map(|(_, ptr)| ptr)
                .ok_or("Can't get pointer to image pixels"),
        )?;

        Ok(Self {
            pil_image: Some(pil_image),
            pixel_type,
            width,
            height,
            rows_ptr: Some(rows_ptr),
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

    #[getter]
    fn pil_image(&self) -> PyResult<&Option<PyObject>> {
        Ok(&self.pil_image)
    }
}

impl PilImageView {
    pub(crate) fn src_image_view(&self) -> PyResult<SrcImageView> {
        if let Some(rows_ptr) = self.rows_ptr {
            let rows_ptr = rows_ptr as *const *const u32;
            let width = self.width.get() as usize;
            let rows = (0..self.height.get() as usize)
                .map(|i| unsafe { slice::from_raw_parts(*rows_ptr.add(i), width) })
                .collect();

            let mut view = result2pyresult(SrcImageView::from_rows(
                self.width,
                self.height,
                rows,
                self.pixel_type,
            ))?;
            if let Some(crop_box) = self.crop_box {
                result2pyresult(view.set_crop_box(crop_box))?;
            }
            Ok(view)
        } else {
            result2pyresult(Err("PIL image is dropped"))
        }
    }

    pub(crate) fn dst_image_view(&mut self) -> PyResult<DstImageView> {
        if let Some(rows_ptr) = self.rows_ptr {
            let rows_ptr = rows_ptr as *const *mut u32;
            let width = self.width.get() as usize;
            let rows = (0..self.height.get() as usize)
                .map(|i| unsafe { slice::from_raw_parts_mut(*rows_ptr.add(i), width) })
                .collect();

            let view = result2pyresult(DstImageView::from_rows(
                self.width,
                self.height,
                rows,
                self.pixel_type,
            ))?;
            Ok(view)
        } else {
            result2pyresult(Err("PIL image is dropped"))
        }
    }
}
