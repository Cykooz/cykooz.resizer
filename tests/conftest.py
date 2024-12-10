"""
:Authors: cykooz
:Date: 22.08.2021
"""
from pathlib import Path

import pytest
from PIL import Image

from cykooz.resizer import ResizerThreadPool


@pytest.fixture(name='source_image', scope='session')
def source_image_fixture() -> Image.Image:
    data_dir = Path(__file__).parent / 'data'
    img_path = data_dir / 'nasa-4928x3279.png'
    image: Image.Image = Image.open(img_path)
    if image.mode != 'RGBA':
        image = image.convert('RGBA')
    return image


@pytest.fixture(
    name='thread_pool',
    params=[
        False,
        True,
    ],
    ids=[
        'single_thread',
        'multi_thread',
    ],
    scope='session',
)
def thread_pool_fixture(request):
    if request.param:
        return ResizerThreadPool(num_threads=6)
