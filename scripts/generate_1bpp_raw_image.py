from pathlib import Path
from typing import Iterable, cast

from PIL import Image

im = Image.open("resources/weather_bg_indexed.bmp")

# Ensure paletted / 1-bit
if im.mode not in ("1", "P"):
    im = im.convert("1")

width, height = im.size
pixels = list(cast(Iterable[int], im.getdata()))  # 0 or 1

raw = bytearray()

for i in range(0, len(pixels), 8):
    b = 0
    for j in range(8):
        if i + j < len(pixels):
            b |= (pixels[i + j] & 1) << (7 - j)
    raw.append(b)

Path("resources/weather_bg_1b.raw").write_bytes(raw)

print(f"Wrote {len(raw)} bytes")
