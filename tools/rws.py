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


def frame_ids(d):
    """HAnim node id stored in each frame's own extension (frame 0 has none)."""
    clump = next(p for t, _, _, p in chunks(d) if t == 0x10)
    fl = next((s, p) for t, s, _, p in chunks(d, clump + 12) if t == 0x0E)
    ids = [None]
    for t, s, _, p in chunks(d, fl[1] + 12, fl[1] + 12 + fl[0]):
        if t == 3 and s:
            ids.append(next(struct.unpack_from("<3I", d, q + 12)[1] for tt, _, _, q in chunks(d, p + 12, p + 12 + s) if tt == 0x11E))
    return ids


def check_track_order(d):
    """.nmt tracks 3..23 are rotations of node ids 0..20 in id order (ids 21, 22 have none)."""
    from nmt import MOTION, parse
    par, _, _ = skeleton(d)
    ids = frame_ids(d)
    parent = {ids[f]: ids[par[f]] for f in range(1, 29) if par[f] > 0}
    assert sorted(set(parent) - set(parent.values())) == [10, 22, 24, 25, 26, 27]  # head, id 22 chain end, 4 tips
    assert [parent[i] for i in (24, 25, 26, 27)] == [3, 6, 15, 20]  # feet and hands end their limb chains
    clips = [parse(f.read_bytes()) for f in sorted(MOTION.glob("*01.nmt"))]
    clips = [m for m in clips if len(m["tracks"]) == 24 and m["tracks_end"]]
    static = [sum(len(m["tracks"][t]) <= 2 for m in clips) / len(clips) for t in range(3, 24)]
    # hands (ids 15, 20 = tracks 18, 23) hold one pose in most clips; no other rotation does
    assert [i + 3 for i, s in enumerate(static) if s > 0.5] == [18, 23], static


if __name__ == "__main__":
    if sys.argv[1:]:
        d = (MODEL / (sys.argv[1] + ".rws")).read_bytes()
        tree(d, maxd=int(sys.argv[2]) if sys.argv[2:] else 4)
    else:
        parents, _, nodes = skeleton((MODEL / "KAGETORA_BODY.rws").read_bytes())
        tips = [n for n in nodes if n[2] & 1 and n[0] >= 24]
        assert len(parents) == 29 and len(nodes) == 28 and len(tips) == 4, (len(parents), len(nodes), len(tips))
        assert sorted(n[0] for n in nodes) == list(range(28)) and [n[1] for n in nodes] == list(range(28))
        check_track_order((MODEL / "KAGETORA_BODY.rws").read_bytes())
        print("ok: 29 frames, 28 HAnim nodes (ids 0..27), 4 tip nodes (24..27) -> 24 animated, matches 24 .nmt tracks")
