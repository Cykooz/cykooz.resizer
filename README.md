# cykooz.resizer

```cykooz.resizer``` is package with optimized version of image resizing
based on Rust's crate [fast_image_resize](https://crates.io/crates/fast_image_resize).

[CHANGELOG](https://github.com/Cykooz/cykooz.resizer/blob/main/CHANGES.md)

## Installation

```shell
python3 -m pip install cykooz.resizer
```

Or with automatically installing Pillow:

```shell
python3 -m pip install cykooz.resizer[pillow]
```

## Information

Supported pixel types and available optimisations:
- ``U8x4`` - four bytes per pixel (RGB, RGBA, CMYK):
  - native Rust-code without forced SIMD
  - SSE4.1
  - AVX2
- ``I32`` - one signed integer (32 bits) per pixel:
  - native Rust-code without forced SIMD
- ``F32`` - one float (32 bits) per pixel:
  - native Rust-code without forced SIMD
- ``U8`` - one byte per pixel:
  - native Rust-code without forced SIMD

Implemented resize algorithms:
- Nearest - is nearest-neighbor interpolation, replacing every pixel with the 
  nearest pixel in the output; for upscaling this means multiple pixels of the
  same color will be present.
- Convolution with different filters:
  - box
  - bilinear
  - catmull_rom
  - mitchell
  - lanczos3
- Super sampling - resizing an image in two steps.
  First step uses the "nearest" algorithm. Second step uses "convolution" 
  with configurable filter.


## Usage Examples

### Resize Pillow's image

```python
from PIL import Image

from cykooz.resizer import FilterType, ResizeAlg, Resizer


resizer = Resizer(ResizeAlg.convolution(FilterType.lanczos3))
dst_size = (255, 170)
dst_image = Image.new('RGBA', dst_size)

for i in range(1, 10):
    image = Image.open('nasa_%d-4928x3279.png' % i)
    resizer.resize_pil(image, dst_image)
    dst_image.save('nasa_%d-255x170.png' % i)
```

### Resize raw image with alpha channel

```python
from cykooz.resizer import AlphaMulDiv, FilterType, ImageData, PixelType, ResizeAlg, Resizer

def resize_raw(width: int, height: int, pixels: bytes):
    src_image = ImageData(
        width,
        height,
        PixelType.U8x4,
        pixels,
    )
    alpha_mul_div = AlphaMulDiv()
    resizer = Resizer(ResizeAlg.convolution(FilterType.lanczos3))
    dst_image = ImageData(255, 170, PixelType.U8x4)
    alpha_mul_div.multiply_alpha_inplace(src_image)
    resizer.resize(src_image, dst_image)
    alpha_mul_div.divide_alpha_inplace(dst_image)    
    return dst_image
```

### Change used CPU-extensions

```python
from cykooz.resizer import FilterType, ResizeAlg, Resizer, CpuExtensions


resizer = Resizer(ResizeAlg.convolution(FilterType.lanczos3))
resizer.cpu_extensions = CpuExtensions.sse4_1
...
```

## Benchmarks

Environment:
- CPU: Intel(R) Core(TM) i7-6700K CPU @ 4.00GHz
- RAM: DDR4 3000 MHz
- Ubuntu 20.04 (linux 5.11)
- Python 3.9
- Rust 1.56
- cykooz.resizer = "1.1"

Other Python libraries used to compare of resizing speed:
- Pillow = "8.4.0" (https://pypi.org/project/Pillow/)

Resize algorithms:
- Nearest
- Convolution with Bilinear filter
- Convolution with Lanczos3 filter

### Resize RGBA image 4928x3279 => 852x567

- Source image [nasa-4928x3279.png](https://github.com/Cykooz/cykooz.resizer/blob/main/tests/data/nasa-4928x3279.png)

| Package (time in ms)    |   nearest |   bilinear |   lanczos3 |
|:------------------------|----------:|-----------:|-----------:|
| Pillow                  |      0.92 |      99.39 |     230.47 |
| cykooz.resizer          |      0.51 |      68.74 |     126.34 |
| cykooz.resizer - sse4_1 |      0.51 |      25.90 |      39.40 |
| cykooz.resizer - avx2   |      0.51 |      17.99 |      28.40 |


### Resize grayscale (U8) image 4928x3279 => 852x567

- Source image [nasa-4928x3279.png](https://github.com/Cykooz/cykooz.resizer/blob/main/tests/data/nasa-4928x3279.png)
  has converted into grayscale image with one byte per pixel.

| Package (time in ms)    |   nearest |   bilinear |   lanczos3 |
|:------------------------|----------:|-----------:|-----------:|
| Pillow                  |      0.33 |      39.52 |     118.68 |
| cykooz.resizer          |      0.21 |      25.82 |      49.49 |
