"""Dry run of night.py's control loop with fake usage, workers and clock. No CLI, git or sleeping.
Run: .venv/bin/python tools/test_night.py"""
import sys, tempfile
from pathlib import Path
sys.path.insert(0, str(Path(__file__).parent))
import night

now = [1_000_000.0]
night.time.time = lambda: now[0]
night.time.sleep = lambda s: now.__setitem__(0, now[0] + s)
night.LOG = Path(tempfile.mkdtemp()) / "t.log"
night.git = lambda *a: ""            # protected files clean, nothing to commit
night.commit_codex = lambda: None
(night.ROOT / "NIGHT_PROMPT.md").exists() or sys.exit("needs NIGHT_PROMPT.md")

used = {"claude": 50, "codex": 20}   # 5h percent
reset = now[0] + 3600
ran, n = [], [0]
night.USAGE = {w: (lambda w=w: dict(five=used[w], week=10, reset=reset)) for w in used}
night.commits = lambda: n[0]

def fake_run(w, prompt, seen):
    ran.append(w)
    n[0] += 1                         # each run makes a commit
    used[w] = 95                      # and fills its window
    if len(ran) == 3:                 # after the wait the windows have reset
        pass
    return 0, True

def fake_sleep(s):
    now[0] += s
    if s >= 60 and len(ran) == 2:     # the long wait: both windows reset
        used.update(claude=0, codex=0)

night.time.sleep = fake_sleep
night.run = fake_run
sys.argv = ["night.py", "07:00"]
night.END = float("inf")
import datetime
end = now[0] + 3 * 3600
night.datetime = type("D", (datetime.datetime,), {"now": classmethod(lambda c, tz=None: datetime.datetime.fromtimestamp(now[0]))})
real_main_end = end
# main() computes END from HH:MM; make it land 3h ahead of the fake clock
h = datetime.datetime.fromtimestamp(end)
sys.argv[1] = f"{h:%H:%M}"
night.main()
print(ran)
assert ran[:3] == ["claude", "codex", "claude"], ran
print("ok: claude -> codex -> wait for reset -> claude")
