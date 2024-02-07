# cykooz.resizer

```cykooz.resizer``` is package with the optimized version of image resizing
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

| Format | Description                                                   | SSE4.1 | AVX2 | Neon |
|:------:|:--------------------------------------------------------------|:------:|:----:|:----:|
|   U8   | One `u8` component per pixel (e.g. L)                         |   +    |  +   |  +   |
|  U8x2  | Two `u8` components per pixel (e.g. LA)                       |   +    |  +   |  +   |
|  U8x3  | Three `u8` components per pixel (e.g. RGB)                    |   +    |  +   |  +   |
|  U8x4  | Four `u8` components per pixel (e.g. RGBA, RGBx, CMYK)        |   +    |  +   |  +   |
|  U16   | One `u16` components per pixel (e.g. L16)                     |   +    |  +   |  +   |
| U16x2  | Two `u16` components per pixel (e.g. LA16)                    |   +    |  +   |  +   |
| U16x3  | Three `u16` components per pixel (e.g. RGB16)                 |   +    |  +   |  +   |
| U16x4  | Four `u16` components per pixel (e.g. RGBA16, RGBx16, CMYK16) |   +    |  +   |  +   |
|  I32   | One `i32` component per pixel                                 |   -    |  -   |  -   |
|  F32   | One `f32` component per pixel                                 |   -    |  -   |  -   |

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
- Super sampling - is resizing an image in two steps.
  The first step uses the "nearest" algorithm. The second step uses "convolution" 
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

### Resize raw image with an alpha channel

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
- CPU: AMD Ryzen 9 5950X
- RAM: DDR4 3800 MHz
- Ubuntu 22.04 (linux 6.5.0)
- Python 3.10
- Rust 1.75.0
- cykooz.resizer = "2.2"

Other Python libraries used to compare of resizing speed:
- Pillow = "10.2.0" (https://pypi.org/project/Pillow/)

Resize algorithms:
- Nearest
- Convolution with Bilinear filter
- Convolution with Lanczos3 filter

### Resize RGBA image 4928x3279 => 852x567

- Source image [nasa-4928x3279.png](https://github.com/Cykooz/cykooz.resizer/blob/main/tests/data/nasa-4928x3279.png)

| Package (time in ms)       | nearest | bilinear | lanczos3 |
|:---------------------------|--------:|---------:|---------:|
| Pillow                     |    0.88 |   105.27 |   200.80 |
| cykooz.resizer             |    0.20 |    29.64 |    58.83 |
| cykooz.resizer - sse4_1    |    0.20 |    14.83 |    27.87 |
| cykooz.resizer - avx2      |    0.20 |    10.44 |    21.34 |


### Resize grayscale (U8) image 4928x3279 => 852x567

- Source image [nasa-4928x3279.png](https://github.com/Cykooz/cykooz.resizer/blob/main/tests/data/nasa-4928x3279.png)
  has converted into grayscale image with one byte per pixel.

| Package (time in ms)       | nearest | bilinear | lanczos3 |
|:---------------------------|--------:|---------:|---------:|
| Pillow U8                  |    0.27 |    23.78 |    61.23 |
| cykooz.resizer U8          |    0.17 |     5.29 |    11.44 |
| cykooz.resizer U8 - sse4_1 |    0.17 |     2.30 |     5.90 |
| cykooz.resizer U8 - avx2   |    0.17 |     1.85 |     4.17 |
