"""Check wall-hug motion selection against source pose endpoints and root tracks."""
from nmt import MOTION, parse
from pose import sample


def motion(name):
    m = parse((MOTION / f"{name}.nmt").read_bytes())
    assert len(m["tracks"]) == 24, name
    return m


def pose(m, frame):
    return [sample(t, frame, "Q") for t in m["tracks"][3:24]]


def matches(a, b):
    return all(abs(sum(x * y for x, y in zip(p, q))) > .99 for p, q in zip(a, b))


def main():
    for side, sign, peek in [("RH", -1, "STOPLR05"), ("LH", 1, "STOPLL05")]:
        walk = motion(f"HUG_{side}05")
        assert walk["frames"] == 52
        start = sample(walk["tracks"][0], 0, "T")
        end = sample(walk["tracks"][0], walk["frames"], "T")
        assert abs(end[0] - start[0] - sign * .599853515625) < 1e-8
        assert end[1:] == start[1:]
        assert matches(pose(walk, 0), pose(walk, walk["frames"]))
        idle, hold = motion(f"STOP_{side}05"), motion(peek)
        look, back = motion(f"LOOK_{side}05"), motion(f"BACK_{side}05")
        assert matches(pose(look, 0), pose(idle, 0))
        assert matches(pose(look, look["frames"]), pose(hold, 0))
        assert matches(pose(back, 0), pose(hold, 0))
        assert matches(pose(back, back["frames"]), pose(idle, 0))
    for transition, idle in [("ONE06", "STOP_B06"), ("TWO06", "STOP_A06")]:
        m = motion(transition)
        assert matches(pose(m, m["frames"]), pose(motion(idle), 0))
        assert not matches(pose(m, 0), pose(m, m["frames"]))
    print("ok: 05 hug loops move +/-0.599854 m/52f; LOOK/BACK match peek endpoints; ONE/TWO06 are transitions")


if __name__ == "__main__":
    main()
