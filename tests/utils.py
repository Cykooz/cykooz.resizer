"""
:Authors: cykooz
:Date: 12.08.2021
"""
import dataclasses
import os
from collections import defaultdict
from pathlib import Path

from PIL import Image
from tabulate import tabulate


@dataclasses.dataclass(frozen=True)
class Checksum:
    c1: int = 0
    c2: int = 0
    c3: int = 0
    c4: int = 0

    def __repr__(self):
        return f'{self.__class__.__name__}({self.c1}, {self.c2}, {self.c3}, {self.c4})'


def get_image_checksum(buffer: bytes):
    c1 = sum(buffer[0::4])
    c2 = sum(buffer[1::4])
    c3 = sum(buffer[2::4])
    c4 = sum(buffer[3::4])
    return Checksum(c1, c2, c3, c4)


class BenchResults:
    def __init__(self):
        self.columns = []
        self.rows = defaultdict(dict)

    def add(self, row_name, column_name, value):
        self.rows[row_name][column_name] = value
        if column_name not in self.columns:
            self.columns.append(column_name)

    def print_table(self):
        headers = ['Package (time in ms)'] + self.columns
        table = []
        for row_name, columns in self.rows.items():
            row = [row_name]
            for c_name in self.columns:
                row.append(columns.get(c_name, ''))
            table.append(row)
        print(
            tabulate(
                table,
                headers=headers,
                tablefmt='pipe',
                disable_numparse=True,
                colalign=['left'] + ['right'] * len(self.columns),
            )
        )


def save_result(image: Image.Image, rel_path: Path, file_name):
    if not os.environ.get('DONT_SAVE_RESULT', ''):
        data_dir = Path(__file__).parent / 'data'
        result_dir = data_dir / 'result' / rel_path
        result_dir.mkdir(parents=True, exist_ok=True)
        if image.mode == 'RGBa':
            image = image.convert('RGBA')
        elif image.mode in ('I', 'F'):
            image = image.convert('RGB')
        elif image.mode in ('CMYK',):
            file_name = file_name.replace('.png', '.tiff')
        dst_path = result_dir / file_name
        image.save(dst_path)
