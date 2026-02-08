"""
:Authors: cykooz
:Date: 02.08.2021
"""
from typing import Optional


try:
    from PIL import Image as PilImage
except ImportError:
    PilImage = None

from .alpha import AlphaMulDiv, set_image_mode
from .rust_lib import PilImageWrapper, RustResizer
from .structs import Algorithm, CpuExtensions, ImageData, ResizeOptions


class Resizer:
    def __init__(self):
        self._rust_resizer = RustResizer()
        self._alpha_mul_div = AlphaMulDiv()

    @property
    def cpu_extensions(self) -> CpuExtensions:
        extensions: int = self._rust_resizer.get_cpu_extensions()
        return CpuExtensions(extensions)

    @cpu_extensions.setter
    def cpu_extensions(self, extensions: CpuExtensions):
        self._rust_resizer.set_cpu_extensions(extensions.value)
        self._alpha_mul_div.cpu_extensions = extensions

    def resize(
            self,
            src_image: ImageData,
            dst_image: ImageData,
            options: Optional[ResizeOptions] = None
    ):
        """Resize source image into size of destination image and store result
        into buffer of destination image.
        """
        self._rust_resizer.resize(
            src_image.rust_image,
            dst_image.rust_image,
            options.rust_options if options else None,
        )

    def resize_pil(
            self,
            src_image: 'PilImage.Image',
            dst_image: 'PilImage.Image',
            options: Optional[ResizeOptions] = None
    ):
        """Resize source image into size of destination image and store result
        into buffer of destination image.
        """
        src_image.load()
        src_mode = src_image.mode
        if src_mode not in ('RGB', 'RGBA', 'RGBa', 'CMYK', 'I', 'F', 'L'):
            raise ValueError(f'"{src_mode}" is unsupported mode of source PIL image')
        dst_mode = dst_image.mode

        if src_mode != dst_mode:
            if src_mode in ('CMYK', 'I', 'F', 'L') or dst_mode not in (
                    'RGB',
                    'RGBa',
                    'RGBA',
            ):
                src_image = self._convert(src_image, dst_mode)
                src_mode = src_image.mode

        options = options.copy() if options else ResizeOptions()

        if src_mode == 'RGBA' and dst_mode == 'RGBa':
            resize_alg = options.resize_alg
            if resize_alg.algorithm != Algorithm.nearest:
                src_image = self._alpha_mul_div.multiply_alpha_pil(
                    src_image,
                    options.thread_pool,
                )
                src_mode = 'RGBa'

        src_view = PilImageWrapper(src_image)
        set_image_mode(dst_image, src_image.mode)
        dst_view = PilImageWrapper(dst_image)

        if src_mode == 'RGBA':
            options.use_alpha = True
        else:
            options.use_alpha = False

        self._rust_resizer.resize_pil(
            src_view,
            dst_view,
            options.rust_options,
        )

        if src_mode == 'RGBa' and dst_mode == 'RGBA':
            self._alpha_mul_div.divide_alpha_pil_inplace(
                dst_image,
                options.thread_pool,
            )
        elif src_mode in ('RGBa', 'RGBA') and dst_mode == 'RGB':
            set_image_mode(dst_image, 'RGB')
        elif src_mode == 'RGB' and dst_mode in ('RGBa', 'RGBA'):
            set_image_mode(dst_image, dst_mode)

    def _convert(
            self,
            image: 'PilImage.Image',
            mode: str,
    ) -> 'PilImage.Image':
        img_mode = image.mode
        if img_mode == mode:
            return image

        if img_mode == 'RGB':
            if mode in ('RGBA', 'RGBa'):
                image = image.copy()
                set_image_mode(image, mode)
                return image
        elif img_mode == 'RGBa':
            if mode == 'RGBA':
                return image
            image = self._alpha_mul_div.divide_alpha_pil(image)

        if mode == 'RGBa':
            image = image.convert('RGB')
            set_image_mode(image, 'RGBa')
            return image

        if img_mode == 'CMYK' and mode in ('I', 'F'):
            image = image.convert('RGB')
        elif img_mode in ('I', 'F') and mode == 'CMYK':
            image = image.convert('RGB')

        return image.convert(mode)
