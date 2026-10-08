#!/usr/bin/env python3
"""Overnight runner: Claude until its 5h window is near full, then Codex until its own is, then wait.

Usage: .venv/bin/python tools/night.py [HH:MM]     (default 07:00; night.sh wraps this in caffeinate)

Rules (thresholds below):
  - a worker runs only while its 5-hour usage is under NEAR; a running worker is killed when it crosses it
  - a worker whose weekly usage reaches WEEK is never used again this run; both gone -> exit
  - both 5h windows near full -> sleep until the earlier reset
  - a worker run that makes no commit is "idle"; all usable workers idle -> done or blocked, exit
Usage comes from the CLIs themselves: Claude's `rate_limit_event` (stream-json) and Codex's
`account/rateLimits/read` (app-server). Claude utilization is read as a fraction (0.4 = 40%).
"""
import json, subprocess, sys, threading, time
from datetime import datetime, timedelta
from pathlib import Path

NEAR, WEEK, POLL = 90, 80, 30  # percent 5h cutoff, percent weekly cutoff, watchdog seconds
END = float("inf")  # set in main(): a running worker is killed at this time
CLAUDE = "/Users/arthursantos/.claude/local/claude"
ROOT = Path(__file__).resolve().parent.parent
LOG = ROOT / "target/logs" / f"night-{datetime.now():%Y-%m-%d}.log"
PROTECTED = ["AGENTS.md", "CLAUDE.md", "NIGHT_PROMPT.md", "night.sh", "night-settings.json",
             "tools/night.py", ".claude", ".codex"]
NOTE = {
    "claude": "You are the Claude worker. Commit after each green step.",
    "codex": "You are the Codex worker. Do NOT run git: the runner commits when `cargo test -q` passes. "
             "Network is off; if you need a download, record a Blocker. Update PROGRESS.md before you stop.",
}


def log(msg):
    line = f"== {datetime.now():%H:%M:%S} {msg}"
    print(line, flush=True)
    with LOG.open("a") as f:
        f.write(line + "\n")


def git(*a):
    return subprocess.run(["git", *a], cwd=ROOT, capture_output=True, text=True).stdout.strip()


def commits():
    return int(git("rev-list", "--count", "HEAD") or 0)


def parse_claude(event):
    w = event["rate_limit_info"].get("unifiedWindows", {})
    five, week = w.get("five_hour", {}), w.get("seven_day", {})
    return dict(five=five.get("utilization", 0) * 100, week=week.get("utilization", 0) * 100,
                reset=five.get("resetsAt", time.time() + 1800))


def claude_usage():
    r = subprocess.run([CLAUDE, "-p", "ok", "--model", "haiku", "--output-format", "stream-json", "--verbose"],
                       cwd=ROOT, capture_output=True, text=True, stdin=subprocess.DEVNULL)
    for l in r.stdout.splitlines():
        if '"rate_limit_event"' in l:
            return parse_claude(json.loads(l))
    return dict(five=100, week=0, reset=time.time() + 1800)  # unknown: assume full, retry in 30 min


def codex_usage():
    p = subprocess.Popen(["codex", "app-server"], stdin=subprocess.PIPE, stdout=subprocess.PIPE,
                         stderr=subprocess.DEVNULL, text=True)
    try:
        for m in ({"id": 1, "method": "initialize", "params": {"clientInfo": {"name": "night", "version": "1"}}},
                  {"method": "initialized"}, {"id": 2, "method": "account/rateLimits/read"}):
            p.stdin.write(json.dumps(m) + "\n")
            p.stdin.flush()
        for l in p.stdout:
            d = json.loads(l)
            if d.get("id") == 2:
                rl = d["result"]["rateLimits"]
                five, week = rl["primary"], rl["secondary"]
                return dict(five=five["usedPercent"], week=week["usedPercent"], reset=five["resetsAt"])
    except Exception as e:
        log(f"codex usage read failed: {e}")
    finally:
        p.kill()
    return dict(five=100, week=0, reset=time.time() + 1800)


USAGE = {"claude": claude_usage, "codex": codex_usage}


def run(worker, prompt, seen):
    """Run one worker turn; kill it if its usage crosses a limit. Returns (rc, killed)."""
    if worker == "claude":
        cmd = [CLAUDE, "-p", prompt, "--model", "sonnet", "--permission-mode", "auto", "--settings",
               "./night-settings.json", "--output-format", "stream-json", "--verbose", "--max-turns", "60"]
    else:
        cmd = ["codex", "exec", "--json", "-C", str(ROOT), "-s", "workspace-write", prompt]
    p = subprocess.Popen(cmd, cwd=ROOT, stdin=subprocess.DEVNULL, stdout=subprocess.PIPE,
                         stderr=subprocess.STDOUT, text=True)
    done, killed = threading.Event(), []

    def watch():
        while not done.wait(POLL):
            u = seen.get("claude") if worker == "claude" else codex_usage()
            if time.time() >= END or (u and (u["five"] >= NEAR or u["week"] >= WEEK)):
                log(f"{worker} stopped: end time or usage limit ({u and (u['five'], u['week'])})")
                killed.append(1)
                p.terminate()
                if not done.wait(20):
                    p.kill()  # it ignored SIGTERM
                return

    threading.Thread(target=watch, daemon=True).start()
    with LOG.open("a") as f:
        for l in p.stdout:
            if worker == "claude" and '"rate_limit_event"' in l:
                seen["claude"] = parse_claude(json.loads(l))
            if worker == "codex" or '"type":"result"' in l or '"rate_limit_event"' in l:
                f.write(l[:600].rstrip() + "\n")
    done.set()
    return p.wait(), bool(killed)


def commit_codex():
    """Codex can't run git. Restore protected files, commit only if the tests pass."""
    for path in PROTECTED:  # one call each: a path git doesn't track would abort a combined checkout
        git("checkout", "--", path)
    if not git("status", "--porcelain"):
        return
    if subprocess.run(["cargo", "test", "-q"], cwd=ROOT, capture_output=True).returncode == 0:
        git("add", "-A")
        git("commit", "-m", f"Codex night step {datetime.now():%H:%M}")
    else:
        git("stash", "push", "-u", "-m", f"codex-red-{datetime.now():%H%M}")
        log("codex left failing tests: work stashed (git stash list)")


def main():
    hhmm = sys.argv[1] if len(sys.argv) > 1 else "07:00"
    end = datetime.now().replace(hour=int(hhmm[:2]), minute=int(hhmm[3:]), second=0, microsecond=0)
    if end < datetime.now():
        end += timedelta(days=1)
    global END
    end = END = end.timestamp()
    LOG.parent.mkdir(parents=True, exist_ok=True)
    dirty = git("status", "--porcelain", "--", *PROTECTED)  # commit_codex restores these from HEAD
    if dirty:
        sys.exit(f"commit your changes to the protected files first, or the run would revert them:\n{dirty}")
    base = (ROOT / "NIGHT_PROMPT.md").read_text()
    dead, idle, errs, seen = set(), set(), {"claude": 0, "codex": 0}, {}
    log(f"start, until {hhmm}; near-full {NEAR}%, weekly stop {WEEK}%")
    while time.time() < end:
        u = {w: USAGE[w]() for w in USAGE if w not in dead}
        for w, x in u.items():
            log(f"{w}: {x['five']:.0f}% 5h, {x['week']:.0f}% week")
            if x["week"] >= WEEK:
                dead.add(w)
                log(f"{w}: weekly limit reached, not using it again")
        u = {w: x for w, x in u.items() if w not in dead}
        if not u:
            log("both weekly limits reached: end")
            return
        live = [w for w in u if w not in idle]  # idle = last run made no commit
        if not live:
            log("every usable worker made no commit: done or blocked, end")
            return
        todo = [w for w in live if u[w]["five"] < NEAR]
        if not todo:
            wake = min(u[w]["reset"] for w in live) + 60
            if wake >= end:
                log("next reset is after the end time: end")
                return
            log(f"5h windows near full, sleeping until {datetime.fromtimestamp(wake):%H:%M}")
            time.sleep(max(wake - time.time(), 60))
            continue
        w = todo[0]  # claude first, codex after
        before = commits()
        log(f"running {w}")
        seen.pop("claude", None)
        rc, killed = run(w, f"{base}\n\n{NOTE[w]}", seen)
        if w == "codex":
            commit_codex()
        # a failure that still committed (e.g. --max-turns reached) is progress, not an error
        errs[w] = errs[w] + 1 if rc and not killed and commits() == before else 0
        if errs[w] >= 3:
            dead.add(w)
            log(f"{w}: 3 failed runs in a row, not using it again")
        if commits() > before:
            idle.clear()
        elif not killed:
            idle.add(w)
        log(f"{w} exit {rc}{' (killed at limit)' if killed else ''}, commits +{commits() - before}")
    log("end time reached")


if __name__ == "__main__":
    main()
