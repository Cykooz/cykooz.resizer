"""
:Authors: cykooz
:Date: 12.08.2021
"""
from pathlib import Path

import pytest
from PIL import Image

from cykooz.resizer import AlphaMulDiv, CpuExtensions

from utils import Checksum, get_image_checksum, save_result


@pytest.mark.parametrize(
    ('cpu_extensions', 'checksum'),
    [
        (CpuExtensions.none, Checksum(1091845751, 1090022383, 1061212976, 2282335752)),
        (
            CpuExtensions.sse4_1,
            Checksum(1091845751, 1090022383, 1061212976, 2282335752),
        ),
        (CpuExtensions.avx2, Checksum(1091845751, 1090022383, 1061212976, 2282335752)),
    ],
    ids=[
        'wo forced SIMD',
        'sse4.1',
        'avx2',
    ],
)
@pytest.mark.parametrize(
    ('inplace',),
    [
        (False,),
        (True,),
    ],
    ids=['not inplace', 'inplace'],
)
def test_multiply_alpha_pil(
    source_image: Image.Image,
    inplace: bool,
    cpu_extensions: CpuExtensions,
    checksum: int,
):
    mul_div = AlphaMulDiv()
    if (
        cpu_extensions == CpuExtensions.avx2
        and mul_div.cpu_extensions != CpuExtensions.avx2
    ):
        raise pytest.skip('AVX2 instruction not supported by CPU')
    mul_div.cpu_extensions = cpu_extensions

    image = source_image.copy()
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

    save_result(
        res_image,
        Path('alpha_mul') / 'pil' / dir_name,
        f'nasa-multiply-{cpu_extensions.name}.png',
    )


@pytest.mark.parametrize(
    ('cpu_extensions', 'checksum'),
    [
        (CpuExtensions.none, Checksum(1093599367, 1091535535, 1062524230, 2282335752)),
        (
            CpuExtensions.sse4_1,
            Checksum(1093599367, 1091535535, 1062524230, 2282335752),
        ),
        (CpuExtensions.avx2, Checksum(1093599367, 1091535535, 1062524230, 2282335752)),
    ],
    ids=[
        'wo forced SIMD',
        'sse4.1',
        'avx2',
    ],
)
@pytest.mark.parametrize(
    ('inplace',),
    [
        (False,),
        (True,),
    ],
    ids=['not inplace', 'inplace'],
)
def test_divide_alpha_pil(
    source_image: Image.Image,
    inplace: bool,
    cpu_extensions: CpuExtensions,
    checksum: int,
):
    mul_div = AlphaMulDiv()
    if (
        cpu_extensions == CpuExtensions.avx2
        and mul_div.cpu_extensions != CpuExtensions.avx2
    ):
        raise pytest.skip('AVX2 instruction not supported by CPU')
    mul_div.cpu_extensions = cpu_extensions

    image = source_image.copy()
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

    save_result(
        res_image,
        Path('alpha_mul') / 'pil' / dir_name,
        f'nasa-multiply-{cpu_extensions.name}.png',
    )
