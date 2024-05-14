use std::slice;

use fast_image_resize::pixels::PixelType;
use fast_image_resize::{ImageView, ImageViewMut, IntoImageView, IntoImageViewMut, PixelTrait};
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
pub struct PilImageWrapper {
    pil_image: Option<PyObject>,
    pixel_type: PixelType,
    width: u32,
    height: u32,
    rows_ptr: Option<u64>,
}

#[pymethods]
impl PilImageWrapper {
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
        })
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

impl PilImageWrapper {
    /// Get the typed version of the image.
    fn typed_image<P: PixelTrait>(&self) -> Option<TypedPilImage<P>> {
        TypedPilImage::new(self)
    }

    /// Get the typed mutable version of the image.
    fn typed_image_mut<P: PixelTrait>(&mut self) -> Option<TypedPilImageMut<P>> {
        TypedPilImageMut::new(self)
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

impl IntoImageView for PilImageWrapper {
    fn pixel_type(&self) -> Option<PixelType> {
        Some(self.pixel_type)
    }

    fn width(&self) -> u32 {
        self.width
    }

    fn height(&self) -> u32 {
        self.height
    }

    fn image_view<P: PixelTrait>(&self) -> Option<impl ImageView<Pixel = P>> {
        self.typed_image()
    }
}

impl IntoImageViewMut for PilImageWrapper {
    fn image_view_mut<P: PixelTrait>(&mut self) -> Option<impl ImageViewMut<Pixel = P>> {
        self.typed_image_mut()
    }
}

/// Generic image container that provides [ImageView].
pub(crate) struct TypedPilImage<'a, P: PixelTrait> {
    pil_image: &'a PilImageWrapper,
    rows_ptr: *const *const P,
}

impl<'a, P: PixelTrait> TypedPilImage<'a, P> {
    pub fn new(pil_image: &'a PilImageWrapper) -> Option<Self> {
        if let Some(rows_ptr) = pil_image.rows_ptr {
            if P::pixel_type() == pil_image.pixel_type {
                return Some(Self {
                    pil_image,
                    rows_ptr: rows_ptr as *const *const P,
                });
            }
        }
        None
    }
}

unsafe impl<'a, P: PixelTrait> ImageView for TypedPilImage<'a, P> {
    type Pixel = P;

    fn width(&self) -> u32 {
        self.pil_image.width
    }

    fn height(&self) -> u32 {
        self.pil_image.height
    }

    fn iter_rows(&self, start_row: u32) -> impl Iterator<Item = &[Self::Pixel]> {
        let start = start_row as usize;
        let end = self.height() as usize;
        let width = self.width() as usize;
        (start..end).map(move |i| unsafe { slice::from_raw_parts(*self.rows_ptr.add(i), width) })
    }
}

pub(crate) struct TypedPilImageMut<'a, P: Default + Copy> {
    pil_image: &'a PilImageWrapper,
    rows_ptr: *const *mut P,
}

impl<'a, P: PixelTrait> TypedPilImageMut<'a, P> {
    pub fn new(pil_image: &'a PilImageWrapper) -> Option<Self> {
        if let Some(rows_ptr) = pil_image.rows_ptr {
            if P::pixel_type() == pil_image.pixel_type {
                return Some(Self {
                    pil_image,
                    rows_ptr: rows_ptr as *const *mut P,
                });
            }
        }
        None
    }
}

unsafe impl<'a, P: PixelTrait> ImageView for TypedPilImageMut<'a, P> {
    type Pixel = P;

    fn width(&self) -> u32 {
        self.pil_image.width
    }

    fn height(&self) -> u32 {
        self.pil_image.height
    }

    fn iter_rows(&self, start_row: u32) -> impl Iterator<Item = &[Self::Pixel]> {
        let start = start_row as usize;
        let end = self.height() as usize;
        let width = self.width() as usize;
        (start..end).map(move |i| unsafe { slice::from_raw_parts(*self.rows_ptr.add(i), width) })
    }
}

unsafe impl<'a, P: PixelTrait> ImageViewMut for TypedPilImageMut<'a, P> {
    fn iter_rows_mut(&mut self, start_row: u32) -> impl Iterator<Item = &mut [Self::Pixel]> {
        let start = start_row as usize;
        let end = self.height() as usize;
        let width = self.width() as usize;
        (start..end)
            .map(move |i| unsafe { slice::from_raw_parts_mut(*self.rows_ptr.add(i), width) })
    }
}
