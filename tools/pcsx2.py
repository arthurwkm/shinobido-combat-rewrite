"""Drive the owner's PCSX2 (Shinobido running) with key events and grab its own screenshots.

    python tools/pcsx2.py shot <name>             game window capture -> target/pcsx2/<name>.png
    python tools/pcsx2.py load                    load the current save-state slot (F3)
    python tools/pcsx2.py play <script> [<name>]  timed pad script (below); frames go to target/pcsx2/<name>/
    python tools/pcsx2.py video <script> [<name>] same, and records the game window to target/pcsx2/<name>.mov

Script lines (`#` comments), times in 60 Hz frames from the start:
    <frame> down <button...>   hold         <frame> up <button...>   release
    <frame> tap <button...>    down, 3 frames, up
    <frame> shot               screenshot   <frame> load             load the current save state
Buttons: square cross circle triangle l1 l2 r1 r2 l3 r3 start select up down left right (d-pad),
lup ldown lleft lright (left stick), rup rdown rleft rright (right stick); the keys are the owner's
PCSX2 bindings (img/keymappings pcsx2 *.png, [Pad1] of PCSX2.ini). Keyboard sticks are full tilt only.

Needs macOS Accessibility (key events) for the process that runs it; PCSX2 is brought to the front
because it pauses on focus loss. Timing is wall-clock, not emulated frames: expect +-1-2 frames.
"""

import subprocess
import sys
import time
from pathlib import Path

import AppKit
import Quartz

ROOT = Path(__file__).resolve().parent.parent
OUT = ROOT / "target" / "pcsx2"

# macOS virtual key codes of the owner's bindings.
KEYS = {
    "up": 126, "right": 124, "down": 125, "left": 123,
    "triangle": 34, "circle": 37, "cross": 40, "square": 38,  # I L K J
    "select": 51, "start": 36,  # Backspace, Return
    "l1": 12, "l2": 18, "r1": 14, "r2": 20, "l3": 19, "r3": 21,  # Q 1 E 3 2 4
    "lup": 13, "lright": 2, "ldown": 1, "lleft": 0,  # W D S A
    "rup": 17, "rright": 4, "rdown": 5, "rleft": 3,  # T H G F
    "f1": 122, "f2": 120, "f3": 99, "f8": 100, "esc": 53,  # Esc: PCSX2 pause menu / return to game
}


def pid():
    out = subprocess.run(["pgrep", "-x", "PCSX2"], capture_output=True, text=True).stdout.split()
    if not out:
        sys.exit("PCSX2 is not running")
    return int(out[0])


def front(p):
    app = AppKit.NSRunningApplication.runningApplicationWithProcessIdentifier_(p)
    app.activateWithOptions_(AppKit.NSApplicationActivateIgnoringOtherApps)
    time.sleep(0.3)


def key(p, name, down):
    e = Quartz.CGEventCreateKeyboardEvent(None, KEYS[name], down)
    Quartz.CGEventPostToPid(p, e)


def tap(p, name, hold=0.05):
    key(p, name, True)
    time.sleep(hold)
    key(p, name, False)


def window():
    """The game window's (id, bounds); PCSX2 titles it after the running game."""
    w = next(w for w in Quartz.CGWindowListCopyWindowInfo(Quartz.kCGWindowListOptionOnScreenOnly, Quartz.kCGNullWindowID)
             if w.get("kCGWindowOwnerName") == "PCSX2" and w.get("kCGWindowName", "").startswith("Shinobido"))
    return w["kCGWindowNumber"], w["kCGWindowBounds"]


def shot(p, dest):
    """Capture the game window to `dest` (F8 snapshots came back half-written)."""
    dest.parent.mkdir(parents=True, exist_ok=True)
    subprocess.run(["screencapture", "-x", "-o", "-l", str(window()[0]), str(dest)], check=True)
    return dest


def load(p):
    """Load the current save-state slot (F3; F2 cycles slots, the OSD shows which)."""
    tap(p, "f3")
    time.sleep(1.0)


def play(p, script, name):
    events = []
    for line in Path(script).read_text().splitlines():
        line = line.split("#")[0].split()
        if line:
            events.append((int(line[0]), line[1], line[2:]))
    events.sort(key=lambda e: e[0])
    for _, cmd, args in events:
        if cmd not in ("down", "up", "tap", "shot", "load") or any(a not in KEYS for a in args if cmd in ("down", "up", "tap")):
            sys.exit(f"bad script line: {cmd} {args}")
    t0 = time.monotonic()
    held = set()
    try:
        for frame, cmd, args in events:
            wait = t0 + frame / 60 - time.monotonic()
            if wait > 0:
                time.sleep(wait)
            if cmd == "down":
                for a in args:
                    key(p, a, True)
                    held.add(a)
            elif cmd == "up":
                for a in args:
                    key(p, a, False)
                    held.discard(a)
            elif cmd == "tap":
                for a in args:
                    key(p, a, True)
                time.sleep(3 / 60)
                for a in args:
                    key(p, a, False)
            elif cmd == "shot":
                print(shot(p, OUT / name / f"{frame:05}.png"))
            elif cmd == "load":
                load(p)
                t0 = time.monotonic() - frame / 60  # loading takes wall time; keep script frames relative
    finally:
        for a in held:
            key(p, a, False)


def video(p, script, name):
    """`play` while macOS `screencapture` records the PCSX2 game window (real time, the screen's frame rate)."""
    b = window()[1]
    last = max(int(l.split()[0]) for l in Path(script).read_text().splitlines() if l.split("#")[0].split())
    dest = OUT / f"{name}.mov"
    dest.parent.mkdir(parents=True, exist_ok=True)
    dest.unlink(missing_ok=True)
    rect = f"{int(b['X'])},{int(b['Y'])},{int(b['Width'])},{int(b['Height'])}"
    rec = subprocess.Popen(["screencapture", "-x", "-V", str(last // 60 + 2), "-R", rect, str(dest)])
    time.sleep(0.5)  # recorder start-up
    play(p, script, name)
    rec.wait()
    print(dest)


def main():
    if len(sys.argv) < 2:
        sys.exit(__doc__)
    p = pid()
    front(p)
    cmd, arg = sys.argv[1], (sys.argv + [""])[2]
    if cmd == "shot" and arg:
        print(shot(p, OUT / f"{arg}.png"))
    elif cmd == "load":
        load(p)
    elif cmd == "play":
        play(p, arg, sys.argv[3] if len(sys.argv) > 3 else Path(arg).stem)
    elif cmd == "video":
        video(p, arg, sys.argv[3] if len(sys.argv) > 3 else Path(arg).stem)
    else:
        sys.exit(__doc__)


if __name__ == "__main__":
    main()
