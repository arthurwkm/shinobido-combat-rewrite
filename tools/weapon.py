"""Parse weapon\\wpdata.bin and weapon\\wpset.bin (extracted/volume/weapon); `python tools/weapon.py` self-checks.

wpdata: 55 records of 112 bytes: 8 x u16 header, then 6 x vec4 f32 (96 bytes).
wpset:  u32 count (25), count u32 offsets, then per set u16 words (a, b, c, n, n indices, 0 pad if n is odd)
        where the indices are < 55 and point into wpdata. Meaning of the fields: unknown (docs/formats.md).
"""

import struct

from iso import ROOT

W = ROOT / "extracted" / "volume" / "weapon"


def load():
    d = (W / "wpdata.bin").read_bytes()
    recs = [
        (struct.unpack_from("<8H", d, o), struct.unpack_from("<24f", d, o + 16))
        for o in range(0, len(d), 112)
    ]
    s = (W / "wpset.bin").read_bytes()
    n = struct.unpack_from("<I", s)[0]
    offs = list(struct.unpack_from(f"<{n}I", s, 4)) + [len(s)]
    sets = [struct.unpack(f"<{(offs[i + 1] - offs[i]) // 2}H", s[offs[i] : offs[i + 1]]) for i in range(n)]
    return len(d) % 112, recs, sets


if __name__ == "__main__":
    rem, recs, sets = load()
    assert rem == 0 and len(recs) == 55 and len(sets) == 25
    for st in sets:
        assert len(st) == 4 + st[3] + st[3] % 2 and all(i < 55 for i in st[4 : 4 + st[3]]), st
    print(f"ok: {len(recs)} wpdata records, {len(sets)} wpset entries, all indices valid")
