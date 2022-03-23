use std::fmt::Display;

use fast_image_resize as fr;
use fast_image_resize::pixels::PixelType;
use pyo3::exceptions;
use pyo3::prelude::*;

pub(crate) fn result2pyresult<T, E: Display>(res: Result<T, E>) -> PyResult<T> {
    res.map_err(|err| PyErr::new::<exceptions::PyRuntimeError, _>(err.to_string()))
}

macro_rules! into_non_zero {
    ($x:expr) => {{
        $crate::utils::result2pyresult(
            std::num::NonZeroU32::new($x)
                .ok_or_else(|| format!("Value of '{}' is zero", stringify!($x))),
        )
    }};
}

pub(crate) fn pixel_type_from_u8(pixel_type: u8) -> PixelType {
    match pixel_type {
        1 => PixelType::U8,
        2 => PixelType::I32,
        3 => PixelType::F32,
        4 => PixelType::U8x3,
        5 => PixelType::U8x4,
        6 => PixelType::U16x3,
        _ => PixelType::U8x4,
    }
}

pub(crate) fn cpu_extensions_from_u8(extensions: u8) -> fr::CpuExtensions {
    match extensions {
        1 => fr::CpuExtensions::None,
        2 => fr::CpuExtensions::Sse4_1,
        3 => fr::CpuExtensions::Avx2,
        _ => Default::default(),
    }
}

pub(crate) fn cpu_extensions_to_u8(extensions: fr::CpuExtensions) -> u8 {
    match extensions {
        fr::CpuExtensions::None => 1,
        fr::CpuExtensions::Sse4_1 => 2,
        fr::CpuExtensions::Avx2 => 3,
    }
}
