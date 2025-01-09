use std::fmt::Display;

use fast_image_resize as fr;
use fast_image_resize::pixels::PixelType;
use pyo3::exceptions;
use pyo3::prelude::*;

#[inline]
pub(crate) fn result2pyresult<T, E: Display>(res: Result<T, E>) -> PyResult<T> {
    res.map_err(|err| PyErr::new::<exceptions::PyRuntimeError, _>(err.to_string()))
}

pub(crate) fn pixel_type_from_u8(pixel_type: u8) -> PixelType {
    match pixel_type {
        1 => PixelType::U8,
        2 => PixelType::U8x2,
        3 => PixelType::U8x3,
        4 => PixelType::U8x4,
        5 => PixelType::U16,
        6 => PixelType::U16x2,
        7 => PixelType::U16x3,
        8 => PixelType::U16x4,
        9 => PixelType::I32,
        10 => PixelType::F32,
        11 => PixelType::F32x2,
        12 => PixelType::F32x3,
        13 => PixelType::F32x4,
        _ => PixelType::U8x4,
    }
}

pub(crate) fn cpu_extensions_from_u8(extensions: u8) -> fr::CpuExtensions {
    match extensions {
        1 => fr::CpuExtensions::None,
        #[cfg(target_arch = "x86_64")]
        2 => fr::CpuExtensions::Sse4_1,
        #[cfg(target_arch = "x86_64")]
        3 => fr::CpuExtensions::Avx2,
        #[cfg(target_arch = "aarch64")]
        4 => fr::CpuExtensions::Neon,
        _ => Default::default(),
    }
}

pub(crate) fn cpu_extensions_to_u8(extensions: fr::CpuExtensions) -> u8 {
    match extensions {
        fr::CpuExtensions::None => 1,
        #[cfg(target_arch = "x86_64")]
        fr::CpuExtensions::Sse4_1 => 2,
        #[cfg(target_arch = "x86_64")]
        fr::CpuExtensions::Avx2 => 3,
        #[cfg(target_arch = "aarch64")]
        fr::CpuExtensions::Neon => 4,
    }
}
