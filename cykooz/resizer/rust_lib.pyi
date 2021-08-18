""" This module is a python module implemented in Rust. """
from typing import Optional, Tuple

from PIL import Image


class ImageView:

    def __init__(
            self,
            width: int,
            height: int,
            pixel_type: int,
            buffer: Optional[bytes],
    ):
        ...

    def set_crop_box(self, left: int, top: int, width: int, height: int):
        ...

    def width(self) -> int:
        ...

    def height(self) -> int:
        ...

    def buffer(self) -> bytes:
        ...


class PilImageView:

    def __init__(self, image: Image.Image):
        ...

    def set_crop_box(self, left: int, top: int, width: int, height: int):
        ...

    @property
    def pil_image(self) -> Optional[Image.Image]:
        ...

class RustAlphaMulDiv:

    def __init__(self):
        ...

    def get_cpu_extensions(self) -> int:
        """Returns CPU extensions."""
        ...

    def set_cpu_extensions(self, extensions: int):
        """Set CPU extensions."""
        ...

    def divide_alpha(self, src_image: ImageView, dst_image: ImageView):
        """
        Divides RGB-channels of source image by alpha-channel and store
        result into destination image.
        """
        ...

    def divide_alpha_inplace(self, image: ImageView):
        """Divides RGB-channels of image by alpha-channel inplace."""
        ...

    def divide_alpha_pil(self, src_image: PilImageView, dst_image: PilImageView):
        """
        Divides RGB-channels of source image by alpha-channel and store
        result into destination image.
        """
        ...

    def divide_alpha_pil_inplace(self, image: PilImageView):
        """Divides RGB-channels of image by alpha-channel inplace."""
        ...

    def multiply_alpha(self, src_image: ImageView, dst_image: ImageView):
        """
        Multiplies RGB-channels of source image by alpha-channel and store
        result into destination image.
        """
        ...

    def multiply_alpha_inplace(self, image: ImageView):
        """Multiplies RGB-channels of image by alpha-channel inplace."""
        ...

    def multiply_alpha_pil(self, src_image: PilImageView, dst_image: PilImageView):
        """
        Multiplies RGB-channels of source image by alpha-channel and store
        result into destination image.
        """
        ...

    def multiply_alpha_pil_inplace(self, image: PilImageView):
        """Multiplies RGB-channels of image by alpha-channel inplace."""
        ...


class RustResizer:

    def __init__(self, algorithm: int, filter_type: int, multiplicity: int):
        ...

    def get_algorithm(self) -> Tuple[int, int, int]:
        """Returns resize algorithm."""
        ...

    def set_algorithm(self, algorithm: int, filter_type: int, multiplicity: int):
        """ Set resize algorithm. """
        ...

    def get_cpu_extensions(self) -> int:
        """Returns CPU extensions."""
        ...

    def set_cpu_extensions(self, extensions: int):
        """Set CPU extensions."""
        ...

    def resize(self, src_image: ImageView, dst_image: ImageView):
        """Resize source image into destination image."""
        ...

    def resize_pil(self, src_image: PilImageView, dst_image: PilImageView):
        """Resize source image into destination image."""
        ...


# variables with complex values

__all__ = [
    'ImageView',
    'PilImageView',
    'RustResizer',
    'RustAlphaMulDiv',
]
