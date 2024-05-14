"""
:Authors: cykooz
:Date: 12.08.2021
"""
import weakref

from PIL import Image

from cykooz.resizer.rust_lib import PilImageWrapper


def test_pillow_image_view_gc():
    image = Image.new('RGBA', (256, 256))
    image_ref = weakref.ref(image)
    assert image_ref() is not None
    del image
    assert image_ref() is None

    image = Image.new('RGBA', (256, 256))
    image_ref = weakref.ref(image)
    assert image_ref() is not None
    _image_view = PilImageWrapper(image)
    assert image_ref() is not None
    del image
    assert image_ref() is not None
    del _image_view
    assert image_ref() is None
