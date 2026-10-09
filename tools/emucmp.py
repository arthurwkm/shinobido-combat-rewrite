"""Side by side: PCSX2 recording (tools/pcsx2.py video) vs viewer record (--record ... --every n), same pad script times.

    python tools/emucmp.py <game.mov> <viewer rec dir> <out.jpg> <tick> [<tick>...]

Ticks are 60 Hz script frames; the game video starts LEAD s before the script (recorder start-up in pcsx2.py).
One row per tick: game left, viewer right.
"""

import subprocess
import sys
from pathlib import Path

LEAD = 0.5
W, H = 400, 300


def main():
    if len(sys.argv) < 5:
        sys.exit(__doc__)
    mov, rec, out, ticks = sys.argv[1], Path(sys.argv[2]), sys.argv[3], [int(t) for t in sys.argv[4:]]
    args, f = [], ""
    for i, t in enumerate(ticks):
        args += ["-ss", f"{t / 60 + LEAD:.3f}", "-i", mov, "-i", str(rec / f"{t:05}.png")]
        fit = f"scale={W}:{H}:force_original_aspect_ratio=decrease,pad={W}:{H}:(ow-iw)/2:(oh-ih)/2,setsar=1"
        f += f"[{2 * i}]{fit}[g{i}];[{2 * i + 1}]{fit}[v{i}];[g{i}][v{i}]hstack[r{i}];"
    f += "".join(f"[r{i}]" for i in range(len(ticks))) + f"vstack={len(ticks)}" if len(ticks) > 1 else "[r0]null"
    subprocess.run(["ffmpeg", "-loglevel", "error", "-y", *args, "-filter_complex", f, "-frames:v", "1", out], check=True)
    print(out)


if __name__ == "__main__":
    main()
