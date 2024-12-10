use std::ffi::{c_int, c_void};
use std::marker::PhantomData;
use std::slice;

use crate::utils::result2pyresult;
use fast_image_resize::pixels::PixelType;
use fast_image_resize::{ImageView, ImageViewMut, IntoImageView, IntoImageViewMut, PixelTrait};
use pyo3::prelude::*;
use pyo3::types::PyCapsule;
use pyo3::{PyTraverseError, PyVisit};

// https://github.com/python-pillow/Pillow/blob/master/src/libImaging/Imaging.h#L80
#[repr(C)]
struct ImagingMemoryInstance {
    /// Band names ("1", "L", "P", "RGB", "RGBA", "CMYK", "YCbCr", "BGR;xy")
    mode: [u8; 7],
    /// Data type (IMAGING_TYPE_*)
    r#type: c_int,
    /// Depth (ignored in this version)
    depth: c_int,
    /// Number of bands (1, 2, 3, or 4)
    bands: c_int,
    /// Image dimension.
    xsize: c_int,
    /// Image dimension.
    ysize: c_int,
    /// Colour palette (for "P" images only)
    palette: *mut c_void,
    /// Set for 8-bit images (pixelsize=1)
    image8: *mut *mut u8,
    /// Set for 32-bit images (pixelsize=4)
    image32: *mut *mut i32,
}

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

        let pil_struct = pil_struct(&pil_image, py)?;
        let rows_ptr = match pixel_type {
            PixelType::U8 => pil_struct.image8 as u64,
            _ => pil_struct.image32 as u64,
        };

        Ok(Self {
            pil_image: Some(pil_image),
            pixel_type,
            width,
            height,
            rows_ptr: Some(rows_ptr),
        })
    }

    #[getter]
    fn pil_image(&self, py: Python) -> Option<PyObject> {
        self.pil_image.as_ref().map(|img| img.clone_ref(py))
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

/// Returns reference to `ImagingMemoryInstance` struct from Pillow C-code.
fn pil_struct<'i>(pil_image: &'i PyObject, py: Python) -> PyResult<&'i ImagingMemoryInstance> {
    let im = pil_image.call_method0(py, "getim")?;
    if let Ok(capsule) = im.downcast_bound::<PyCapsule>(py) {
        let image_ptr = capsule.pointer() as *const ImagingMemoryInstance;
        if !image_ptr.is_null() {
            return Ok(unsafe { &*image_ptr });
        }
    }
    result2pyresult(Err(
        "Unable to get ImagingMemoryInstance struc from PIL image",
    ))
}

/// Returns mutable reference to `ImagingMemoryInstance` struct from Pillow C-code.
fn pil_struct_mut<'i>(
    pil_image: &'i mut PyObject,
    py: Python,
) -> PyResult<&'i mut ImagingMemoryInstance> {
    let im = pil_image.call_method0(py, "getim")?;
    if let Ok(capsule) = im.downcast_bound::<PyCapsule>(py) {
        let image_ptr = capsule.pointer() as *mut ImagingMemoryInstance;
        if !image_ptr.is_null() {
            return Ok(unsafe { &mut *image_ptr });
        }
    }
    result2pyresult(Err(
        "Unable to get ImagingMemoryInstance struc from PIL image",
    ))
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
            let pil_struct = pil_struct(pil_image, py)?;
            return Ok(pil_struct.mode.starts_with(b"RGB"));
        }
        result2pyresult(Err("Unknown mode of PIL image"))
    }

    pub(crate) fn set_rgb_mode(&mut self, py: Python, value: RgbMode) -> PyResult<()> {
        if let Some(pil_image) = &mut self.pil_image {
            let pil_struct = pil_struct_mut(pil_image, py)?;
            match value {
                RgbMode::RgbA => pil_struct.mode.copy_from_slice(b"RGBA\0\0\0"),
                RgbMode::Rgba => pil_struct.mode.copy_from_slice(b"RGBa\0\0\0"),
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
    rows_ptr: u64,
    phantom: PhantomData<P>,
}

impl<'a, P: PixelTrait> TypedPilImage<'a, P> {
    pub fn new(pil_image: &'a PilImageWrapper) -> Option<Self> {
        if let Some(rows_ptr) = pil_image.rows_ptr {
            if P::pixel_type() == pil_image.pixel_type {
                return Some(Self {
                    pil_image,
                    rows_ptr,
                    phantom: PhantomData,
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
        let ptr = self.rows_ptr as *const *const P;
        (start..end).map(move |i| unsafe { slice::from_raw_parts(*ptr.add(i), width) })
    }
}

pub(crate) struct TypedPilImageMut<'a, P: Default + Copy> {
    pil_image: &'a PilImageWrapper,
    rows_ptr: u64,
    phantom: PhantomData<P>,
}

impl<'a, P: PixelTrait> TypedPilImageMut<'a, P> {
    pub fn new(pil_image: &'a PilImageWrapper) -> Option<Self> {
        if let Some(rows_ptr) = pil_image.rows_ptr {
            if P::pixel_type() == pil_image.pixel_type {
                return Some(Self {
                    pil_image,
                    rows_ptr,
                    phantom: PhantomData,
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
        let ptr = self.rows_ptr as *const *const P;
        (start..end).map(move |i| unsafe { slice::from_raw_parts(*ptr.add(i), width) })
    }
}

unsafe impl<'a, P: PixelTrait> ImageViewMut for TypedPilImageMut<'a, P> {
    fn iter_rows_mut(&mut self, start_row: u32) -> impl Iterator<Item = &mut [Self::Pixel]> {
        let start = start_row as usize;
        let end = self.height() as usize;
        let width = self.width() as usize;
        let ptr = self.rows_ptr as *const *mut P;
        (start..end).map(move |i| unsafe { slice::from_raw_parts_mut(*ptr.add(i), width) })
    }
}
