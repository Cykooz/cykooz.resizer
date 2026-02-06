use crate::utils::result2pyresult;
use fast_image_resize::pixels::PixelType;
use fast_image_resize::{ImageView, ImageViewMut, IntoImageView, IntoImageViewMut, PixelTrait};
use pyo3::prelude::*;
use pyo3::types::PyCapsule;
use pyo3::{PyTraverseError, PyVisit, intern};
use std::ffi::{CStr, c_int, c_void};
use std::marker::PhantomData;
use std::ptr::NonNull;
use std::slice;
use std::str::FromStr;

// https://github.com/python-pillow/Pillow/blob/master/src/libImaging/Imaging.h#L67
static IMAGING_MAGIC: &CStr = c"Pillow Imaging";
// https://github.com/python-pillow/Pillow/blob/master/src/libImaging/Mode.h#L4
const IMAGING_MODE_RGBA: c_int = 13;
#[allow(non_upper_case_globals)]
const IMAGING_MODE_RGBa: c_int = 15;

// https://github.com/python-pillow/Pillow/blob/master/src/libImaging/Imaging.h#L80
#[repr(C)]
struct ImagingMemoryInstanceV11 {
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

#[repr(C)]
struct ImagingMemoryInstanceV12 {
    /// Band names ("1", "L", "P", "RGB", "RGBA", "CMYK", "YCbCr", "BGR;xy")
    mode: c_int,
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

/// Wrapper for `ImagingMemoryInstance` structure from Pillow's C code
struct ImagingMemory<'a> {
    pil_version: u8,
    image_ptr: NonNull<c_void>,
    _phantom: PhantomData<&'a Py<PyAny>>,
}

impl<'a> ImagingMemory<'a> {
    pub fn new(py: Python, pil_image: &'a Py<PyAny>) -> PyResult<Self> {
        // Get reference to `ImagingMemoryInstance` struct from Pillow C-code.
        let im = pil_image.call_method0(py, "getim")?;
        if let Ok(capsule) = im.cast_bound::<PyCapsule>(py) {
            let image_ptr = capsule.pointer_checked(Some(IMAGING_MAGIC))?;
            return Ok(Self {
                pil_version: get_pillow_major_version(py)?,
                image_ptr,
                _phantom: PhantomData,
            });
        }
        result2pyresult(Err(
            "Unable to get ImagingMemoryInstance struc from PIL image",
        ))
    }

    pub fn row_ptr(&self, pixel_type: PixelType) -> u64 {
        let (image8, image32) = if self.pil_version < 12 {
            let image_struct = self.v11_struct();
            (image_struct.image8 as u64, image_struct.image32 as u64)
        } else {
            let image_struct = self.v12_struct();
            (image_struct.image8 as u64, image_struct.image32 as u64)
        };
        match pixel_type {
            PixelType::U8 => image8,
            _ => image32,
        }
    }

    pub fn set_rgb_mode(&mut self, value: RgbMode) {
        if self.pil_version < 12 {
            let image_struct = self.v11_struct_mut();
            match value {
                RgbMode::RgbA => image_struct.mode.copy_from_slice(b"RGBA\0\0\0"),
                RgbMode::Rgba => image_struct.mode.copy_from_slice(b"RGBa\0\0\0"),
            }
        } else {
            let image_struct = self.v12_struct_mut();
            match value {
                RgbMode::RgbA => image_struct.mode = IMAGING_MODE_RGBA,
                RgbMode::Rgba => image_struct.mode = IMAGING_MODE_RGBa,
            }
        }
    }

    fn v11_struct(&self) -> &ImagingMemoryInstanceV11 {
        let image_ptr = self.image_ptr.as_ptr() as *const ImagingMemoryInstanceV11;
        unsafe { &*image_ptr }
    }

    fn v11_struct_mut(&mut self) -> &mut ImagingMemoryInstanceV11 {
        let image_ptr = self.image_ptr.as_ptr() as *mut ImagingMemoryInstanceV11;
        unsafe { &mut *image_ptr }
    }

    fn v12_struct(&self) -> &ImagingMemoryInstanceV12 {
        let image_ptr = self.image_ptr.as_ptr() as *const ImagingMemoryInstanceV12;
        unsafe { &*image_ptr }
    }

    fn v12_struct_mut(&mut self) -> &mut ImagingMemoryInstanceV12 {
        let image_ptr = self.image_ptr.as_ptr() as *mut ImagingMemoryInstanceV12;
        unsafe { &mut *image_ptr }
    }
}

fn get_pillow_major_version(py: Python) -> PyResult<u8> {
    let pil = py.import(intern!(py, "PIL"))?;
    let cache_key = intern!(py, "__cr_major_version__");

    let major_version: u8 = if let Ok(major_version) = pil.getattr(cache_key) {
        major_version.extract()?
    } else {
        let version = pil.getattr(intern!(py, "__version__"))?;
        let version_parts = version.call_method1(intern!(py, "split"), (intern!(py, "."),))?;
        let major_version_str = version_parts.get_item(0)?.to_string();
        let major_version = u8::from_str(&major_version_str)?;
        pil.setattr(cache_key, major_version)?;
        major_version
    };
    Ok(major_version)
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) enum RgbMode {
    RgbA,
    Rgba,
}

#[pyclass]
pub struct PilImageWrapper {
    pil_image: Option<Py<PyAny>>,
    pixel_type: PixelType,
    width: u32,
    height: u32,
    rows_ptr: Option<u64>,
}

#[pymethods]
impl PilImageWrapper {
    #[new]
    fn new(py: Python, pil_image: Py<PyAny>) -> PyResult<Self> {
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

        let pil_struct = ImagingMemory::new(py, &pil_image)?;
        let rows_ptr = pil_struct.row_ptr(pixel_type);

        Ok(Self {
            pil_image: Some(pil_image),
            pixel_type,
            width,
            height,
            rows_ptr: Some(rows_ptr),
        })
    }

    #[getter]
    fn pil_image(&self, py: Python) -> Option<Py<PyAny>> {
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

impl PilImageWrapper {
    /// Get the typed version of the image.
    fn typed_image<P: PixelTrait>(&self) -> Option<TypedPilImage<'_, P>> {
        TypedPilImage::new(self)
    }

    /// Get the typed mutable version of the image.
    fn typed_image_mut<P: PixelTrait>(&mut self) -> Option<TypedPilImageMut<'_, P>> {
        TypedPilImageMut::new(self)
    }

    pub(crate) fn is_rgb_mode(&self, py: Python) -> PyResult<bool> {
        if let Some(ref pil_image) = self.pil_image {
            let py_mode = pil_image.getattr(py, "mode")?;
            let mode: String = py_mode.extract(py)?;
            return Ok(mode.starts_with("RGB"));
        }
        result2pyresult(Err("Unknown mode of PIL image"))
    }

    pub(crate) fn set_rgb_mode(&mut self, py: Python, value: RgbMode) -> PyResult<()> {
        if let Some(pil_image) = &mut self.pil_image {
            let mut pil_struct = ImagingMemory::new(py, &pil_image)?;
            pil_struct.set_rgb_mode(value);
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
