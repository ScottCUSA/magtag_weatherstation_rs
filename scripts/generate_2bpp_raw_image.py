import math

import numpy as np
from PIL import Image

im = Image.open("resources/weather_icons_20px_60x60_4b.bmp")
pixels = np.array(im)  # shape: (height, width), palette indices

# swap grey and dark grey pixels in the output image
conditions = [pixels == 1, pixels == 2]
choices = [np.array(2, dtype=pixels.dtype), np.array(1, dtype=pixels.dtype)]
pixels = np.select(conditions, choices, default=pixels)

height, width = pixels.shape

# Number of pixels per byte for Gray2
PPB = 4

# Compute padded width
padded_width = math.ceil(width / PPB) * PPB
pad_pixels = padded_width - width

raw = bytearray()

for y in range(height):
    row = pixels[y]

    # Pad row with 0s (black) if needed
    if pad_pixels:
        row = np.pad(row, (0, pad_pixels), constant_values=0)

    # Pack 4 pixels into each byte
    for x in range(0, padded_width, 4):
        b = 0
        for j in range(4):
            # Invert by subtracting the value from 3
            # inverted_pixel = 3 - (row[x + j] & 0x03)
            # b |= inverted_pixel << (6 - 2 * j)
            b |= (row[x + j] & 0x03) << (6 - 2 * j)
        raw.append(b)

with open("resources/weather_icons_20px_60x60_2b.raw", "wb") as f:
    f.write(raw)

print(f"Original size: {width}x{height}")
print(f"Padded size:   {padded_width}x{height}")
print(f"Bytes per row: {padded_width // 4}")
