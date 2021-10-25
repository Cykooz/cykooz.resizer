"""
:Authors: cykooz
:Date: 02.08.2021
"""
from typing import Optional

try:
    from PIL import Image as PilImage
except ImportError:
    PilImage = None

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
        """Resize source image into size of destination image and store result
        into buffer of destination image.
        """
        self._rust_resizer.resize(src_image.rust_image, dst_image.rust_image)

    def resize_pil(
            self,
            src_image: 'PilImage.Image',
            dst_image: 'PilImage.Image',
            crop_box: Optional[CropBox] = None,
    ):
        """Resize source image into size of destination image and store result
        into buffer of destination image.
        """
        src_mode = src_image.mode
        if src_mode not in ('RGB', 'RGBA', 'RGBa', 'CMYK', 'I', 'F', 'L'):
            raise ValueError(f'"{src_mode}" is unsupported mode of source PIL image')
        dst_mode = dst_image.mode
        orig_src_image = src_image

        if src_mode != dst_mode:
            if src_mode in ('CMYK', 'I', 'F', 'L') or dst_mode not in ('RGB', 'RGBa', 'RGBA'):
                src_image = self._convert(
                    src_image,
                    dst_mode,
                    in_place=orig_src_image is not src_image
                )
                src_mode = src_image.mode

        if self.algorithm.algorithm != Algorithm.nearest and src_mode == 'RGBA':
            src_image = self._alpha_mul_div.multiply_alpha_pil(src_image)
            src_mode = 'RGBa'

        src_view = PilImageView(src_image)
        if crop_box:
            src_view.set_crop_box(
                crop_box.left,
                crop_box.top,
                crop_box.width,
                crop_box.height,
            )
        dst_image.mode = src_image.mode
        dst_view = PilImageView(dst_image)

        self._rust_resizer.resize_pil(src_view, dst_view)

        if src_mode == 'RGBa' and dst_mode == 'RGBA':
            self._alpha_mul_div.divide_alpha_pil_inplace(dst_image)
        elif src_mode == 'RGBA' and dst_mode == 'RGBa':
            self._alpha_mul_div.multiply_alpha_pil_inplace(dst_image)
        elif src_mode in ('RGBa', 'RGBA') and dst_mode == 'RGB':
            dst_image.mode = 'RGB'
        elif src_mode == 'RGB' and dst_mode in ('RGBa', 'RGBA'):
            dst_image.mode = dst_mode

    def _convert(self, image: 'PilImage.Image', mode: str, in_place=False) -> 'PilImage.Image':
        img_mode = image.mode
        if img_mode == mode:
            return image

        if img_mode == 'RGB':
            if mode in ('RGBA', 'RGBa'):
                if not in_place:
                    image = image.copy()
                image.mode = mode
                return image
        elif img_mode == 'RGBa':
            if mode == 'RGBA':
                return image
            image = self._alpha_mul_div.divide_alpha_pil(image)

        if mode == 'RGBa':
            image = image.convert('RGB')
            image.mode = 'RGBa'
            return image

        if img_mode == 'CMYK' and mode in ('I', 'F'):
            image = image.convert('RGB')
        elif img_mode in ('I', 'F') and mode == 'CMYK':
            image = image.convert('RGB')

        return image.convert(mode)
