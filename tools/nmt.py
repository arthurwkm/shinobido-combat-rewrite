"""Parse Acquire .nmt motion files (little-endian) from extracted/volume/motion.

    python tools/nmt.py                 self-check on every file, summary of what holds
    python tools/nmt.py RUN01           dump one motion

Layout (see docs/formats.md): header, event list, common block, track offset table, tracks.
A track is an 8-byte header (u16 0, u16 count, u32 0) and `count` 16-byte keys (2 x 4 s16).
"""

import struct
import sys
from collections import Counter
from pathlib import Path

from iso import ROOT

MOTION = ROOT / "extracted" / "volume" / "motion"
FPS = 60


def parse(d):
    h = struct.unpack_from("<5f5I", d)
    m = {"t": h[:5], "flags": h[5:7], "ev_off": h[7], "len_s": h[1], "frames_hdr": round(h[1] * FPS)}
    p, events = h[7] or 0x28, []
    if h[7]:
        while struct.unpack_from("<I", d, p + 4)[0]:
            events.append(struct.unpack_from("<fIII", d, p))
            p += 16
    m["events"] = events
    m["frames"], m["bones"] = struct.unpack_from("<HH", d, p + 12)
    q, ents = p + 24, []
    while struct.unpack_from("<I", d, q)[0]:
        ents.append(struct.unpack_from("<I", d, q)[0])
        q += 4
    m["entries"] = ents
    pos, tracks = q + 4, []
    for _ in range(len(ents) + 1 if ents else 0):
        n = struct.unpack_from("<H", d, pos + 2)[0]
        if pos + 8 + 16 * n > len(d):
            pos = -1
            break
        keys = [struct.unpack_from("<8h", d, pos + 8 + 16 * i) for i in range(n)]
        tracks.append(keys)
        pos += 8 + 16 * n
    m["tracks_end"] = pos
    m["tracks"] = tracks
    return m


def times(track, kind):
    """(start frame, duration) per key. kind 'T' (position+tangent): A[3], B[3]; 'Q' (quaternion): B[0], B[1]."""
    return [(k[3], k[7]) if kind == "T" else (k[4], k[5]) for k in track]


def tiles(ts):
    """Keys start at 0 and each starts where the previous one ends."""
    return ts[0][0] == 0 and all(a + b == c for (a, b), (c, _) in zip(ts, ts[1:]))


def unit(track):
    return all(abs(sum(x * x for x in k[:4]) ** 0.5 / 4096 - 1) < 0.01 for k in track)


def check():
    c = Counter()
    for f in sorted(MOTION.glob("*.nmt")):
        d = f.read_bytes()
        m = parse(d)
        if not m["tracks"]:
            c["no tracks"] += 1
            continue
        c["tracks end at EOF", m["tracks_end"] == len(d)] += 1
        c["frames == length*60", m["frames"] == m["frames_hdr"]] += 1
        if m["tracks_end"] == len(d):
            tr = m["tracks"]
            c["track 0 position keys tile"] = c["track 0 position keys tile"] + tiles(times(tr[0], "T"))
            c["track 2 position keys tile"] = c["track 2 position keys tile"] + tiles(times(tr[2], "T"))
            c["tracks 3..20 quaternion keys tile, unit"] += all(
                tiles(times(t, "Q")) and unit(t) for t in tr[3:21]
            )
    for k, v in sorted(c.items(), key=str):
        print(v, *k) if isinstance(k, tuple) else print(v, k)


if __name__ == "__main__":
    if sys.argv[1:]:
        m = parse((MOTION / (sys.argv[1] + ".nmt")).read_bytes())
        print({k: v for k, v in m.items() if k != "tracks"})
        for i, t in enumerate(m["tracks"]):
            print(i, len(t), t[:2])
    else:
        check()
