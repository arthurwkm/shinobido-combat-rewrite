"""Assert motion evidence for candidate execution phases; does not assign gameplay roles.

    .venv/bin/python tools/execution_phases.py

Requires extracted/volume/motion. Writes nothing; see formats.md "Execution phase audit".
"""

import math

from extract import player_ids
from nmt import MOTION, parse
from pose import sample


def motion(name):
    return parse((MOTION / f"{name}.nmt").read_bytes())


def tracks(m):
    """Root, hips, and 21 joint rotations in the three surveyed layouts."""
    tr = m["tracks"]
    assert len(tr) in (24, 66, 68), len(tr)
    if len(tr) == 66:
        return tr[1], tr[3], tr[4:46:2]
    return tr[0], tr[2], tr[3:45:2] if len(tr) == 68 else tr[3:24]


def boundary(a, b):
    ar, ah, aq = tracks(a)
    br, bh, bq = tracks(b)
    assert ar[-1][:3] == br[0][:3], "root boundary"
    assert ah[-1][:3] == bh[0][:3], "hips boundary"
    assert all(x[-1][:4] == y[0][:4] for x, y in zip(aq, bq)), "joint boundary"


def main():
    boundary(motion("KIME_A01"), motion("KIME_B01"))
    for prefix, suffix in (("HI_", "05"), ("KI_HI", "07")):
        for direction in "BUC":
            boundary(motion(f"{prefix}{direction}S{suffix}"), motion(f"{prefix}{direction}A{suffix}"))
    print("ok: exact root/hips/21-joint boundaries: KIME_A01 -> B01; all six 05/07 B/U/C S -> A")

    ids = player_ids()
    for direction, duration in (("B", 80), ("U", 40)):
        for prefix, suffix in (("HI_", "05"), ("KI_HI", "07")):
            name = f"{prefix}{direction}C{suffix}"
            m = motion(name)
            assert m["frames_hdr"] == m["frames"] == duration
            cancels = [(round(t * 60), a, b) for t, typ, a, b in m["events"] if typ == 3]
            assert cancels == [(16, ids[name], 0xFFFF0000),
                               (30, ids[f"{prefix}{direction}BA{suffix}"], 0xFFFF0028)]
    print("ok: 05/07 B/U C phases share durations and self@16 / BA@30 cancel events")

    for name, frame in (("HI_BA05", 48), ("HI_UA05", 98), ("HI_CA05", 68),
                        ("HI_BA01", 20), ("HI_UA01", 23), ("HI_C01", 0)):
        assert (frame, 9, 20) in [(round(t * 60), a, b) for t, typ, a, b in motion(name)["events"] if typ == 1]
    print("ok: active attack events in 05 B/U/C A and 01 BA/UA/C")

    start, finish = motion("HI_BS01"), motion("HI_BA01")
    assert (start["frames_hdr"], start["frames"]) == (40, 160)
    _, _, sq = tracks(start)
    _, _, fq = tracks(finish)
    errors = []
    for frame in (start["frames_hdr"], start["frames"]):
        angles = []
        for a, b in zip(sq, fq):
            dot = abs(sum(x * y for x, y in zip(sample(a, frame, "Q"), sample(b, 0, "Q"))))
            angles.append(math.degrees(2 * math.acos(min(1, dot))))
        errors.append(sum(angles) / len(angles))
    assert errors[0] < 6 and errors[1] > 40, errors
    print(f"ok: HI_BS01 -> BA01 mean pose error: header40={errors[0]:.1f}deg, common160={errors[1]:.1f}deg")
    print("Roles, actor pairing and input conditions remain hypotheses; continuity alone does not establish them.")


if __name__ == "__main__":
    main()
