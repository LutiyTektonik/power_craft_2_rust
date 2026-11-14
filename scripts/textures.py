#!/usr/bin/env python3
"""Original 16x16 placeholder textures. Python stdlib only; no downloaded art."""
import struct
import zlib


def png(pixel):
    def chunk(kind, data):
        crc = zlib.crc32(kind + data) & 0xFFFFFFFF
        return struct.pack('>I', len(data)) + kind + data + struct.pack('>I', crc)

    raw = b''.join(b'\0' + bytes(c for x in range(16) for c in pixel(x, y)) for y in range(16))
    header = struct.pack('>IIBBBBB', 16, 16, 8, 2, 0, 0, 0)
    return b'\x89PNG\r\n\x1a\n' + chunk(b'IHDR', header) + chunk(b'IDAT', zlib.compress(raw, 9)) + chunk(b'IEND', b'')


def top(x, y):
    if x in (0, 15):
        return (143, 151, 159)
    if (3 <= y <= 7 and abs(x - 7.5) <= y - 2) or (7 <= y <= 12 and 6 <= x <= 9):
        return (245, 184, 48)
    shade = 42 if y % 4 < 2 else 57
    return (shade, shade + 5, shade + 10)


def side(x, y):
    shade = 110 if y % 4 == 0 or x in (0, 15) else 64
    return (shade, shade + 6, shade + 12)


def textures():
    return {'conveyor_top.png': png(top), 'conveyor_side.png': png(side)}


if __name__ == '__main__':
    from pathlib import Path
    root = Path(__file__).resolve().parents[1] / 'content/assets/powercraft2/textures/block'
    root.mkdir(parents=True, exist_ok=True)
    for name, data in textures().items():
        (root / name).write_bytes(data)
