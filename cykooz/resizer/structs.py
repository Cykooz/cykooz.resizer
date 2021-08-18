"""
:Authors: cykooz
:Date: 02.08.2021
"""
import dataclasses
from enum import Enum, unique
from typing import Optional

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
        if self.left <= 0 or self.top <= 0:
            raise ValueError('"left" and "right" must be greater or equal to zero')
        if self.width <= 0 or self.height <= 0:
            raise ValueError('"width" and "height" must be greater than zero')


class ImageData:
    __slots__ = ('image_view',)

    def __init__(self, width: int, height: int, pixel_type: PixelType, pixels: Optional[bytes] = None):
        if width <= 0 or height <= 0:
            raise ValueError('"width" and "height" must be greater than zero')
        if pixels:
            min_size = width * height * 4
            if len(pixels) < min_size:
                raise ValueError(f'Size of "pixels" must be greater or equal to {min_size} bytes')
        self.image_view = ImageView(width, height, pixel_type.value, pixels)

    @property
    def width(self) -> int:
        return self.image_view.width()

    @property
    def height(self) -> int:
        return self.image_view.height()

    def get_buffer(self) -> bytes:
        """Returns copy of internal buffer of pixels"""
        return self.image_view.buffer()
