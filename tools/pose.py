"""Write extracted/anim/{skeleton,clips}.bin for the viewer's rig: the game skeleton and every player/enemy clip sampled per frame.

    python tools/pose.py        (needs extracted/volume/motion and model; see docs/formats.md "Pose export")

skeleton.bin: u32 n=28; per node id 0..27: i32 parent id (-1 for the root id 23), 3 x f32 rest offset in the parent's frame
(model units, already x SCALE to metres), then 6 f32 head-local min/max bounds in metres. clips.bin: u32 count; per clip: 24 bytes name (NUL padded), u32 frames, then per
frame 87 f32: hips position (3), then 21 local quaternions x y z w for node ids 0..20 (as stored in the .nmt).
"""

import struct

import rws
import model
from extract import enemy_names, player_names
from iso import ROOT
from nmt import MOTION, parse

OUT = ROOT / "extracted" / "anim"
SCALE = 0.84 / 9.5  # World/model units: emulator jump calibration, docs/formats.md.
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
    head = (rws.MODEL / "KAGETORA_HEAD_H.rws").read_bytes()
    vertices = [v for _, _, native, _ in model.geometries(head) for _, pk in model.packets(native) for v in pk["pos"]]
    # Head vertices are local to node 10; include hair in the primitive's bounds.
    bounds = [[fn(v[k] for v in vertices) * SCALE for k in range(3)] for fn in (min, max)]
    return out + struct.pack("<6f", *bounds[0], *bounds[1])


def main():
    body = (rws.MODEL / "KAGETORA_BODY.rws").read_bytes()
    head = (rws.MODEL / "KAGETORA_HEAD_H.rws").read_bytes()
    model.check(body)
    model.check(head)
    sole, _ = model.height(body)
    _, crown = model.height(head)
    assert abs((crown - sole) * SCALE - 1.568810577) < 1e-6
    print(f"model: sole {sole:.9f}, crown {crown:.9f}, height {crown - sole:.9f} units = {(crown - sole) * SCALE:.9f} m")
    OUT.mkdir(parents=True, exist_ok=True)
    (OUT / "skeleton.bin").write_bytes(skeleton())
    # 24-track clips: root T, root Q, hips T, 21 Q. 66-track clips (set 05 modes): root T, root Q, then a (T, Q) pair per
    # bone, node k = bone k+1. 26-track clips (soldier swings, set 19): empty, root T, root Q, hips T, 21 Q, 1 extra. Other counts (SHEATH_BU05 68, LOOK07 22) are skipped.
    names = [n for n in player_names() + enemy_names() if len(parse((MOTION / f"{n}.nmt").read_bytes())["tracks"]) in (24, 26, 66)]
    body = b""
    for name in names:
        m = parse((MOTION / f"{name}.nmt").read_bytes())
        tr = m["tracks"]
        body += name.encode().ljust(24, b"\0") + struct.pack("<I", m["frames"])
        for f in range(m["frames"]):
            if len(tr) == 26:  # set 19: a leading empty track and one trailing extra (weapon) track around the 24 layout
                row = sample(tr[3], f, "T") + [x for i in range(21) for x in sample(tr[4 + i], f, "Q")]
            elif len(tr) == 66:
                row = sample(tr[3], f, "T") + [x for i in range(21) for x in sample(tr[4 + 2 * i], f, "Q")]
            else:
                row = sample(tr[2], f, "T") + [x for i in range(21) for x in sample(tr[3 + i], f, "Q")]
            body += struct.pack("<87f", *row)
    (OUT / "clips.bin").write_bytes(struct.pack("<I", len(names)) + body)
    print(f"ok: {len(names)} clips, {NODES} nodes -> {OUT}")


if __name__ == "__main__":
    main()
