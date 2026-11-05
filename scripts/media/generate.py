#!/usr/bin/env python3
"""Independently authored CC0 test signals; requires only Python's standard library."""
import math
from pathlib import Path
import struct
import wave

root = Path(__file__).resolve().parents[2] / "fixtures/media"
root.mkdir(parents=True, exist_ok=True)
with wave.open(str(root / "flash-beep.wav"), "wb") as audio:
    audio.setparams((1, 2, 48000, 0, "NONE", "not compressed"))
    for sample in range(96000):
        seconds = sample / 48000
        active = any(start <= seconds < start + 0.08 for start in (0.5, 1.5))
        value = int(8000 * math.sin(2 * math.pi * 880 * seconds)) if active else 0
        audio.writeframesraw(struct.pack("<h", value))

def block(data):
    return b"".join(bytes([len(data[i:i + 255])]) + data[i:i + 255] for i in range(0, len(data), 255)) + b"\0"

def frame(left, top, width, height, pixels, disposal, delay):
    codes = [value for pixel in pixels for value in (4, pixel)] + [5]
    bits = sum(code << (3 * index) for index, code in enumerate(codes))
    compressed = bits.to_bytes((len(codes) * 3 + 7) // 8, "little")
    return (b"!\xf9\x04" + bytes([disposal << 2 | 1]) + struct.pack("<H", delay) + b"\0\0"
            + b"," + struct.pack("<HHHHB", left, top, width, height, 0) + b"\x02" + block(compressed))

# Global palette: transparent, red, green, blue. Tests exercise offsets and
# dispose-to-background (2) followed by restore-previous (3), not opaque snapshots.
gif = b"GIF89a" + struct.pack("<HHBBB", 4, 4, 0x81, 0, 0)
gif += bytes([0, 0, 0, 255, 0, 0, 0, 255, 0, 0, 0, 255])
gif += b"!\xff\x0bNETSCAPE2.0\x03\x01\0\0\0"
gif += frame(0, 0, 4, 4, [1, 1, 1, 1, 1, 0, 0, 1, 1, 0, 0, 1, 1, 1, 1, 1], 1, 10)
gif += frame(1, 1, 2, 2, [2] * 4, 2, 20)
gif += frame(0, 0, 1, 1, [3], 3, 30)
gif += frame(3, 3, 1, 1, [3], 1, 10)
(root / "disposal.gif").write_bytes(gif + b";")
(root / "corrupt.mp4").write_bytes(b"This is deliberately not a media container.\n")
print(root)
