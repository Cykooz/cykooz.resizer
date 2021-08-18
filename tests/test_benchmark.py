"""
:Authors: cykooz
:Date: 12.08.2021
"""
from pathlib import Path

import pytest
from PIL import Image
from pytest_benchmark.stats import Metadata

from cykooz.resizer import (
    Algorithm, AlphaMulDiv, CpuExtensions, FilterType, ImageData,
    PixelType, ResizeAlg, Resizer,
)
from utils import BenchResults


CPU_EXTENSIONS = [
    CpuExtensions.none,
    CpuExtensions.sse4_1,
    CpuExtensions.avx2,
]


@pytest.fixture(
    name='cpu_extensions',
    params=CPU_EXTENSIONS,
    ids=[e.name for e in CPU_EXTENSIONS],
)
def cpu_extensions_fixture(request):
    return request.param


@pytest.fixture(
    name='resize_alg',
    params=[
        ResizeAlg.nearest(),
        ResizeAlg.convolution(FilterType.bilinear),
        ResizeAlg.convolution(FilterType.lanczos3),
    ],
    ids=[
        'nearest',
        'bilinear',
        'lanczos3',
    ],
)
def resize_alg_fixture(request):
    return request.param


@pytest.fixture(name='resizer')
def resizer_fixture(cpu_extensions, resize_alg):
    resizer = Resizer(resize_alg)
    resizer.cpu_extensions = cpu_extensions
    return resizer


@pytest.fixture(name='alpha_mul_div')
def alpha_mul_div_fixture(cpu_extensions):
    alpha_mul_div = AlphaMulDiv()
    alpha_mul_div.cpu_extensions = cpu_extensions
    return alpha_mul_div


@pytest.fixture(name='source_image')
def source_image_fixture() -> Image.Image:
    data_dir = Path(__file__).parent / 'data'
    img_path = data_dir / 'nasa-4928x3279.png'
    image: Image.Image = Image.open(img_path)
    if image.mode != 'RGBA':
        image = image.convert('RGBA')
    image.load()
    return image


@pytest.fixture(name='results', scope='session')
def results_fixture():
    results = BenchResults()
    yield results
    print()
    results.print_table()


DST_SIZE = (852, 567)

# Pillow-SIMD

PIL_FILTERS = {
    Image.NEAREST: 'nearest',
    Image.BILINEAR: 'bilinear',
    Image.LANCZOS: 'lanczos3',
}


@pytest.fixture(
    name='pil_filter',
    params=list(PIL_FILTERS.keys()),
    ids=list(PIL_FILTERS.values()),
)
def pil_filter_fixture(request):
    return request.param


def resize_pillow(src_image: Image.Image, pil_filter):
    src_image.resize(DST_SIZE, pil_filter)


def test_resize_pillow(benchmark, pil_filter, source_image, results: BenchResults):
    if source_image.mode != 'RGBA':
        source_image = source_image.convert('RGBA')

    def setup():
        src_image = source_image.copy()
        return (src_image, pil_filter), {}

    benchmark.pedantic(resize_pillow, setup=setup, rounds=50, warmup_rounds=3)

    alg = PIL_FILTERS[pil_filter]
    stats: Metadata = benchmark.stats
    value = stats.stats.mean * 1000
    results.add('Pillow', alg, f'{value:.2f}')


# cykooz.resizer - resize raw image

def resize_raw(
        alpha_mul_div: AlphaMulDiv,
        resizer: Resizer,
        src_image: ImageData,
        dst_image: ImageData,
):
    alpha_mul_div.multiply_alpha_inplace(src_image)
    resizer.resize(src_image, dst_image)
    alpha_mul_div.divide_alpha_inplace(dst_image)


@pytest.mark.skip('Only manual running')
def test_resize_raw(benchmark, resizer, alpha_mul_div, source_image):
    if source_image.mode != 'RGBA':
        source_image = source_image.convert('RGBA')
    width, height = source_image.size
    dst_image = ImageData(DST_SIZE[0], DST_SIZE[1], PixelType.U8x4)

    def setup():
        src_image = ImageData(width, height, PixelType.U8x4, source_image.tobytes())
        return (alpha_mul_div, resizer, src_image, dst_image), {}

    benchmark.pedantic(resize_raw, setup=setup, rounds=50, warmup_rounds=3)


# cykooz.resizer - resize PIL image

def resize_pil(
        resizer: Resizer,
        src_image: Image.Image,
        dst_image: Image.Image,
):
    resizer.resize_pil(src_image, dst_image)


def test_resize_pil(benchmark, resizer: Resizer, alpha_mul_div, source_image, results: BenchResults):
    if source_image.mode != 'RGBA':
        source_image = source_image.convert('RGBA')
    dst_image = Image.new('RGBA', DST_SIZE)

    def setup():
        dst_image.mode = 'RGBA'
        return (alpha_mul_div, resizer, source_image, dst_image), {}

    benchmark.pedantic(resize_pil, setup=setup, rounds=10, warmup_rounds=3)

    row_name = 'cykooz.resizer'
    if resizer.cpu_extensions != CpuExtensions.none:
        row_name += f' - {resizer.cpu_extensions.name}'

    alg = resizer.algorithm.algorithm
    if alg == Algorithm.nearest:
        alg = 'nearest'
    else:
        alg = resizer.algorithm.filter_type.name

    stats: Metadata = benchmark.stats
    value = stats.stats.mean * 1000
    results.add(row_name, alg, f'{value:.2f}')
