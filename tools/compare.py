"""Run every extracted action through the simulation and compare with the game files.

    python tools/compare.py     prints mismatches; exits 1 if any (frames, final root position, 1 cm)
"""

import subprocess
import sys

from extract import player_names
from iso import ROOT
from nmt import MOTION, parse

out = subprocess.run(
    ["cargo", "run", "-q", "--example", "dump"], cwd=ROOT, check=True, capture_output=True, text=True
).stdout
sim = {l.split()[0]: l.split() for l in out.splitlines()}
bad = 0
for name in player_names():
    m = parse((MOTION / f"{name}.nmt").read_bytes())
    k = m["tracks"][1 if len(m["tracks"]) in (26, 66) else 0]
    want = [(k[-1][i] - k[0][i]) / 4096 for i in range(3)] if k else [0.0] * 3
    _, n, *pos = sim[name]
    if int(n) != m["frames"] or any(abs(float(p) - w) > 0.01 for p, w in zip(pos, want)):
        bad += 1
        print(f"MISMATCH {name}: sim {n} f {pos}, file {m['frames']} f {want}")
print(f"{len(sim) - bad}/{len(sim)} actions within tolerance")
sys.exit(bad > 0)
