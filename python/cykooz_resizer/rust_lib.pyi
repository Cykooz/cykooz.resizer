""" This module is the python module implemented in Rust. """
from typing import Optional, Tuple

from PIL import Image as PilImage


class Image:
    def __init__(
            self,
            width: int,
            height: int,
            pixel_type: int,
            buffer: Optional[bytes],
    ): ...

    def width(self) -> int: ...

    def height(self) -> int: ...

    def buffer(self) -> bytes: ...


class PilImageWrapper:
    def __init__(self, image: PilImage.Image): ...

    @property
    def pil_image(self) -> Optional[PilImage.Image]: ...


class ResizerThreadPool:
    def __init__(self, num_threads: Optional[int] = None):
        ...

    @property
    def current_num_threads(self) -> int:
        ...


class RustAlphaMulDiv:
    def __init__(self): ...

    def get_cpu_extensions(self) -> int:
        """Returns CPU extensions."""
        ...

    def set_cpu_extensions(self, extensions: int):
        """Set CPU extensions."""
        ...

    def divide_alpha(
            self,
            src_image: Image,
            dst_image: Image,
            thread_pool: Optional[ResizerThreadPool] = None,
    ):
        """
        Divides RGB-channels of the source image by alpha-channel and store
        a result into the destination image.
        """
        ...

    def divide_alpha_inplace(
            self,
            image: Image,
            thread_pool: Optional[ResizerThreadPool] = None,
    ):
        """Divides RGB-channels of the image by alpha-channel inplace."""
        ...

    def divide_alpha_pil(
            self,
            src_image: PilImageWrapper,
            dst_image: PilImageWrapper,
            thread_pool: Optional[ResizerThreadPool] = None,
    ):
        """
        Divides RGB-channels of the source image by alpha-channel and store
        a result into the destination image.
        """
        ...

    def divide_alpha_pil_inplace(
            self,
            image: PilImageWrapper,
            thread_pool: Optional[ResizerThreadPool] = None,
    ):
        """Divides RGB-channels of the image by alpha-channel inplace."""
        ...

    def multiply_alpha(
            self,
            src_image: Image,
            dst_image: Image,
            thread_pool: Optional[ResizerThreadPool] = None,
    ):
        """
        Multiplies RGB-channels of the source image by alpha-channel and store
        a result into the destination image.
        """
        ...

    def multiply_alpha_inplace(
            self,
            image: Image,
            thread_pool: Optional[ResizerThreadPool] = None,
    ):
        """Multiplies RGB-channels of the image by alpha-channel inplace."""
        ...

    def multiply_alpha_pil(
            self,
            src_image: PilImageWrapper,
            dst_image: PilImageWrapper,
            thread_pool: Optional[ResizerThreadPool] = None,
    ):
        """
        Multiplies RGB-channels of the source image by alpha-channel and store
        a result into the destination image.
        """
        ...

    def multiply_alpha_pil_inplace(
            self,
            image: PilImageWrapper,
            thread_pool: Optional[ResizerThreadPool] = None,
    ):
        """Multiplies RGB-channels of the image by alpha-channel inplace."""
        ...


class RustResizeOptions:
    def __init__(self): ...

    def copy(self) -> 'RustResizeOptions':
        ...

    def get_resize_alg(self) -> Tuple[int, int, int]:
        """Returns resize algorithm."""
        ...

    def set_resize_alg(
            self,
            algorithm: int,
            filter_type: int,
            multiplicity: int,
    ) -> 'RustResizeOptions':
        """Set resize algorithm."""
        ...

    def get_crop_box(self) -> Optional[Tuple[float, float, float, float]]:
        """Get a crop box."""
        ...

    def set_crop_box(
            self,
            left: float,
            top: float,
            width: float,
            height: float,
    ) -> 'RustResizeOptions':
        """Set crop box for source image."""
        ...

    def get_fit_into_destination_centering(self) -> Optional[Tuple[float, float]]:
        """Get centering parameter of fitting source image into
        the aspect ratio of destination"""
        ...

    def set_fit_into_destination(
            self,
            centering: Optional[Tuple[float, float]] = None,
    ) -> 'RustResizeOptions':
        """Fit the source image into the aspect ratio of the destination
        image without distortions."""
        ...

    def get_use_alpha(self) -> bool:
        ...

    def set_use_alpha(self, v: bool) -> 'RustResizeOptions':
        """Enable or disable consideration of the alpha channel when resizing."""
        ...

    def get_thread_pool(self) -> Optional[ResizerThreadPool]:
        ...

    def set_thread_pool(self, thread_pool: Optional[ResizerThreadPool]) -> 'RustResizeOptions':
        ...


class RustResizer:
    def __init__(self): ...

    def get_cpu_extensions(self) -> int:
        """Returns CPU extensions."""
        ...

    def set_cpu_extensions(self, extensions: int):
        """Set CPU extensions."""
        ...

    def resize(
            self,
            src_image: Image,
            dst_image: Image,
            options: Optional[RustResizeOptions] = None,
    ):
        """Resize source image into destination image."""
        ...

    def resize_pil(
            self,
            src_image: PilImageWrapper,
            dst_image: PilImageWrapper,
            options: Optional[RustResizeOptions] = None,
    ):
        """Resize source image into destination image."""
        ...


# variables with complex values

__all__ = [
    'Image',
    'PilImageWrapper',
    'ResizerThreadPool',
    'RustResizeOptions',
    'RustResizer',
    'RustAlphaMulDiv',
]
