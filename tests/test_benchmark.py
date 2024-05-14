"""
:Authors: cykooz
:Date: 12.08.2021
"""
from pathlib import Path

import pytest
from PIL import Image
from PIL.Image import Resampling
from pytest_benchmark.stats import Metadata

from cykooz.resizer import (
    Algorithm,
    CpuExtensions,
    FilterType,
    ImageData,
    PixelType,
    ResizeAlg,
    ResizeOptions, Resizer,
)
from cykooz.resizer.alpha import set_image_mode
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
def resizer_fixture(cpu_extensions):
    resizer = Resizer()
    resizer.cpu_extensions = cpu_extensions
    return resizer


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

# Pillow

PIL_FILTERS = {
    Resampling.NEAREST: 'nearest',
    Resampling.BILINEAR: 'bilinear',
    Resampling.LANCZOS: 'lanczos3',
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
        resizer: Resizer,
        resize_options: ResizeOptions,
        src_image: ImageData,
        dst_image: ImageData,
):
    resizer.resize(src_image, dst_image, resize_options)


@pytest.mark.skip('Only manual running')
def test_resize_raw(benchmark, resizer, resize_alg, source_image):
    if source_image.mode != 'RGBA':
        source_image = source_image.convert('RGBA')
    width, height = source_image.size
    dst_image = ImageData(DST_SIZE[0], DST_SIZE[1], PixelType.U8x4)
    resize_options = ResizeOptions(resize_alg)

    def setup():
        src_image = ImageData(width, height, PixelType.U8x4, source_image.tobytes())
        return (resizer, resize_options, src_image, dst_image), {}

    benchmark.pedantic(resize_raw, setup=setup, rounds=50, warmup_rounds=3)


# cykooz.resizer - resize PIL image


def resize_pil(
        resizer: Resizer,
        resize_options: ResizeOptions,
        src_image: Image.Image,
        dst_image: Image.Image,
):
    resizer.resize_pil(src_image, dst_image, resize_options)


def test_resize_pil(benchmark, resizer: Resizer, resize_alg, source_image, results: BenchResults):
    if source_image.mode != 'RGBA':
        source_image = source_image.convert('RGBA')
    dst_image = Image.new('RGBA', DST_SIZE)
    resize_options = ResizeOptions(resize_alg)

    def setup():
        set_image_mode(dst_image, 'RGBA')
        return (resizer, resize_options, source_image, dst_image), {}

    benchmark.pedantic(resize_pil, setup=setup, rounds=10, warmup_rounds=3)

    row_name = 'cykooz.resizer'
    if resizer.cpu_extensions != CpuExtensions.none:
        row_name += f' - {resizer.cpu_extensions.name}'

    alg = resize_alg.algorithm
    if alg == Algorithm.nearest:
        alg = 'nearest'
    else:
        alg = resize_alg.filter_type.name

    stats: Metadata = benchmark.stats
    value = stats.stats.mean * 1000
    results.add(row_name, alg, f'{value:.2f}')


# Pillow - U8


def test_resize_pillow_u8(benchmark, pil_filter, source_image, results: BenchResults):
    if source_image.mode != 'L':
        source_image = source_image.convert('L')

    def setup():
        src_image = source_image.copy()
        return (src_image, pil_filter), {}

    benchmark.pedantic(resize_pillow, setup=setup, rounds=50, warmup_rounds=3)

    alg = PIL_FILTERS[pil_filter]
    stats: Metadata = benchmark.stats
    value = stats.stats.mean * 1000
    results.add('Pillow U8', alg, f'{value:.2f}')


# cykooz.resizer - resize PIL U8 image

# def test_resize_pil_u8(benchmark, resize_alg, source_image, results: BenchResults):
#     resizer = Resizer(resize_alg)
#     resizer.cpu_extensions = CpuExtensions.none
#
#     if source_image.mode != 'L':
#         source_image = source_image.convert('L')
#     dst_image = Image.new('L', DST_SIZE)
#
#     def setup():
#         set_image_mode(dst_image, 'L')
#         return (resizer, source_image, dst_image), {}
#
#     benchmark.pedantic(resize_pil, setup=setup, rounds=10, warmup_rounds=3)
#
#     row_name = 'cykooz.resizer U8'
#
#     alg = resizer.algorithm.algorithm
#     if alg == Algorithm.nearest:
#         alg = 'nearest'
#     else:
#         alg = resizer.algorithm.filter_type.name
#
#     stats: Metadata = benchmark.stats
#     value = stats.stats.mean * 1000
#     results.add(row_name, alg, f'{value:.2f}')


def test_resize_pil_u8(
        benchmark, resizer: Resizer, resize_alg, source_image, results: BenchResults
):
    if source_image.mode != 'L':
        source_image = source_image.convert('L')
    dst_image = Image.new('L', DST_SIZE)
    resize_options = ResizeOptions(resize_alg)

    def setup():
        set_image_mode(dst_image, 'L')
        return (resizer, resize_options, source_image, dst_image), {}

    benchmark.pedantic(resize_pil, setup=setup, rounds=10, warmup_rounds=3)

    row_name = 'cykooz.resizer U8'
    if resizer.cpu_extensions != CpuExtensions.none:
        row_name += f' - {resizer.cpu_extensions.name}'

    alg = resize_alg.algorithm
    if alg == Algorithm.nearest:
        alg = 'nearest'
    else:
        alg = resize_alg.filter_type.name

    stats: Metadata = benchmark.stats
    value = stats.stats.mean * 1000
    results.add(row_name, alg, f'{value:.2f}')
