"""Parse `camera\\*.cmt` keyframe cameras and `cmtnfile.bin` (docs/formats.md "Camera data").

    python tools/camera.py     parses every .cmt, asserts the track layout fills each file exactly
"""

import struct

from iso import find_iso
from volume import read_entry, volume_entries

# (track, words per key, words before the key list) -> key layout, see formats.md
STRIDE = 8


def parse_cmt(data):
    """{length, pos, rot, fov}: pos/rot keys (value tuple, t, dur), fov keys (value, t, dur).
    Header: u32 0, u32 0x10000|frames, u32 0, f32 frames, then 3 u32 track offsets (0x20, ...)."""
    z, flen, z2, fl = struct.unpack("<IIIf", data[:16])
    assert z == 0 and z2 == 0 and flen >> 16 == 1 and (flen & 0xFFFF) == fl, (flen, fl)
    offs = struct.unpack("<3I", data[16:28])
    assert offs[0] == 0x20, offs
    ends = offs[1:] + (len(data),)
    out = {"length": int(fl)}
    for name, a, b, width in zip(("pos", "rot", "fov"), offs, ends, (3, 4, 1)):
        zero, n, zero2, total = struct.unpack("<IIIf", data[a : a + 16])
        stride = 4 if name == "fov" else STRIDE
        assert zero == 0 and zero2 == 0 and total == fl, (name, zero, zero2, total)
        assert b - a == 16 + stride * 4 * n, (name, b - a, n)
        keys = []
        for k in range(n):
            w = struct.unpack("<%df" % stride, data[a + 16 + 4 * stride * k :][: 4 * stride])
            if name == "fov":
                keys.append(((w[0],), w[1], w[3]))
            elif name == "pos":
                keys.append((w[:3], w[3], w[7]))
            else:
                keys.append((w[:4], w[4], w[5]))
        times = [k[1] for k in keys]
        assert times[0] == 0 and times[-1] == fl and times == sorted(times), (name, times)
        for k, nxt in zip(keys, keys[1:]):
            assert k[1] + k[2] == nxt[1], (name, k, nxt)
        if name == "rot":
            for q, _, _ in keys:
                assert abs(sum(c * c for c in q) - 1) < 0.05, q
        out[name] = keys
    return out


def main():
    iso = find_iso()
    ents = volume_entries(iso)
    names = sorted(p for p in ents if p.startswith("camera\\") and p.endswith(".cmt"))
    fovs = []
    for p in names:
        c = parse_cmt(read_entry(iso, ents, p))
        fovs += [k[0][0] for k in c["fov"]]
    tn = read_entry(iso, ents, "camera\\cmtnfile.bin")
    n = struct.unpack("<I", tn[:4])[0]
    first = struct.unpack("<I", tn[4:8])[0]
    assert first == 4 + 4 * n, (first, n)
    listed = tn[first:].rstrip(b"\0").split(b"\0")
    assert len(listed) == n, (len(listed), n)
    have = {p[7:].lower() for p in names}
    missing = [s for s in listed if s.decode().lower() not in have]
    assert not missing, missing
    print("ok: %d .cmt parsed, track layout fills every file; fov %.2f..%.2f rad; cmtnfile lists %d names, all present"
          % (len(names), min(fovs), max(fovs), n))


if __name__ == "__main__":
    main()
