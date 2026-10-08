"""Motion category word (.nmt header +0x14) vs. type 5/6 masks of the player's motions (docs/formats.md, "Transition-category bits")."""
from collections import defaultdict

from extract import player_names
from nmt import MOTION, parse

cat, masks = {}, defaultdict(set)
for n in player_names():
    m = parse((MOTION / (n + ".nmt")).read_bytes())
    cat[n] = m["flags"][0]
    masks[n] = {a0 for _, ty, a0, _ in m["events"] if ty == 5}
bits = defaultdict(list)
for n, c in cat.items():
    for b in range(32):
        if c >> b & 1:
            bits[1 << b].append(n)
for b, v in sorted(bits.items()):
    print(f"{b:#x}: {len(v)} motions, e.g. {v[:4]}")
# checks tied to formats.md: attacks carry 0x20000, hit reactions 0x80, JUMP01 0x4000
assert all(cat[n] & 0x20000 for n in ("AT_A01", "AT_C01", "AT_D01", "AT_E01", "SOCCER01"))
assert all(cat[n] & 0x80 for n in cat if n.startswith("HI_") and n != "HI_F01") and cat["JUMP01"] & 0x4000
# every mask bit except 0x4 (DRINK01) and 0x8 (attack chains) selects a player motion header
used = {1 << b for n in masks for a in masks[n] for b in range(32) if a >> b & 1}
print("mask bits used:", sorted(hex(b) for b in used), "matching no header:", sorted(hex(b) for b in used if b not in bits))
assert sorted(b for b in used if b not in bits) == [0x4, 0x8]
print("ok")
