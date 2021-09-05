"""
:Authors: cykooz
:Date: 02.08.2021
"""
try:
    from PIL import Image
except ImportError:
    Image = None

from .rust_lib import PilImageView, RustAlphaMulDiv
from .structs import CpuExtensions, ImageData


class AlphaMulDiv:

    def __init__(self):
        self._rust_alpha_mul_div = RustAlphaMulDiv()

    @property
    def cpu_extensions(self):
        extensions: int = self._rust_alpha_mul_div.get_cpu_extensions()
        return CpuExtensions(extensions)

    @cpu_extensions.setter
    def cpu_extensions(self, extensions: CpuExtensions):
        self._rust_alpha_mul_div.set_cpu_extensions(extensions.value)

    def multiply_alpha(self, src_image: ImageData, dst_image: ImageData):
        self._rust_alpha_mul_div.multiply_alpha(
            src_image.image_view,
            dst_image.image_view,
        )

    def multiply_alpha_inplace(self, image: ImageData):
        self._rust_alpha_mul_div.multiply_alpha_inplace(image.image_view)

    def divide_alpha(self, src_image: ImageData, dst_image: ImageData):
        self._rust_alpha_mul_div.divide_alpha(
            src_image.image_view,
            dst_image.image_view,
        )

    def divide_alpha_inplace(self, image: ImageData):
        self._rust_alpha_mul_div.divide_alpha_inplace(image.image_view)

    def multiply_alpha_pil(self, image: 'Image.Image') -> 'Image.Image':
        if image.mode == 'RGBa':
            return image.copy()
        elif image.mode != 'RGBA':
            raise ValueError('Unsupported mode of source image.')

        src_view = PilImageView(image)
        dst_img = Image.new('RGBa', image.size)
        dst_view = PilImageView(dst_img)
        self._rust_alpha_mul_div.multiply_alpha_pil(src_view, dst_view)
        return dst_img

    def multiply_alpha_pil_inplace(self, image: 'Image.Image'):
        if image.mode == 'RGBa':
            return
        elif image.mode != 'RGBA':
            raise ValueError('Unsupported mode of source image.')
        if image.readonly:
            image._copy()
        image_view = PilImageView(image)
        self._rust_alpha_mul_div.multiply_alpha_pil_inplace(image_view)
        image.mode = 'RGBa'

    def divide_alpha_pil(self, image: 'Image.Image') -> 'Image.Image':
        if image.mode == 'RGBA':
            return image.copy()
        elif image.mode != 'RGBa':
            raise ValueError('Unsupported mode of source image.')
        src_view = PilImageView(image)
        dst_img = Image.new('RGBA', image.size)
        dst_view = PilImageView(dst_img)
        self._rust_alpha_mul_div.divide_alpha_pil(src_view, dst_view)
        return dst_img

    def divide_alpha_pil_inplace(self, image: 'Image.Image'):
        if image.mode == 'RGBA':
            return
        elif image.mode != 'RGBa':
            raise ValueError('Unsupported mode of source image.')
        if image.readonly:
            image._copy()
        image_view = PilImageView(image)
        self._rust_alpha_mul_div.divide_alpha_pil_inplace(image_view)
        image.mode = 'RGBA'
