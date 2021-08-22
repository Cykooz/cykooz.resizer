"""
:Authors: cykooz
:Date: 21.03.2021
"""
from pathlib import Path
from typing import Tuple

import pytest
from PIL import Image

from cykooz.resizer import (
    CpuExtensions, FilterType, ImageData, PixelType, ResizeAlg, Resizer
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
        (CpuExtensions.none, Checksum(2933561, 2927798, 2850381, 6122818)),
        (CpuExtensions.sse4_1, Checksum(2933594, 2927835, 2850433, 6122829)),
        (CpuExtensions.avx2, Checksum(2933561, 2927798, 2850381, 6122818)),
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
    if image.mode != 'RGBa':
        image = image.convert('RGBa')

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

    resizer.resize(src_image, dst_image)
    assert get_image_checksum(dst_image.get_buffer()) == checksum

    return Image.frombuffer(
        'RGBa',
        dst_size,
        dst_image.get_buffer(),
        decoder_name='raw',
    ).convert('RGBA')


def _resize_pil(
        cpu_extensions: CpuExtensions,
        src_image: Image.Image,
        dst_size: Tuple[int, int],
        checksum: Checksum,
) -> Image.Image:
    dst_image = Image.new('RGBa', dst_size)
    assert get_image_checksum(dst_image.tobytes('raw')) == Checksum(0, 0, 0, 0)

    resizer = Resizer(ResizeAlg.convolution(FilterType.lanczos3))
    if cpu_extensions == CpuExtensions.avx2 and resizer.cpu_extensions != CpuExtensions.avx2:
        raise pytest.skip('AVX2 instruction not supported by CPU')
    resizer.cpu_extensions = cpu_extensions

    resizer.resize_pil(src_image, dst_image)
    assert dst_image.mode == 'RGBa'
    assert get_image_checksum(dst_image.tobytes('raw')) == checksum

    return dst_image.convert('RGBA')
