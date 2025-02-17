# CHANGELOG

## [3.1.1] - 2025-02-17

- Updated version of ``fast_image_resize`` to 5.1.2.
    - Fixed error in implementation of `ImageView::split_by_width()`, `ImageView::split_by_height()`,
      `ImageViewMut::split_by_width_mut()` and `ImageViewMut::split_by_height_mut()`.
    
## [3.1.0] - 2025-01-09

- Updated version of ``fast_image_resize`` to 5.1.0.
- Added support for multi-thread image processing.
  You can create an instance of `ResizerThreadPool` and use it for set
  `ResizeOptions.thread_pool` field or put it as argument of `AlphaMulDiv` methods.
- Added support for optimization with help of `SSE4.1` and `AVX2` for
  the `F32` pixel type.
- Added support for new pixel types `F32x2`, `F32x3` and `F32x4` with
  optimizations for `SSE4.1` and `AVX2`.

## [3.0.0] - 2024-05-15

- Updated version of ``fast_image_resize`` to 4.0.0.
- Added support of `Neon` CPU-instructions to speedup image resizing
  on ARM architecture.
- Added optional argument `options: ResizeOptions` into `Resizer.resize()`
  and `Resizer.resize_pil()` methods.
- Added support for images with zero sizes (width or/and height).
- Added `FilterType.gaussian` filter for convolution resize algorithm.
- Updated version of ``pyo3`` to 0.21.2.
- **BREAKING CHANGES:**
    - Argument `resize_alg` was removed from `Resizer.__init__()` method.
      You have to use `options` argument of `Resizer.resize()`
      and `Resizer.resize_pil()` methods to change resize algorithm.
    - `Resizer`, by default, multiplies and divides color channels of image by
      an alpha channel. You may change this behavior through `options` argument.
    - Deleted support of Python 3.7.

## [2.2.1] - 2024-02-15

- Fixed error with incorrect cropping of source image.

## [2.2.0] - 2024-02-08

- Added support of ``Pillow`` >= 10.1.0.
- Updated version of ``pyo3`` to 0.20.2.
- Updated version of ``fast_image_resize`` to 3.0.2.
- Added building of wheel for Python 3.12.

## [2.1.2] - 2022-10-26

- Added building of wheel for Python 3.11.
- Updated version of ``pyo3`` to 0.17.2.

## [2.1.1] - 2022-07-17

- Fixed resizing when the destination image has the same dimensions
  as the source image.

## [2.1] - 2022-07-07

- Added support of new pixel types: `U8x2`, `U16`, `U16x2` and `U16x4`.

## [2.0] - 2022-03-24

- Dropped support for Python 3.6.
- Deleted variant `sse2` from enum `CpuExtensions`.
- Added support of new pixel types: `U8x3` and `U16x3`.
- Added optimization for convolution grayscale images (`U8`)
  with helps of `SSE4.1` and `AVX2` instructions.

## [1.1] - 2021-10-28

- Added support of the new pixel type `U8`.
- Added support of `L-type` PIL images (grayscale with one byte per pixel).

## [1.0] - 2021-10-08

- First stable release.

## [0.1] - 2021-09-05

- Initial version.
