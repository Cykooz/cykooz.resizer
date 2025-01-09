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
|  F32   | One `f32` component per pixel                                 |   +    |  +   |  -   |
| F32x2  | Two `f32` components per pixel (e.g. LA32F)                   |   +    |  +   |  -   |
| F32x3  | Three `f32` components per pixel (e.g. RGB32F)                |   +    |  +   |  -   |
| F32x4  | Four `f32` components per pixel (e.g. RGBA32F)                |   +    |  +   |  -   |

Implemented resize algorithms:

- Nearest - is nearest-neighbor interpolation, replacing every pixel with the
  nearest pixel in the output; for upscaling this means multiple pixels of the
  same color will be present.
- Convolution with different filters:
    - box
    - bilinear
    - catmull_rom
    - mitchell
    - gaussian
    - lanczos3
- Super sampling - is resizing an image in two steps.
  The first step uses the "nearest" algorithm. The second step uses "convolution"
  with configurable filter.

## Usage Examples

### Resize Pillow's image

```python
from PIL import Image

from cykooz.resizer import FilterType, ResizeAlg, Resizer, ResizeOptions


resizer = Resizer()
dst_size = (255, 170)
dst_image = Image.new('RGBA', dst_size)

for i in range(1, 10):
    image = Image.open('nasa_%d-4928x3279.png' % i)
    resizer.resize_pil(image, dst_image)
    dst_image.save('nasa_%d-255x170.png' % i)

# Resize using a bilinear filter and ignoring an alpha channel.
image = Image.open('nasa-4928x3279.png')
resizer.resize_pil(
    image,
    dst_image,
    ResizeOptions(
        resize_alg=ResizeAlg.convolution(FilterType.bilinear),
        use_alpha=False,
    )
)
```

### Resize raw image with an alpha channel

```python
from cykooz.resizer import ImageData, PixelType, Resizer


def resize_raw(width: int, height: int, pixels: bytes):
    src_image = ImageData(
        width,
        height,
        PixelType.U8x4,
        pixels,
    )
    resizer = Resizer()
    dst_image = ImageData(255, 170, PixelType.U8x4)
    # By default, Resizer multiplies and divides by alpha channel
    # images with `U8x2`, `U8x4`, `U16x2` and `U16x4` pixels.
    resizer.resize(src_image, dst_image)
    return dst_image
```

### Change used CPU-extensions

```python
from cykooz.resizer import Resizer, CpuExtensions


resizer = Resizer()
resizer.cpu_extensions = CpuExtensions.sse4_1
...
```

### Resize with using thread-pool

```python
from cykooz.resizer import Resizer, ResizeOptions, ResizerThreadPool


...
thread_pool = ResizerThreadPool(num_threads=6)
resizer = Resizer()
resizer.resize(
    src_image,
    dst_image,
    ResizeOptions(thread_pool=thread_pool),
)
...
```

## Benchmarks

Environment:

- CPU: AMD Ryzen 9 5950X
- RAM: DDR4 4000 MHz
- Ubuntu 24.04 (linux 6.8.0)
- Python 3.12
- Rust 1.83.0
- cykooz.resizer = "3.1" (single-threaded mode)

Other Python libraries used to compare of resizing speed:

- Pillow = "11.0.0" (https://pypi.org/project/Pillow/)

Resize algorithms:

- Nearest
- Convolution with Bilinear filter
- Convolution with Lanczos3 filter

### Resize RGBA image 4928x3279 => 852x567

- Source image [nasa-4928x3279.png](https://github.com/Cykooz/cykooz.resizer/blob/main/tests/data/nasa-4928x3279.png)

| Package (time in ms)    | nearest | bilinear | lanczos3 |
|:------------------------|--------:|---------:|---------:|
| Pillow                  |    0.89 |   107.21 |   203.67 |
| cykooz.resizer          |    0.20 |    26.09 |    50.47 |
| cykooz.resizer - sse4_1 |    0.20 |    12.12 |    24.91 |
| cykooz.resizer - avx2   |    0.20 |     8.53 |    22.10 |

### Resize grayscale (U8) image 4928x3279 => 852x567

- Source image [nasa-4928x3279.png](https://github.com/Cykooz/cykooz.resizer/blob/main/tests/data/nasa-4928x3279.png)
  has converted into grayscale image with one byte per pixel.

| Package (time in ms)    | nearest | bilinear | lanczos3 |
|:------------------------|--------:|---------:|---------:|
| Pillow                  |    0.23 |    21.41 |    51.15 |
| cykooz.resizer          |    0.17 |     5.30 |    12.17 |
| cykooz.resizer - sse4_1 |    0.17 |     2.11 |     5.84 |
| cykooz.resizer - avx2   |    0.17 |     1.86 |     4.58 |
