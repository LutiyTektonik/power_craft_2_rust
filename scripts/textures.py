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


def speedy(x, y):
    color = top(x, y)
    return (63, 215, 242) if color == (245, 184, 48) else color


def brake(x, y, powered=False):
    if 2 <= x <= 13 and y in (4, 5, 10, 11):
        return (245, 55, 45) if powered else (185, 112, 60)
    return top(x, y)


def detector(x, y, powered=False):
    if x in (2, 3, 12, 13) and 4 <= y <= 11:
        return (66, 240, 105) if powered else (145, 163, 180)
    return top(x, y)


def redirector(x, y, powered=False):
    if x in (2, 3, 12, 13) and 4 <= y <= 11:
        return (219, 139, 255) if powered else (100, 71, 132)
    color = top(x, y)
    return (156, 114, 197) if color == (245, 184, 48) else color


def separator(x, y):
    if 2 <= y <= 9 and 3 <= x <= 12:
        for center, color in ((4, (242, 145, 45)), (11, (84, 203, 120))):
            if (2 <= y <= 5 and abs(x - center) <= y - 2) or (6 <= y <= 9 and x == center):
                return color
    color = top(x, y)
    return (177, 192, 208) if color == (245, 184, 48) else color


def ejector(x, y):
    if y in (10, 11, 13, 14) and 3 <= x <= 12:
        return (229, 95, 52)
    color = top(x, y)
    return (234, 132, 81) if color == (245, 184, 48) else color


def elevator(x, y, down=False):
    color = (78, 176, 229) if down else (232, 184, 65)
    y = 15-y if down else y
    if (3 <= y <= 7 and abs(x-7.5) <= y-2) or (8 <= y <= 12 and 6 <= x <= 9):
        return color
    return (91, 103, 115) if y in (0, 15) or x in (0, 15) else (48, 57, 67)


def splitter_plate(x, y, color):
    if x in (0, 15) or y in (0, 15):
        return (41, 49, 59)
    if x in (1, 14) or y in (1, 14):
        return (120, 130, 139)
    if (4 <= y <= 8 and abs(x - 7.5) <= y - 3) or (8 < y <= 11 and 6 <= x <= 9):
        return color
    return (64, 72, 82)


def textures():
    face_colors = {
        'down': (73, 192, 255), 'up': (255, 76, 123),
        'north': (255, 136, 73), 'south': (232, 255, 66),
        'west': (76, 255, 127), 'east': (81, 73, 255),
    }
    return {'conveyor_top.png': png(top), 'conveyor_side.png': png(side),
            'speedy_conveyor_top.png': png(speedy),
            'brake_conveyor_top.png': png(brake),
            'brake_conveyor_powered.png': png(lambda x, y: brake(x, y, True)),
            'detector_conveyor_top.png': png(detector),
            'detector_conveyor_powered.png': png(lambda x, y: detector(x, y, True)),
            'redirector_conveyor_top.png': png(redirector),
            'redirector_conveyor_powered.png': png(lambda x, y: redirector(x, y, True)),
            'separator_conveyor_top.png': png(separator),
            'ejector_conveyor_top.png': png(ejector),
            'elevator_up.png': png(elevator),
            'elevator_down.png': png(lambda x, y: elevator(x, y, True)),
            **{f'splitter_{face}.png': png(lambda x, y: splitter_plate(x, y, color))
               for face, color in face_colors.items()}}


if __name__ == '__main__':
    from pathlib import Path
    root = Path(__file__).resolve().parents[1] / 'content/assets/powercraft2/textures/block'
    root.mkdir(parents=True, exist_ok=True)
    for name, data in textures().items():
        (root / name).write_bytes(data)
