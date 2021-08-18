"""
:Authors: cykooz
:Date: 02.08.2021
"""
from typing import Optional

try:
    from PIL import Image
except ImportError:
    Image = None

from .alpha import AlphaMulDiv
from .rust_lib import PilImageView, RustResizer
from .structs import Algorithm, CpuExtensions, CropBox, FilterType, ResizeAlg, ImageData


class Resizer:

    def __init__(self, resize_alg: ResizeAlg):
        filter_type = resize_alg.filter_type
        filter_type = filter_type.value if filter_type else 0
        multiplicity = resize_alg.multiplicity
        multiplicity = multiplicity if multiplicity else 2
        self._rust_resizer = RustResizer(
            resize_alg.algorithm.value,
            filter_type,
            multiplicity
        )
        self._alpha_mul_div = AlphaMulDiv()

    @property
    def algorithm(self) -> ResizeAlg:
        algorithm_v, filter_type_v, multiplicity = self._rust_resizer.get_algorithm()
        try:
            algorithm = Algorithm(algorithm_v)
            if algorithm is Algorithm.nearest:
                return ResizeAlg.nearest()
            elif algorithm is Algorithm.convolution:
                filter_type = FilterType(filter_type_v)
                return ResizeAlg.convolution(filter_type)
            elif algorithm is Algorithm.super_sampling:
                filter_type = FilterType(filter_type_v)
                return ResizeAlg.super_sampling(filter_type, multiplicity)
        except ValueError:
            pass
        raise RuntimeError(
            f'Unknown resize algorithm parameters '
            f'("{algorithm_v}", "{filter_type_v}", {multiplicity})'
        )

    @algorithm.setter
    def algorithm(self, resize_alg: ResizeAlg):
        filter_type = resize_alg.filter_type
        filter_type = filter_type.value if filter_type else ''
        multiplicity = resize_alg.multiplicity
        multiplicity = multiplicity if multiplicity else 2
        self._rust_resizer.set_algorithm(
            resize_alg.algorithm.value,
            filter_type,
            multiplicity
        )

    @property
    def cpu_extensions(self) -> CpuExtensions:
        extensions: int = self._rust_resizer.get_cpu_extensions()
        return CpuExtensions(extensions)

    @cpu_extensions.setter
    def cpu_extensions(self, extensions: CpuExtensions):
        self._rust_resizer.set_cpu_extensions(extensions.value)
        self._alpha_mul_div.cpu_extensions = extensions

    def resize(self, src_image: ImageData, dst_image: ImageData):
        self._rust_resizer.resize(src_image.image_view, dst_image.image_view)

    def resize_pil(
            self,
            src_image: Image.Image,
            dst_image: Image.Image,
            crop_box: Optional[CropBox] = None,
    ):
        if dst_image.mode not in ('RGBA', 'RGBa', 'RGB'):
            raise ValueError('not supported mode of dst_image')

        src_mode = src_image.mode
        if self.algorithm.algorithm != Algorithm.nearest:
            if src_mode == 'RGBA':
                src_image = self._alpha_mul_div.multiply_alpha_pil(src_image)
            elif src_mode == 'RGB':
                pass
            elif src_mode != 'RGBa':
                src_image = src_image.convert('RGBa')

        src_view = PilImageView(src_image)
        if crop_box:
            src_view.set_crop_box(
                crop_box.left,
                crop_box.top,
                crop_box.width,
                crop_box.height,
            )
        dst_view = PilImageView(dst_image)

        self._rust_resizer.resize_pil(src_view, dst_view)

        if self.algorithm.algorithm != Algorithm.nearest:
            if dst_image.mode == 'RGBA' and src_mode in ('RGBA', 'RGBa'):
                self._alpha_mul_div.divide_alpha_pil_inplace(dst_image)
