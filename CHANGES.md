# CHANGELOG

## [2.1.2] - 2022-10-26

- Added building of wheel for Python 3.11.
- Updated version of ``pyo3`` to 0.17.2.

## [2.1.1] - 2022-07-17

- Fixed resizing when the destination image has the same dimensions 
  as the source image.

## [2.1] - 2022-07-07

- Added support of new types of pixels: `U8x2`, `U16`, `U16x2` and `U16x4`.

## [2.0] - 2022-03-24

- Dropped support for Python 3.6.
- Deleted variant `sse2` from enum `CpuExtensions`.
- Added support of new types of pixels: `U8x3` and `U16x3`.
- Added optimisation for convolution grayscale images (`U8`) 
  with helps of `SSE4.1` and `AVX2` instructions.

## [1.1] - 2021-10-28

- Added support of new type of pixels - U8.
- Added support of L-type of PIL images (grayscale with one byte per pixel).

## [1.0] - 2021-10-08

- First stable release.

## [0.1] - 2021-09-05

- Initial version.
