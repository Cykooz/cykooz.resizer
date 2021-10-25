"""
:Authors: cykooz
:Date: 02.08.2021
"""
import dataclasses
from enum import Enum, unique
from typing import Optional, Tuple

from .rust_lib import ImageView


__all__ = (
    'Algorithm',
    'FilterType',
    'CpuExtensions',
    'PixelType',
    'ResizeAlg',
    'CropBox',
    'ImageData',
)


@unique
class Algorithm(Enum):
    nearest = 1
    convolution = 2
    super_sampling = 3


@unique
class FilterType(Enum):
    box = 1
    bilinear = 2
    catmull_rom = 3
    mitchell = 4
    lanczos3 = 5


@unique
class CpuExtensions(Enum):
    none = 1
    sse2 = 2
    sse4_1 = 3
    avx2 = 4


@unique
class PixelType(Enum):
    U8x4 = 1
    I32 = 2
    F32 = 3
    U8 = 4


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
    def super_sampling(cls, filter_type: FilterType, multiplicity: int = 2) -> 'ResizeAlg':
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
    left: int
    top: int
    width: int
    height: int

    def __post_init__(self):
        if self.left < 0 or self.top < 0:
            raise ValueError('"left" and "right" must be greater or equal to zero')
        if self.width <= 0 or self.height <= 0:
            raise ValueError('"width" and "height" must be greater than zero')

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
            int(round(crop_left)),
            int(round(crop_top)),
            int(round(crop_width)),
            int(round(crop_height)),
        )


class ImageData:
    __slots__ = ('rust_image',)

    def __init__(self, width: int, height: int, pixel_type: PixelType, pixels: Optional[bytes] = None):
        if width <= 0 or height <= 0:
            raise ValueError('"width" and "height" must be greater than zero')
        if pixels:
            pixel_size = 1 if pixel_type == PixelType.U8 else 4
            min_size = width * height * pixel_size
            if len(pixels) < min_size:
                raise ValueError(f'Size of "pixels" must be greater or equal to {min_size} bytes')
        self.rust_image = ImageView(width, height, pixel_type.value, pixels)

    @property
    def width(self) -> int:
        return self.rust_image.width()

    @property
    def height(self) -> int:
        return self.rust_image.height()

    def get_buffer(self) -> bytes:
        """Returns copy of internal buffer of pixels"""
        return self.rust_image.buffer()
