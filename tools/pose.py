"""Write extracted/anim/{skeleton,clips}.bin for the viewer's rig: the game skeleton and every player/enemy clip sampled per frame.

    python tools/pose.py        (needs extracted/volume/motion and model; see docs/formats.md "Pose export")

skeleton.bin: u32 n=28; per node id 0..27: i32 parent id (-1 for the root id 23), 3 x f32 rest offset in the parent's frame
(model units, already x SCALE to metres). clips.bin: u32 count; per clip: 24 bytes name (NUL padded), u32 frames, then per
frame 24 f32: hips position (3), then 21 local quaternions x y z w for node ids 0..20 (as stored in the .nmt).
"""

import struct

import rws
from extract import enemy_names, player_names
from iso import ROOT
from nmt import MOTION, parse

OUT = ROOT / "extracted" / "anim"
SCALE = 0.84 / 9.5  # ESTIMATE: hips height in .nmt (0.84 m) over the model's hips offset (9.5 units)
NODES = 28


def sample(keys, f, kind):
    """Value at frame f: linear (position) or normalised linear (quaternion) between the keys that bracket it."""
    start = lambda k: k[3] if kind == "T" else k[4]
    val = lambda k: [x / 4096 for x in k[:3 if kind == "T" else 4]]
    i = max((j for j, k in enumerate(keys) if start(k) <= f), default=0)
    a = val(keys[i])
    if i + 1 == len(keys):
        return a
    t = min(max((f - start(keys[i])) / max(start(keys[i + 1]) - start(keys[i]), 1), 0), 1)
    b = val(keys[i + 1])
    if kind == "Q":
        if sum(x * y for x, y in zip(a, b)) < 0:
            b = [-x for x in b]
    r = [x + (y - x) * t for x, y in zip(a, b)]
    if kind == "Q":
        n = sum(x * x for x in r) ** 0.5 or 1
        r = [x / n for x in r]
    return r


def skeleton():
    d = (rws.MODEL / "KAGETORA_BODY.rws").read_bytes()
    par, off, _ = rws.skeleton(d)
    ids = rws.frame_ids(d)
    out = struct.pack("<I", NODES)
    by_id = {ids[i]: i for i in range(1, len(ids))}
    for n in range(NODES):
        i = by_id[n]
        p = ids[par[i]] if par[i] and par[i] < len(ids) and ids[par[i]] is not None else -1
        out += struct.pack("<i3f", p, *(x * SCALE for x in off[i]))
    return out


def main():
    OUT.mkdir(parents=True, exist_ok=True)
    (OUT / "skeleton.bin").write_bytes(skeleton())
    # Clips with another track count (DRINK_S05, SHEATH_BU05, LOOK07) use a different skeleton layout: skipped.
    names = [n for n in player_names() + enemy_names() if len(parse((MOTION / f"{n}.nmt").read_bytes())["tracks"]) == 24]
    body = b""
    for name in names:
        m = parse((MOTION / f"{name}.nmt").read_bytes())
        tr = m["tracks"]
        body += name.encode().ljust(24, b"\0") + struct.pack("<I", m["frames"])
        for f in range(m["frames"]):
            row = sample(tr[2], f, "T") + [x for i in range(21) for x in sample(tr[3 + i], f, "Q")]
            body += struct.pack("<87f", *row)
    (OUT / "clips.bin").write_bytes(struct.pack("<I", len(names)) + body)
    print(f"ok: {len(names)} clips, {NODES} nodes -> {OUT}")


if __name__ == "__main__":
    main()
