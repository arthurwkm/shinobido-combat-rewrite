"""PS2 native RenderWare textures (chunk 0x15 in a 0x16 dictionary) -> RGBA -> PNG.

    python tools/texture.py        decode every Kagetora texture to extracted/textures/<part>/<name>.png

The file holds the GS upload packets: GIF tag + A+D regs + IMAGE tag + data for the texels, then the same for the palette.
Texels are stored in GS memory order (8/4-bit uploaded as 32-bit rects), so decoding replays the GS addressing.
"""

import struct
import sys
import zlib

from rws import MODEL, chunks
from iso import ROOT

# GS block order inside a page (PSMCT32 and PSMT8: 8 wide x 4 high; PSMT4: 4 wide x 8 high)
BLOCK32 = [[0, 1, 4, 5, 16, 17, 20, 21], [2, 3, 6, 7, 18, 19, 22, 23], [8, 9, 12, 13, 24, 25, 28, 29], [10, 11, 14, 15, 26, 27, 30, 31]]
BLOCK4 = [[0, 2, 8, 10], [1, 3, 9, 11], [4, 6, 12, 14], [5, 7, 13, 15], [16, 18, 24, 26], [17, 19, 25, 27], [20, 22, 28, 30], [21, 23, 29, 31]]
# pixel order inside a block (word index for 8x8 PSMCT32, byte index for 16x16 PSMT8, nibble index for 32x16 PSMT4)
COL32 = [[0, 1, 4, 5, 8, 9, 12, 13], [2, 3, 6, 7, 10, 11, 14, 15], [16, 17, 20, 21, 24, 25, 28, 29], [18, 19, 22, 23, 26, 27, 30, 31]]
COL32 += [[v + 32 for v in r] for r in COL32]
_c8 = [
    [0, 4, 16, 20, 32, 36, 48, 52, 2, 6, 18, 22, 34, 38, 50, 54],
    [8, 12, 24, 28, 40, 44, 56, 60, 10, 14, 26, 30, 42, 46, 58, 62],
    [33, 37, 49, 53, 1, 5, 17, 21, 35, 39, 51, 55, 3, 7, 19, 23],
    [41, 45, 57, 61, 9, 13, 25, 29, 43, 47, 59, 63, 11, 15, 27, 31],
]


def _rows(base, add):
    """Rows 0-3 as given, rows 4-7 = rows 0-3 + add with the halves of every 8-entry group swapped, rows 8+ repeat +2*add."""
    sw = lambda r: [v for g in range(0, len(r), 8) for v in r[g + 4 : g + 8] + r[g : g + 4]]
    top = base + [sw([v + add for v in r]) for r in base]
    return top + [[v + 2 * add for v in r] for r in top]


COL8 = _rows(_c8, 64)
_c4 = [
    [0, 8, 32, 40, 64, 72, 96, 104, 2, 10, 34, 42, 66, 74, 98, 106, 4, 12, 36, 44, 68, 76, 100, 108, 6, 14, 38, 46, 70, 78, 102, 110],
    [16, 24, 48, 56, 80, 88, 112, 120, 18, 26, 50, 58, 82, 90, 114, 122, 20, 28, 52, 60, 84, 92, 116, 124, 22, 30, 54, 62, 86, 94, 118, 126],
    [65, 73, 97, 105, 1, 9, 33, 41, 67, 75, 99, 107, 3, 11, 35, 43, 69, 77, 101, 109, 5, 13, 37, 45, 71, 79, 103, 111, 7, 15, 39, 47],
    [81, 89, 113, 121, 17, 25, 49, 57, 83, 91, 115, 123, 19, 27, 51, 59, 85, 93, 117, 125, 21, 29, 53, 61, 87, 95, 119, 127, 23, 31, 55, 63],
]
COL4 = _rows(_c4, 128)


def ct32_memory(raster, width):
    """Write a PSMCT32 raster (list of words, `width` px wide, buffer width 1 page) into GS memory words."""
    mem = {}
    for i, v in enumerate(raster):
        x, y = i % width, i // width
        page = (y // 32) * 2048 + BLOCK32[(y % 32) // 8][(x % 64) // 8] * 64
        mem[page + COL32[y % 8][x % 8]] = v
    return mem


def packet(d, pos, size):
    """(rect w, h, data bytes) of one upload packet: GIF tag, NLOOP A+D regs, IMAGE tag, data."""
    nl = struct.unpack_from("<Q", d, pos)[0] & 0x7FFF
    w, h = 0, 0
    for i in range(nl):
        val, addr = struct.unpack_from("<QQ", d, pos + 16 + 16 * i)
        if addr == 0x52:
            w, h = val & 0xFFFFFFFF, val >> 32
    img = struct.unpack_from("<Q", d, pos + 16 + 16 * nl)[0] & 0x7FFF
    start = pos + 32 + 16 * nl
    assert start + 16 * img == pos + size, "IMAGE tag NLOOP does not fill the packet"
    return w, h, d[start : pos + size]


def decode(d, pos):
    """Native texture chunk payload at `pos` (its 0x15 header position) -> (name, w, h, RGBA bytes)."""
    end = pos + 12 + struct.unpack_from("<I", d, pos + 4)[0]
    subs = list(chunks(d, pos + 12, end))
    name = d[subs[1][3] + 12 : subs[1][3] + 28].split(b"\0")[0].decode()
    big = subs[3][3] + 12
    hdr = struct.unpack_from("<16I", d, big + 12)
    w, h, depth = hdr[:3]
    assert depth in (4, 8) and (hdr[4] >> 20) & 63 == {4: 0x14, 8: 0x13}[depth], "tex0 PSM does not match depth"
    p0 = big + 12 + 64 + 12
    _, _, tex = packet(d, p0, hdr[12])
    _, _, pal = packet(d, p0 + hdr[12], hdr[13])
    pw_ = max(w, 32) if depth == 4 else w  # 4-bit rows are padded to the 32 px block width
    assert len(tex) * 8 // depth == pw_ * h

    def addr(x, y):  # (GS memory position, in bytes for 8-bit or nibbles for 4-bit) of texel (x, y)
        if depth == 8:
            return (y // 64) * 8192 + BLOCK32[(y % 64) // 16][(x % 128) // 16] * 256 + COL8[y % 16][x % 16]
        return (y // 128) * 16384 + BLOCK4[(y % 128) // 16][(x % 128) // 32] * 512 + COL4[y % 16][x % 32]

    words = struct.unpack(f"<{len(tex) // 4}I", tex)
    need = {addr(x, y) // (4 if depth == 8 else 8) for y in range(h) for x in range(pw_)}
    # the file is the raster of a PSMCT32 rect; its width is the one whose GS words are exactly the texture's words
    for rw in (8, 16, 32, 64, 128):
        mem = ct32_memory(words, rw)
        if set(mem) == need:
            break
    else:
        raise AssertionError(f"{name}: no PSMCT32 rect width covers the texture's GS words")
    raw = b"".join(struct.pack("<I", mem.get(i, 0)) for i in range(max(mem) + 1))
    # the palette is stored as the GS reads it (CSM1): 8-bit entries 8..15 and 16..23 of each 32 are swapped
    pw = struct.unpack(f"<{len(pal) // 4}I", pal)
    clut = [pw[(i & ~0x18) | ((i & 8) << 1) | ((i & 0x10) >> 1)] for i in range(256)] if depth == 8 else list(pw[:16])
    out = bytearray()
    for y in range(h):
        for x in range(w):
            a = addr(x, y)
            idx = raw[a] if depth == 8 else raw[a // 2] >> (4 * (a & 1)) & 15
            c = clut[idx]
            out += bytes((c & 255, c >> 8 & 255, c >> 16 & 255, min(255, (c >> 24) * 2)))  # PS2 alpha 0x80 = opaque
    return name, w, h, bytes(out)


def textures(d):
    for t, s, _, p in chunks(d):
        if t == 0x16:
            for t2, _, _, p2 in chunks(d, p + 12, p + 12 + s):
                if t2 == 0x15:
                    yield decode(d, p2)


def png(path, w, h, rgba):
    def ck(t, b):
        return struct.pack(">I", len(b)) + t + b + struct.pack(">I", zlib.crc32(t + b))

    rows = b"".join(b"\0" + rgba[y * w * 4 : (y + 1) * w * 4] for y in range(h))
    path.write_bytes(b"\x89PNG\r\n\x1a\n" + ck(b"IHDR", struct.pack(">2I5B", w, h, 8, 6, 0, 0, 0)) + ck(b"IDAT", zlib.compress(rows)) + ck(b"IEND", b""))


def smoothness(w, h, rgba):
    """Mean absolute RGB difference between horizontal neighbours (low for a picture, high for scrambled texels)."""
    tot = sum(abs(rgba[i] - rgba[i + 4]) for y in range(h) for x in range(w - 1) for i in range(y * w * 4 + x * 4, y * w * 4 + x * 4 + 3))
    return tot / (h * (w - 1) * 3)


if __name__ == "__main__":
    assert all(sorted(v for r in t for v in r) == list(range(len(t) * len(t[0]))) for t in (COL32, COL8, COL4))
    out = ROOT / "extracted" / "textures"
    count = 0
    for part in ("BODY", "HEAD_H", "HEAD_L", "WEAPON"):
        d = (MODEL / f"KAGETORA_{part}.rws").read_bytes()
        (out / part.lower()).mkdir(parents=True, exist_ok=True)
        for name, w, h, rgba in textures(d):
            png(out / part.lower() / f"{name}.png", w, h, rgba)
            count += 1
            if "-v" in sys.argv:
                print(part, name, w, h, round(smoothness(w, h, rgba), 1))
    face = next(t for t in textures((MODEL / "KAGETORA_HEAD_H.rws").read_bytes()) if t[0] == "kagetora_face")
    assert smoothness(*face[1:]) < 8, "face texture is scrambled (a picture has smooth neighbours)"
    print(f"ok: {count} textures decoded")
