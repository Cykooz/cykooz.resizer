use std::num::NonZeroU32;
use std::slice;

use fast_image_resize::pixels::{PixelExt, PixelType};
use fast_image_resize::{CropBox, DynamicImageView, DynamicImageViewMut, ImageView, ImageViewMut};
use pyo3::ffi::PyCapsule_GetPointer;
use pyo3::prelude::*;
use pyo3::{PyTraverseError, PyVisit};

use crate::utils::result2pyresult;

// https://github.com/python-pillow/Pillow/blob/master/src/libImaging/Imaging.h#L80
#[repr(C)]
#[derive(Debug)]
struct ImagingMemoryInstance {
    mode: [u8; 7], /* Band names ("1", "L", "P", "RGB", "RGBA", "CMYK", "YCbCr", "BGR;xy") */
}

static IMAGING_MAGIC: &[u8] = b"PIL Imaging\0";

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) enum RgbMode {
    RgbA,
    Rgba,
}

#[pyclass]
pub struct PilImageView {
    pil_image: Option<PyObject>,
    pixel_type: PixelType,
    width: NonZeroU32,
    height: NonZeroU32,
    rows_ptr: Option<u64>,
    crop_box: Option<CropBox>,
}

#[pymethods]
impl PilImageView {
    #[new]
    fn new(py: Python, pil_image: PyObject) -> PyResult<Self> {
        let py_mode = pil_image.getattr(py, "mode")?;
        let mode: String = py_mode.extract(py)?;
        // PIL image data organization
        // https://github.com/python-pillow/Pillow/blob/master/src/libImaging/Imaging.h#L26
        let pixel_type = match mode.as_str() {
            "RGB" | "RGBA" | "RGBa" | "CMYK" | "YCbCr" | "Lab" => PixelType::U8x4,
            "I" => PixelType::I32,
            "F" => PixelType::F32,
            "L" => PixelType::U8,
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
        let im = pil_image.getattr(py, "im")?;
        let py_unsafe_ptrs = im.getattr(py, "unsafe_ptrs")?;
        let unsafe_ptrs: Vec<(String, u64)> = py_unsafe_ptrs.extract(py)?;
        let ptr_name = match pixel_type {
            PixelType::U8 => "image8",
            _ => "image32",
        };
        let rows_ptr = result2pyresult(
            unsafe_ptrs
                .into_iter()
                .find(|(name, _)| name == ptr_name)
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

    fn set_crop_box(&mut self, left: f64, top: f64, width: f64, height: f64) -> PyResult<()> {
        self.crop_box = Some(CropBox {
            left,
            top,
            width,
            height,
        });
        Ok(())
    }

    #[getter]
    fn pil_image(&self) -> PyResult<Option<PyObject>> {
        Ok(self.pil_image.clone())
    }

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

impl PilImageView {
    pub(crate) fn src_image_view(&self) -> PyResult<DynamicImageView> {
        if let Some(rows_ptr) = self.rows_ptr {
            let mut view = match self.pixel_type {
                PixelType::U8x4 => DynamicImageView::U8x4(self.get_image_view(rows_ptr)?),
                PixelType::I32 => DynamicImageView::I32(self.get_image_view(rows_ptr)?),
                PixelType::F32 => DynamicImageView::F32(self.get_image_view(rows_ptr)?),
                PixelType::U8 => DynamicImageView::U8(self.get_image_view(rows_ptr)?),
                _ => return result2pyresult(Err("Not supported type of pixels")),
            };
            if let Some(crop_box) = self.crop_box {
                result2pyresult(view.set_crop_box(crop_box))?;
            }
            Ok(view)
        } else {
            result2pyresult(Err("PIL image is dropped"))
        }
    }

    fn get_image_view<P: PixelExt>(&self, rows_ptr: u64) -> PyResult<ImageView<P>> {
        result2pyresult(ImageView::new(
            self.width,
            self.height,
            self.get_vec_of_rows(rows_ptr),
        ))
    }

    pub(crate) fn dst_image_view(&mut self) -> PyResult<DynamicImageViewMut> {
        if let Some(rows_ptr) = self.rows_ptr {
            let view = match self.pixel_type {
                PixelType::U8x4 => DynamicImageViewMut::U8x4(self.get_image_view_mut(rows_ptr)?),
                PixelType::I32 => DynamicImageViewMut::I32(self.get_image_view_mut(rows_ptr)?),
                PixelType::F32 => DynamicImageViewMut::F32(self.get_image_view_mut(rows_ptr)?),
                PixelType::U8 => DynamicImageViewMut::U8(self.get_image_view_mut(rows_ptr)?),
                _ => return result2pyresult(Err("Not supported type of pixels")),
            };
            Ok(view)
        } else {
            result2pyresult(Err("PIL image is dropped"))
        }
    }

    fn get_image_view_mut<P: PixelExt>(&self, rows_ptr: u64) -> PyResult<ImageViewMut<P>> {
        result2pyresult(ImageViewMut::new(
            self.width,
            self.height,
            self.get_vec_of_mut_rows(rows_ptr),
        ))
    }

    fn get_vec_of_rows<P>(&self, rows_ptr: u64) -> Vec<&[P]> {
        let rows_ptr = rows_ptr as *const *const P;
        let width = self.width.get() as usize;
        (0..self.height.get() as usize)
            .map(|i| unsafe { slice::from_raw_parts(*rows_ptr.add(i), width) })
            .collect()
    }

    fn get_vec_of_mut_rows<P>(&self, rows_ptr: u64) -> Vec<&mut [P]> {
        let rows_ptr = rows_ptr as *const *mut P;
        let width = self.width.get() as usize;
        (0..self.height.get() as usize)
            .map(|i| unsafe { slice::from_raw_parts_mut(*rows_ptr.add(i), width) })
            .collect()
    }

    pub(crate) fn is_rgb_mode(&self, py: Python) -> PyResult<bool> {
        if let Some(ref pil_image) = self.pil_image {
            let im = pil_image.getattr(py, "im")?;
            let pil_c_image_ptr = im.getattr(py, "ptr")?;
            let image_ptr = unsafe {
                PyCapsule_GetPointer(pil_c_image_ptr.as_ptr(), IMAGING_MAGIC.as_ptr() as _)
            };
            if !image_ptr.is_null() {
                let image_ptr = image_ptr as *const ImagingMemoryInstance;
                let mode = unsafe { &(*image_ptr).mode };
                return Ok(mode.starts_with(b"RGB"));
            }
        }
        result2pyresult(Err("Unknown mode of PIL image"))
    }

    pub(crate) fn set_rgb_mode(&mut self, py: Python, value: RgbMode) -> PyResult<()> {
        if let Some(ref pil_image) = self.pil_image {
            let im = pil_image.getattr(py, "im")?;
            let pil_c_image_ptr = im.getattr(py, "ptr")?;
            let image_ptr = unsafe {
                PyCapsule_GetPointer(pil_c_image_ptr.as_ptr(), IMAGING_MAGIC.as_ptr() as _)
            };
            if !image_ptr.is_null() {
                let image_ptr = image_ptr as *mut ImagingMemoryInstance;
                let mode = unsafe { &mut (*image_ptr).mode };
                match value {
                    RgbMode::RgbA => mode.copy_from_slice(b"RGBA\0\0\0"),
                    RgbMode::Rgba => mode.copy_from_slice(b"RGBa\0\0\0"),
                };
            }
        }
        Ok(())
    }
}
