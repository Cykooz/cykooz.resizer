"""
:Authors: cykooz
:Date: 21.03.2021
"""
from pathlib import Path
from typing import Tuple

import pytest
from PIL import Image

from cykooz.resizer import (
    AlphaMulDiv, CpuExtensions, CropBox, FilterType, ImageData, PixelType, ResizeAlg, Resizer
)
from utils import Checksum, get_image_checksum, save_result


def test_resizer_settings():
    alg = ResizeAlg.super_sampling(FilterType.lanczos3, 2)
    resizer = Resizer(alg)
    resizer.cpu_extensions = CpuExtensions.avx2
    assert resizer.algorithm == alg
    assert resizer.cpu_extensions is CpuExtensions.avx2


@pytest.mark.parametrize(
    ('cpu_extensions', 'checksum'),
    [
        (CpuExtensions.none, Checksum(3032562, 3011548, 2921753, 6122818)),
        (CpuExtensions.sse4_1, Checksum(3032287, 3011377, 2921589, 6122829)),
        (CpuExtensions.avx2, Checksum(3031014, 3010669, 2920705, 6122818)),
    ],
    ids=[
        'wo forced SIMD',
        'sse4.1',
        'avx2',
    ],
)
@pytest.mark.parametrize(
    ('source',),
    [
        ('raw',),
        ('pil',),
    ],
)
def test_resizer(
        source_image: Image.Image,
        source: str,
        cpu_extensions: CpuExtensions,
        checksum: Checksum,
):
    """Resize raw image."""
    image = source_image.copy()

    dst_size = (255, 170)
    if source == 'raw':
        dst_image = _resize_raw(cpu_extensions, image, dst_size, checksum)
    else:
        dst_image = _resize_pil(cpu_extensions, image, dst_size, checksum)

    save_result(
        dst_image,
        Path('resize') / source,
        f'nasa-{dst_image.width}x{dst_image.height}-lanczos3-{cpu_extensions.name}.png',
    )


def _resize_raw(
        cpu_extensions: CpuExtensions,
        src_image: Image.Image,
        dst_size: Tuple[int, int],
        checksum: Checksum,
) -> Image.Image:
    src_image = ImageData(
        src_image.width,
        src_image.height,
        PixelType.U8x4,
        src_image.tobytes('raw')
    )
    dst_image = ImageData(dst_size[0], dst_size[1], PixelType.U8x4)
    assert get_image_checksum(dst_image.get_buffer()) == Checksum(0, 0, 0, 0)

    resizer = Resizer(ResizeAlg.convolution(FilterType.lanczos3))
    if cpu_extensions == CpuExtensions.avx2 and resizer.cpu_extensions != CpuExtensions.avx2:
        raise pytest.skip('AVX2 instruction not supported by CPU')
    resizer.cpu_extensions = cpu_extensions
    mul_div = AlphaMulDiv()
    mul_div.cpu_extensions = cpu_extensions

    mul_div.multiply_alpha_inplace(src_image)
    resizer.resize(src_image, dst_image)
    mul_div.divide_alpha_inplace(dst_image)

    assert get_image_checksum(dst_image.get_buffer()) == checksum

    return Image.frombuffer(
        'RGBA',
        dst_size,
        dst_image.get_buffer(),
        decoder_name='raw',
    )


def _resize_pil(
        cpu_extensions: CpuExtensions,
        src_image: Image.Image,
        dst_size: Tuple[int, int],
        checksum: Checksum,
) -> Image.Image:
    dst_image = Image.new('RGBA', dst_size)
    assert get_image_checksum(dst_image.tobytes('raw')) == Checksum(0, 0, 0, 0)

    resizer = Resizer(ResizeAlg.convolution(FilterType.lanczos3))
    if cpu_extensions == CpuExtensions.avx2 and resizer.cpu_extensions != CpuExtensions.avx2:
        raise pytest.skip('AVX2 instruction not supported by CPU')
    resizer.cpu_extensions = cpu_extensions

    resizer.resize_pil(src_image, dst_image)
    assert dst_image.mode == 'RGBA'
    assert get_image_checksum(dst_image.tobytes('raw')) == checksum

    return dst_image


def test_resize_with_cropping(source_image: Image.Image):
    if source_image.mode != 'RGB':
        source_image = source_image.convert('RGB')
    resizer = Resizer(ResizeAlg.super_sampling(FilterType.lanczos3, 2))
    resizer.cpu_extensions = CpuExtensions.none
    dst_size = (1024, 256)
    crop_box = CropBox.get_crop_box_to_fit_dst_size(source_image.size, dst_size)
    dst_image = Image.new('RGB', dst_size)
    resizer.resize_pil(source_image, dst_image, crop_box)

    save_result(
        dst_image,
        Path('resize') / 'cropping',
        f'nasa-{dst_image.width}x{dst_image.height}.png',
    )


@pytest.mark.parametrize('dst_mode', ('RGB', 'RGBA', 'RGBa', 'CMYK', 'I', 'F'))
@pytest.mark.parametrize('src_mode', ('RGB', 'RGBA', 'RGBa', 'CMYK', 'I', 'F'))
def test_image_modes(source_image: Image.Image, src_mode, dst_mode):
    if source_image.mode != src_mode:
        source_image = source_image.convert(src_mode)
    resizer = Resizer(ResizeAlg.super_sampling(FilterType.lanczos3, 2))
    resizer.cpu_extensions = CpuExtensions.none
    dst_size = (
        int(round(source_image.width / 8)),
        int(round(source_image.height / 8))
    )
    dst_image = Image.new(dst_mode, dst_size)
    resizer.resize_pil(source_image, dst_image)

    save_result(
        dst_image,
        Path('resize') / 'modes',
        f'nasa-{src_mode}_into_{dst_mode}.png',
    )
