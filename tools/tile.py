"""Tile recorded viewer frames (row 2) under frames of a Shinobido clip (row 1) into one PNG.
usage: tile.py <out.png> <clip> <sec,sec,...> <rec_dir> <tick,tick,...>   (all tiles 320 px wide)"""
import subprocess, sys, os

out, clip, secs, rec, ticks = sys.argv[1:6]
inputs, tiles = [], []
for s in secs.split(","):
    inputs += ["-ss", s, "-i", clip]
for t in ticks.split(","):
    inputs += ["-i", f"{rec}/{int(t):05d}.png"]
n = len(inputs) // 2 // 1
a, b = len(secs.split(",")), len(ticks.split(","))
f = "".join(f"[{i}:v]scale=320:180[v{i}];" for i in range(a + b))
f += "".join(f"[v{i}]" for i in range(a)) + f"hstack={a}[top];"
f += "".join(f"[v{i}]" for i in range(a, a + b)) + f"hstack={b}[bot];[top][bot]vstack[o]"
os.makedirs(os.path.dirname(out) or ".", exist_ok=True)
subprocess.run(["ffmpeg", "-y", "-v", "error", *inputs, "-filter_complex", f, "-map", "[o]", "-frames:v", "1", out], check=True)
print("wrote", out)
