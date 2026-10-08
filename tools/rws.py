"""Walk RenderWare stream chunks (type, size, library id; little-endian).

    python tools/rws.py KAGETORA_BODY [depth]     print the chunk tree
    python tools/rws.py                           self-check the player skeleton
"""

import struct
import sys

from iso import ROOT

MODEL = ROOT / "extracted" / "volume" / "model"
# Only containers recurse; the rest are payload.
CONTAINERS = {0x03, 0x07, 0x0E, 0x0F, 0x10, 0x14, 0x15, 0x16, 0x1A, 0x1C}


def chunks(d, pos=0, end=None):
    end = len(d) if end is None else end
    while pos + 12 <= end:
        t, size, lib = struct.unpack_from("<3I", d, pos)
        yield t, size, lib, pos
        pos += 12 + size


def tree(d, pos=0, end=None, depth=0, maxd=4):
    for t, size, lib, p in chunks(d, pos, end):
        print("  " * depth + f"{t:#x} size {size} @{p:#x} lib {lib:#x}")
        if t in CONTAINERS and depth < maxd:
            tree(d, p + 12, p + 12 + size, depth + 1, maxd)


def skeleton(d):
    """HAnim hierarchy of the first clump: (frame parents, frame offsets, [(node id, node index, flags)])."""
    clump = next(p for t, _, _, p in chunks(d) if t == 0x10)
    fl = next((s, p) for t, s, _, p in chunks(d, clump + 12) if t == 0x0E)
    n = struct.unpack_from("<I", d, fl[1] + 24)[0]
    frames = [struct.unpack_from("<12f2I", d, fl[1] + 28 + 56 * i) for i in range(n)]
    nodes = []
    for t, s, _, p in chunks(d, fl[1] + 12, fl[1] + 12 + fl[0]):
        if t == 3 and s:
            _, _, _, q = next(chunks(d, p + 12, p + 12 + s))
            ver, nid, nn = struct.unpack_from("<3I", d, q + 12)
            if nn:
                nodes = [struct.unpack_from("<3I", d, q + 12 + 20 + 12 * k) for k in range(nn)]
    return [f[12] for f in frames], [f[9:12] for f in frames], nodes


if __name__ == "__main__":
    if sys.argv[1:]:
        d = (MODEL / (sys.argv[1] + ".rws")).read_bytes()
        tree(d, maxd=int(sys.argv[2]) if sys.argv[2:] else 4)
    else:
        parents, _, nodes = skeleton((MODEL / "KAGETORA_BODY.rws").read_bytes())
        tips = [n for n in nodes if n[2] & 1 and n[0] >= 24]
        assert len(parents) == 29 and len(nodes) == 28 and len(tips) == 4, (len(parents), len(nodes), len(tips))
        assert sorted(n[0] for n in nodes) == list(range(28)) and [n[1] for n in nodes] == list(range(28))
        print("ok: 29 frames, 28 HAnim nodes (ids 0..27), 4 tip nodes (24..27) -> 24 animated, matches 24 .nmt tracks")
