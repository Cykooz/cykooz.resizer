"""
:Authors: cykooz
:Date: 02.08.2021
"""
import dataclasses
from enum import Enum, unique
from typing import Optional, Tuple, Union

from .rust_lib import Image, RustResizeOptions, ResizerThreadPool


__all__ = (
    'Algorithm',
    'FilterType',
    'CpuExtensions',
    'PixelType',
    'ResizeAlg',
    'CropBox',
    'ResizerThreadPool',
    'ResizeOptions',
    'ImageData',
)


@unique
class Algorithm(Enum):
    """Resize algorithm.

    interpolation
        It is like `convolution` but with fixed kernel size.
        This algorithm can be useful if you want to get a result
        similar to `OpenCV` (except `INTER_AREA` interpolation).
    """
    nearest = 1
    convolution = 2
    interpolation = 3
    super_sampling = 4


@unique
class FilterType(Enum):
    box = 1
    bilinear = 2
    catmull_rom = 3
    mitchell = 4
    gaussian = 6
    lanczos3 = 5


@unique
class CpuExtensions(Enum):
    none = 1
    sse4_1 = 2
    avx2 = 3
    neon = 4


@unique
class PixelType(Enum):
    U8 = 1
    U8x2 = 2
    U8x3 = 3
    U8x4 = 4
    U16 = 5
    U16x2 = 6
    U16x3 = 7
    U16x4 = 8
    I32 = 9
    F32 = 10
    F32x2 = 11
    F32x3 = 12
    F32x4 = 13


PIXEL_SIZE = {
    PixelType.U8: 1,
    PixelType.U8x2: 2,
    PixelType.U8x3: 3,
    PixelType.U8x4: 4,
    PixelType.U16: 2,
    PixelType.U16x2: 4,
    PixelType.U16x3: 6,
    PixelType.U16x4: 8,
    PixelType.I32: 4,
    PixelType.F32: 4,
    PixelType.F32x2: 8,
    PixelType.F32x3: 12,
    PixelType.F32x4: 16,
}


class ResizeAlg:
    __slots__ = ('_algorithm', '_filter_type', '_multiplicity')

    def __init__(self):
        self._algorithm: Algorithm = Algorithm.nearest
        self._filter_type: Optional[FilterType] = None
        self._multiplicity: Optional[int] = None

    @classmethod
    def nearest(cls) -> 'ResizeAlg':
        return cls()

    @classmethod
    def convolution(cls, filter_type: FilterType) -> 'ResizeAlg':
        res = cls()
        res._algorithm = Algorithm.convolution
        res._filter_type = filter_type
        return res

    @classmethod
    def super_sampling(
            cls, filter_type: FilterType, multiplicity: int = 2
    ) -> 'ResizeAlg':
        if not isinstance(multiplicity, int) or 255 < multiplicity < 2:
            raise ValueError('"multiplicity" must be integer value in range [2, 255]')
        res = cls()
        res._algorithm = Algorithm.super_sampling
        res._filter_type = filter_type
        res._multiplicity = multiplicity
        return res

    @property
    def algorithm(self) -> Algorithm:
        return self._algorithm

    @property
    def filter_type(self) -> Optional[FilterType]:
        return self._filter_type

    @property
    def multiplicity(self) -> Optional[int]:
        return self._multiplicity

    def __eq__(self, other):
        if not isinstance(other, self.__class__):
            return False
        return (
                self._algorithm is other._algorithm
                and self._filter_type is other._filter_type
                and self._multiplicity == other._multiplicity
        )

    def __str__(self):
        return (
            f'<{self.__class__.__name__} '
            f'({self._algorithm}, {self._filter_type}, '
            f'{self._multiplicity})>'
        )


@dataclasses.dataclass(frozen=True)
class CropBox:
    left: float
    top: float
    width: float
    height: float

    def __post_init__(self):
        if self.left < 0 or self.top < 0:
            raise ValueError('"left" and "right" must be greater or equal to zero')
        if self.width < 0 or self.height < 0:
            raise ValueError('"width" and "height" must be greater or equal to zero')

    @classmethod
    def get_crop_box_to_fit_dst_size(
            cls,
            src_size: Tuple[int, int],
            dst_size: Tuple[int, int],
            centering=(0.5, 0.5),
    ) -> 'CropBox':
        """
        Returns a crop box to resize the source image into the
        aspect ratio of destination image size without distortions.

        :param src_size: The size of source image to crop.
        :param dst_size: The requested output size in pixels, given as a
                         (width, height) tuple.
        :param centering: Used to control the cropping position. Use
                          (0.5, 0.5) for center cropping (e.g. if cropping the
                          width, take 50% off of the left side, and therefore
                          50% off the right side). (0.0, 0.0) will crop from
                          the top left corner (i.e. if cropping the width,
                          take all of the crop off of the right side, and if
                          cropping the height, take all of it off the bottom).
                          (1.0, 0.0) will crop from the bottom left corner,
                          etc. (i.e. if cropping the width, take all of the crop
                          off the left side, and if cropping the height take
                          none from the top, and therefore all off the bottom).
        :return: A crop box for source image.

        This function based on code of ImageOps.fit() from Pillow package:
        https://github.com/python-pillow/Pillow/blob/master/src/PIL/ImageOps.py
        """
        # ensure centering is mutable
        centering = list(centering)

        if not 0.0 <= centering[0] <= 1.0:
            centering[0] = 0.5
        if not 0.0 <= centering[1] <= 1.0:
            centering[1] = 0.5

        # calculate the area to use for resizing and cropping, subtracting

        # calculate the aspect ratio of the source image
        src_ratio = float(src_size[0]) / src_size[1]

        # calculate the aspect ratio of the destination image
        dst_ratio = float(dst_size[0]) / dst_size[1]

        # figure out if the sides or top/bottom will be cropped off
        if src_ratio == dst_ratio:
            # live_size is already the needed ratio
            crop_width = src_size[0]
            crop_height = src_size[1]
        elif src_ratio >= dst_ratio:
            # live_size is wider than what's needed, crop the sides
            crop_width = dst_ratio * src_size[1]
            crop_height = src_size[1]
        else:
            # live_size is taller than what's needed, crop the top and bottom
            crop_width = src_size[0]
            crop_height = src_size[0] / dst_ratio

        # make the crop
        crop_left = (src_size[0] - crop_width) * centering[0]
        crop_top = (src_size[1] - crop_height) * centering[1]

        return cls(
            crop_left,
            crop_top,
            crop_width,
            crop_height,
        )


class ResizeOptions:

    def __init__(
            self,
            resize_alg: Optional[ResizeAlg] = None,
            use_alpha: Optional[bool] = None,
            crop_box: Optional[CropBox] = None,
            fit_into_destination: Union[bool, Tuple[float, float]] = False,
            thread_pool: Optional[ResizerThreadPool] = None,
    ):
        self.rust_options = RustResizeOptions()
        if resize_alg:
            self.resize_alg = resize_alg
        if crop_box:
            self.crop_box = crop_box
        if fit_into_destination:
            centering = fit_into_destination if isinstance(fit_into_destination, tuple) else None
            self.fit_into_destination(centering)
        if use_alpha is not None:
            self.use_alpha = use_alpha
        if thread_pool is not None:
            self.thread_pool = thread_pool

    def copy(self) -> 'ResizeOptions':
        copy = self.__class__()
        copy.rust_options = self.rust_options.copy()
        return copy

    @property
    def resize_alg(self) -> ResizeAlg:
        algorithm_v, filter_type_v, multiplicity = self.rust_options.get_resize_alg()
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

    @resize_alg.setter
    def resize_alg(self, alg: ResizeAlg):
        self.rust_options = self._change_resize_alg(alg)

    def _change_resize_alg(self, alg: ResizeAlg) -> RustResizeOptions:
        filter_type = alg.filter_type
        filter_type = filter_type.value if filter_type else 0
        multiplicity = alg.multiplicity
        multiplicity = multiplicity if multiplicity else 2
        return self.rust_options.set_resize_alg(
            alg.algorithm.value,
            filter_type,
            multiplicity,
        )

    @property
    def crop_box(self) -> Optional[CropBox]:
        """Get crop box for source image."""
        box = self.rust_options.get_crop_box()
        if box:
            return CropBox(*box)

    @crop_box.setter
    def crop_box(self, value: CropBox):
        """Set crop box for source image."""
        self.rust_options = self.rust_options.set_crop_box(
            value.left,
            value.top,
            value.width,
            value.height,
        )

    def fit_into_destination(
            self,
            centering: Optional[Tuple[float, float]] = None,
    ):
        """Fit source image into the aspect ratio of destination
        image without distortions."""
        self.rust_options = self.rust_options.set_fit_into_destination(centering)

    def get_fit_into_destination_centering(self) -> Optional[Tuple[float, float]]:
        return self.rust_options.get_fit_into_destination_centering()

    @property
    def use_alpha(self) -> bool:
        return self.rust_options.get_use_alpha()

    @use_alpha.setter
    def use_alpha(self, value: bool):
        """Enable or disable consideration of the alpha channel when resizing."""
        self.rust_options = self.rust_options.set_use_alpha(value)

    @property
    def thread_pool(self) -> Optional[ResizerThreadPool]:
        return self.rust_options.get_thread_pool()

    @thread_pool.setter
    def thread_pool(self, thread_pool: Optional[ResizerThreadPool]):
        self.rust_options = self.rust_options.set_thread_pool(thread_pool)


class ImageData:
    __slots__ = ('rust_image',)

    def __init__(
            self,
            width: int,
            height: int,
            pixel_type: PixelType,
            pixels: Optional[bytes] = None,
    ):
        if width < 0 or height < 0:
            raise ValueError('"width" and "height" must be greater ot equal to zero')
        if pixels:
            pixel_size = PIXEL_SIZE[pixel_type]
            min_size = width * height * pixel_size
            if len(pixels) < min_size:
                raise ValueError(
                    f'Size of "pixels" must be greater or equal to {min_size} bytes'
                )
        self.rust_image = Image(width, height, pixel_type.value, pixels)

    @property
    def width(self) -> int:
        return self.rust_image.width()

    @property
    def height(self) -> int:
        return self.rust_image.height()

    def get_buffer(self) -> bytes:
        """Returns copy of internal buffer with pixels"""
        return self.rust_image.buffer()
