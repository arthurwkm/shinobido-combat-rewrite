#!/usr/bin/env python3
"""Overnight runner: Claude until its 5h window is near full, then Codex until its own is, then wait.

Usage: .venv/bin/python tools/night.py [HH:MM]     (default 07:00; night.sh wraps this in caffeinate)

Rules (thresholds below):
  - a worker runs only while its 5-hour usage is under NEAR; a running worker is killed when it crosses it
  - a worker whose weekly usage reaches WEEK is never used again this run; both gone -> exit
  - both 5h windows near full -> sleep until the earlier reset
  - a worker run that makes no commit is "idle"; all usable workers idle -> done or blocked, exit
  - unanswered Blockers in PROGRESS.md (at start for ASK_START seconds, and when all workers go idle
    until the end time): beep and ask each one at the console, one at a time. Answers are written under their blockers as
    `  - Owner: ...` in one commit, then the workers start. Enter skips one; no terminal skips all.
    Waiting uses no tokens.
Usage comes from the CLIs themselves: Claude's `rate_limit_event` (stream-json) and Codex's
`account/rateLimits/read` (app-server). Claude utilization is read as a fraction (0.4 = 40%).
"""
import json, os, select, subprocess, sys, termios, threading, time
from datetime import datetime, timedelta
from pathlib import Path

NEAR, WEEK = 90, 80  # percent 5h cutoff, percent weekly cutoff
POLL_MIN, POLL_MAX = 30, 300  # watchdog seconds: 300 at 40+ points of headroom, 150 at 10, 30 at 0
ASK_START = 600  # seconds to wait for blocker answers at start before the night goes on without them
END = float("inf")  # set in main(): a running worker is killed at this time
CLAUDE = "/Users/arthursantos/.claude/local/claude"
ROOT = Path(__file__).resolve().parent.parent
LOG = ROOT / "target/logs" / f"night-{datetime.now():%Y-%m-%d}.log"
PROTECTED = ["AGENTS.md", "CLAUDE.md", "NIGHT_PROMPT.md", "night.sh", "night-settings.json",
             "tools/night.py", ".claude", ".codex"]
NOTE = {
    "claude": "You are the Claude worker. Commit after each green step.",
    "codex": "You are the Codex worker. Do not commit or stage (`.git` is read-only in your sandbox); "
             "read-only git (log, diff, status) is fine. The runner commits when `cargo test -q` passes. "
             "Network is off: if a step needs a new crate or PyPI package, do not record a Blocker; append "
             "`(needs download: <name>, for Claude)` to the item's line in PROGRESS.md and stop. "
             "Update PROGRESS.md before you stop.",
}


def log(msg):
    line = f"== {datetime.now():%H:%M:%S} {msg}"
    print(line, flush=True)
    with LOG.open("a") as f:
        f.write(line + "\n")


def git(*a):
    return subprocess.run(["git", *a], cwd=ROOT, capture_output=True, text=True).stdout.strip()


def poll_delay(headroom):
    return min(POLL_MAX, 100 + 5 * headroom) if headroom >= 10 else max(POLL_MIN, 30 + 12 * headroom)


def commits():
    return int(git("rev-list", "--count", "HEAD") or 0)


def parse_claude(event):
    w = event["rate_limit_info"].get("unifiedWindows", {})
    five, week = w.get("five_hour", {}), w.get("seven_day", {})
    return dict(five=five.get("utilization", 0) * 100, week=week.get("utilization", 0) * 100,
                reset=five.get("resetsAt", time.time() + 1800))


def claude_usage():
    try:
        r = subprocess.run([CLAUDE, "-p", "ok", "--model", "haiku", "--output-format", "stream-json", "--verbose"],
                           cwd=ROOT, capture_output=True, text=True, stdin=subprocess.DEVNULL, timeout=120)
    except subprocess.TimeoutExpired:
        log("claude usage read timed out")
        return dict(five=100, week=0, reset=time.time() + 1800, unknown=True)
    for l in r.stdout.splitlines():
        if '"rate_limit_event"' in l:
            return parse_claude(json.loads(l))
    return dict(five=100, week=0, reset=time.time() + 1800, unknown=True)  # assume full, retry in 30 min


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
    return dict(five=100, week=0, reset=time.time() + 1800, unknown=True)


USAGE = {"claude": claude_usage, "codex": codex_usage}


def run(worker, prompt, seen):
    """Run one worker turn; kill it if its usage crosses a limit. Returns (rc, killed)."""
    if worker == "claude":
        cmd = [CLAUDE, "-p", prompt, "--model", "sonnet", "--permission-mode", "auto", "--settings",
               "./night-settings.json", "--output-format", "stream-json", "--verbose", "--max-turns", "300"]
    else:
        cmd = ["codex", "exec", "--json", "-C", str(ROOT), "-s", "workspace-write", prompt]
    p = subprocess.Popen(cmd, cwd=ROOT, stdin=subprocess.DEVNULL, stdout=subprocess.PIPE,
                         stderr=subprocess.STDOUT, text=True)
    done, killed = threading.Event(), []

    def watch():
        delay = 60
        while not done.wait(delay):
            u = seen.get("claude") if worker == "claude" else codex_usage()
            if u and u.get("unknown"):  # a failed or timed-out read is not evidence of a full window
                u, delay = None, POLL_MIN  # check again soon, for either worker
            elif u:
                delay = poll_delay(min(NEAR - u["five"], WEEK - u["week"]))
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


def blockers():
    """Unanswered Blockers in PROGRESS.md: top-level `- ` lines not followed by an `  - Owner:` line."""
    text = (ROOT / "PROGRESS.md").read_text()
    if "## Blockers\n" not in text:
        return []
    lines = [l for l in text.split("## Blockers\n", 1)[1].split("\n## ", 1)[0].splitlines() if l.strip()]
    return [l for i, l in enumerate(lines) if l.startswith("- ")
            and not (i + 1 < len(lines) and lines[i + 1].startswith("  - Owner:"))]


def read_answer(until):
    """One line from the console, beeping every 10 min; "" at `until`."""
    termios.tcflush(sys.stdin, termios.TCIFLUSH)  # drop keys typed while the workers ran
    while time.time() < until:
        subprocess.run(["afplay", "/System/Library/Sounds/Glass.aiff"])
        if select.select([sys.stdin], [], [], min(600, max(until - time.time(), 0)))[0]:
            return os.read(sys.stdin.fileno(), 65536).decode(errors="replace").strip()  # unbuffered, so select stays true
    return ""


def foreground():
    """stdin is a terminal and we are its foreground job (a background job touching it gets stopped)."""
    try:
        return sys.stdin.isatty() and os.tcgetpgrp(sys.stdin.fileno()) == os.getpgrp()
    except OSError:  # a terminal that is not ours
        return False


def ask_owner(until):
    """Ask about each unanswered Blocker in turn until `until`, then write all answers in one commit.
    True if any answer was written, so the workers start once with every answer in place."""
    todo = blockers()
    if not todo or not foreground():
        return False
    log(f"{len(todo)} blockers need the owner, waiting at the console")
    answers = {}
    for i, q in enumerate(todo):
        print(f"\nBlockers remaining: {len(todo) - i}\n{q}\nAnswer in one line (Enter to skip):", flush=True)
        if a := read_answer(until):
            answers[q] = a
        if time.time() >= until:
            break
    if not answers:
        log("no answers to the blockers")
        return False
    path = ROOT / "PROGRESS.md"
    head, section = path.read_text().split("## Blockers\n", 1)  # answers go only inside Blockers
    lines = section.split("\n")
    for q, a in answers.items():
        lines.insert(lines.index(q) + 1, f"  - Owner: {a}")
    path.write_text(head + "## Blockers\n" + "\n".join(lines))
    git("commit", "-qm", "PROGRESS: owner answers to Blockers", "--", "PROGRESS.md")
    log(f"owner answered {len(answers)} of {len(todo)} blockers")
    return True


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
    ask_owner(min(time.time() + ASK_START, end))
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
        if not live and ask_owner(end):
            idle.clear()
            continue
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
