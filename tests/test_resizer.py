"""
:Authors: cykooz
:Date: 21.03.2021
"""
from pathlib import Path
from typing import Optional, Tuple

import pytest
from PIL import Image

from cykooz.resizer import (
    CpuExtensions,
    FilterType,
    ImageData,
    PixelType,
    ResizeAlg,
    ResizeOptions,
    Resizer,
    ResizerThreadPool,
)
from utils import Checksum, get_image_checksum, save_result


def test_resize_options():
    alg = ResizeAlg.super_sampling(FilterType.lanczos3, 2)
    options = ResizeOptions(alg)
    assert options.resize_alg == alg


def test_resizer_cpu_extensions():
    resizer = Resizer()
    if resizer.cpu_extensions is CpuExtensions.avx2:
        resizer.cpu_extensions = CpuExtensions.sse4_1
        assert resizer.cpu_extensions is CpuExtensions.sse4_1
    elif resizer.cpu_extensions is CpuExtensions.neon:
        resizer.cpu_extensions = CpuExtensions.none
        assert resizer.cpu_extensions is CpuExtensions.none


@pytest.mark.parametrize(
    ('cpu_extensions', 'checksum'),
    [
        (CpuExtensions.none, Checksum(3037693, 3015698, 2922607, 6122718)),
        (CpuExtensions.sse4_1, Checksum(3037693, 3015698, 2922607, 6122718)),
        (CpuExtensions.avx2, Checksum(3037693, 3015698, 2922607, 6122718)),
        (CpuExtensions.neon, Checksum(3037693, 3015698, 2922607, 6122718)),
    ],
    ids=[
        'wo forced SIMD',
        'sse4.1',
        'avx2',
        'neon',
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
        thread_pool,
        cpu_extensions: CpuExtensions,
        checksum: Checksum,
):
    """Resize raw image."""
    image = source_image.copy()

    dst_size = (255, 170)
    if source == 'raw':
        dst_image = _resize_raw(cpu_extensions, image, dst_size, checksum, thread_pool)
    else:
        dst_image = _resize_pil(cpu_extensions, image, dst_size, checksum, thread_pool)

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
        thread_pool: Optional[ResizerThreadPool],
) -> Image.Image:
    src_image = ImageData(
        src_image.width, src_image.height, PixelType.U8x4, src_image.tobytes('raw')
    )
    dst_image = ImageData(dst_size[0], dst_size[1], PixelType.U8x4)
    assert get_image_checksum(dst_image.get_buffer()) == Checksum(0, 0, 0, 0)

    resizer = Resizer()
    resizer.cpu_extensions = cpu_extensions
    if resizer.cpu_extensions != cpu_extensions:
        raise pytest.skip(f'{cpu_extensions.name} instruction not supported by CPU')

    resizer.resize(
        src_image,
        dst_image,
        ResizeOptions(
            ResizeAlg.convolution(FilterType.lanczos3),
            thread_pool=thread_pool,
        ),
    )

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
        thread_pool: Optional[ResizerThreadPool],
) -> Image.Image:
    dst_image = Image.new('RGBA', dst_size)
    assert get_image_checksum(dst_image.tobytes('raw')) == Checksum(0, 0, 0, 0)

    resizer = Resizer()
    resizer.cpu_extensions = cpu_extensions
    if resizer.cpu_extensions != cpu_extensions:
        raise pytest.skip(f'{cpu_extensions.name} instruction not supported by CPU')

    resizer.resize_pil(
        src_image,
        dst_image,
        ResizeOptions(
            ResizeAlg.convolution(FilterType.lanczos3),
            thread_pool=thread_pool,
        ),
    )
    assert dst_image.mode == 'RGBA'
    assert get_image_checksum(dst_image.tobytes('raw')) == checksum

    return dst_image


def test_resize_with_cropping(source_image: Image.Image, thread_pool):
    if source_image.mode != 'RGB':
        source_image = source_image.convert('RGB')

    resizer = Resizer()
    resizer.cpu_extensions = CpuExtensions.none
    dst_size = (1024, 256)
    dst_image = Image.new('RGB', dst_size)

    resizer.resize_pil(
        source_image,
        dst_image,
        ResizeOptions(
            ResizeAlg.super_sampling(FilterType.lanczos3, 2),
            fit_into_destination=True,
            thread_pool=thread_pool,
        )
    )

    save_result(
        dst_image,
        Path('resize') / 'cropping',
        f'nasa-{dst_image.width}x{dst_image.height}.png',
    )


@pytest.mark.parametrize('dst_mode', ('RGB', 'RGBA', 'RGBa', 'CMYK', 'I', 'F', 'L'))
@pytest.mark.parametrize('src_mode', ('RGB', 'RGBA', 'RGBa', 'CMYK', 'I', 'F', 'L'))
def test_image_modes(source_image: Image.Image, thread_pool, src_mode, dst_mode):
    if source_image.mode != src_mode:
        source_image = source_image.convert(src_mode)
    resizer = Resizer()
    resizer.cpu_extensions = CpuExtensions.none
    dst_size = (int(round(source_image.width / 8)), int(round(source_image.height / 8)))
    dst_image = Image.new(dst_mode, dst_size)
    resizer.resize_pil(
        source_image,
        dst_image,
        ResizeOptions(
            ResizeAlg.super_sampling(FilterType.lanczos3, 2),
            thread_pool=thread_pool,
        ),
    )

    save_result(
        dst_image,
        Path('resize') / 'modes',
        f'nasa-{src_mode}_into_{dst_mode}.png',
    )
