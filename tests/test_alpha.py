"""
:Authors: cykooz
:Date: 12.08.2021
"""
from pathlib import Path

import pytest
from PIL import Image

from cykooz.resizer import AlphaMulDiv, CpuExtensions

from utils import Checksum, get_image_checksum


@pytest.mark.parametrize(
    ('cpu_extensions', 'checksum'),
    [
        (CpuExtensions.none, Checksum(1091845751, 1090022383, 1061212976, 2282335752)),
        (CpuExtensions.sse2, Checksum(1091845751, 1090022383, 1061212976, 2282335752)),
        (CpuExtensions.sse4_1, Checksum(1091845751, 1090022383, 1061212976, 2282335752)),
        (CpuExtensions.avx2, Checksum(1091739260, 1089908614, 1061097954, 2282335752)),
    ]
)
@pytest.mark.parametrize(
    ('inplace',),
    [
        (False,),
        (True,),
    ],
    ids=['not inplace', 'inplace']
)
def test_multiply_alpha_pil(inplace: bool, cpu_extensions: CpuExtensions, checksum: int):
    mul_div = AlphaMulDiv()
    mul_div.cpu_extensions = cpu_extensions

    data_dir = Path(__file__).parent / 'data'
    img_path = data_dir / 'nasa-4928x3279.png'
    image: Image.Image = Image.open(img_path)
    if image.mode != 'RGBA':
        image = image.convert('RGBA')
    assert get_image_checksum(image.tobytes('raw')) == Checksum(
        1095901781, 1098442059, 1075159669, 2282335752
    )

    if inplace:
        mul_div.multiply_alpha_pil_inplace(image)
        res_image = image
        dir_name = 'inplace'
    else:
        res_image = mul_div.multiply_alpha_pil(image)
        dir_name = 'not_inplace'

    assert get_image_checksum(res_image.tobytes('raw')) == checksum

    result_dir = data_dir / 'result' / 'alpha_mul' / 'pil' / dir_name
    result_dir.mkdir(parents=True, exist_ok=True)
    file_name = f'nasa-multiply-{cpu_extensions.name}.png'
    dst_path = result_dir / file_name
    res_image.mode = 'RGB'
    res_image.save(dst_path)


@pytest.mark.parametrize(
    ('cpu_extensions', 'checksum'),
    [
        (CpuExtensions.none, Checksum(1093603374, 1091538008, 1062526277, 2282335752)),
        (CpuExtensions.sse2, Checksum(1093597720, 1091533607, 1062522217, 2282335752)),
        (CpuExtensions.sse4_1, Checksum(1093597720, 1091533607, 1062522217, 2282335752)),
        (CpuExtensions.avx2, Checksum(1093597720, 1091533607, 1062522217, 2282335752)),
    ]
)
@pytest.mark.parametrize(
    ('inplace',),
    [
        (False,),
        (True,),
    ],
    ids=['not inplace', 'inplace']
)
def test_divide_alpha_pil(inplace: bool, cpu_extensions: CpuExtensions, checksum: int):
    mul_div = AlphaMulDiv()
    mul_div.cpu_extensions = cpu_extensions

    data_dir = Path(__file__).parent / 'data'
    img_path = data_dir / 'nasa-4928x3279.png'
    image: Image.Image = Image.open(img_path)
    if image.mode != 'RGBa':
        image = image.convert('RGBa')
    assert get_image_checksum(image.tobytes('raw')) == Checksum(
        1091845751, 1090022383, 1061212976, 2282335752
    )

    if inplace:
        mul_div.divide_alpha_pil_inplace(image)
        res_image = image
        dir_name = 'inplace'
    else:
        res_image = mul_div.divide_alpha_pil(image)
        dir_name = 'not_inplace'

    assert res_image.mode == 'RGBA'
    assert get_image_checksum(res_image.tobytes('raw')) == checksum

    result_dir = data_dir / 'result' / 'alpha_div' / 'pil' / dir_name
    result_dir.mkdir(parents=True, exist_ok=True)
    file_name = f'nasa-multiply-{cpu_extensions.name}.png'
    dst_path = result_dir / file_name
    res_image.save(dst_path)
