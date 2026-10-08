# AGENTS.md — Shinobido: Way of the Ninja, combat rewrite

Goal: recreate the player's (Kagetora's) movement and combat from Shinobido: Way of the Ninja
(PS2, Acquire, 2005) as a pure simulation in Rust, with timings, motion and parameters read from
the game's own files instead of tuned by eye. **This is not a decompilation.** Enemies, AI, stealth,
missions, story, sound and rendering are out of scope unless `PROGRESS.md` says otherwise.

Reference design (read-only, never edit): `/Users/arthursantos/arthurwkm_workspace/ELDEN-RING-Combat-Rewrite`.
Copy its architecture: `src/sim/` is the whole game as an engine-free state machine, tested
headless with `cargo test`; Python tools in `tools/` read the game files and generate
`src/sim/extracted.rs`; every value not read from the game is marked `ESTIMATE` in `src/sim/data.rs`.

The game disc image (`*.iso`) is in this folder. It is the only source of game data.
What is known about it is in `docs/formats.md`; read it before any format work and keep it current.

## Method: where values come from, in this order
1. **Data files in `VOLUME.DAT`** (motions `motion\*.nmt`, player model, weapon tables). Use `tools/volume.py`.
2. **Targeted lookups in the executable `SCES_539.31`**, only for a specific value the files don't hold
   (e.g. run speed, gravity, cancel rules). Find it by searching for the float constant, or for code
   that references a known string/file name, with `rabbitizer`/`spimdisasm` (installed in `.venv`).
   Disassemble only the functions you need; write findings to `docs/formats.md` with addresses.
3. **`ESTIMATE`** in `src/sim/data.rs`, with a one-line reason.

Not allowed: whole-program decompilation, splat/matching-decomp projects, m2c, decoding `ACQ.BIN`,
movies (`PAL/`), sound (`SOUND/`), or the script VM, unless a Blocker in `PROGRESS.md` shows a needed
value lives there and the owner approves.

There is no measured ground truth from the running game. Confirm a format hypothesis by consistency
across many files instead (e.g. header length equals keyframe duration, left/right variants mirror,
values land on 1/60 s). Mark each statement in `docs/formats.md` confirmed (with how) or hypothesis.

## Disk
- Read from the ISO by offset (`tools/iso.py`, `tools/volume.py`); never extract the whole disc.
- Extract into `extracted/` only the files a step needs (e.g. `motion\`, `model\KAGETORA*`, `weapon\`).
- Check `df -h .` before large extractions or builds; stop with a Blocker below 5 GB free.

## Done means
- `cargo test -q` passes and `python tools/compare.py` reports every extracted action within tolerance.
- Show evidence: the command you ran and the last lines of its output. Never claim success without it.

## Loop
1. Read `PROGRESS.md` first (skip its Log). Work on the first unblocked unchecked item of the current
   milestone. Each checklist item is one feature.
2. Write or extend a failing test, make it pass, run the full suite. For format work with no Rust yet,
   the test is a check in the Python tool that fails when the parse is wrong.
3. Commit locally after each green step (`git commit`), once `cargo clippy -q --all-targets` is warning-free.
   Update `PROGRESS.md` in the same commit: the item's done (with evidence) and next. Never push; there is no remote.
4. Feature boundary: when an item is checked off or blocked, add one Log line, commit and stop. The next
   feature starts in a fresh session. Otherwise continue with the next step; never stop just to report.
   Before checking off a milestone's last item, do one cleanup pass over that milestone's code
   (dead code, duplication), tests green.
5. If the same approach fails 3 times, record it under Blockers in `PROGRESS.md`; that blocks the item
   (step 4: commit and stop). Anything that needs the owner (installing software, a decision, missing data) is a
   Blocker, not a workaround.
6. If at session start every remaining item is blocked, or the milestone list is done, exit without a
   commit: the runner reads a session with no commit as "done or blocked" and ends the night.
   Do not keep trying: unused usage is worth more than a stuck session.

## Scope (hard limits)
- Only create or change files inside this folder. The `.iso` is read-only: never modify, move or rename it.
- Python only through `.venv/` in this folder (`.venv/bin/python`). PyPI packages go into `.venv` only
  and are listed in `requirements.txt`.
- Rust crates only from crates.io, declared in `Cargo.toml`. No Bevy until the viewer milestone.
- Never install, upgrade or configure system software: no `brew`, `sudo`, `rustup`, global npm/pip,
  Python or Rust version changes, emulators, or edits to shell profiles or OS settings.
- Never edit agent configuration: `.claude/`, `.codex/`, `night-settings.json`, `night.sh` unless when explicitly prompted to,
  `NIGHT_PROMPT.md`, `CLAUDE.md`, `AGENTS.md`, or anything in the home folder.
- Network: crates.io and PyPI for this project's dependencies; web search and docs only to research
  PS2, RenderWare or Acquire file formats. No accounts, payments, sign-ups, uploads or unrelated sites.
- Game-derived output (extracted files, tables, assets, listings) lives in `extracted/` or the
  generated paths in `.gitignore` and is never committed. Code and format documentation are committed;
  documentation describes layouts and may quote a few bytes or values, never whole tables.

## Token discipline
- Quiet commands: `cargo build -q`, `cargo test -q`. Send long output to `target/logs/<name>.log`
  and read only the errors or the tail.
- Never print whole binary, generated or extracted files. Use `rg`, `head -c`, hex dumps of small
  ranges, and read file ranges, not whole large files. Find the `docs/formats.md` section you need with `rg`.
- Use a subagent for broad searches or format surveys; keep the main session for edits.
- Smallest change that passes the test. No speculative abstractions or features outside `PROGRESS.md`.
- Status notes: one line per step. The detailed record goes in `PROGRESS.md`.

## Claude and Codex
- `tools/night.py` alternates the two workers by usage: Claude until its 5-hour window is near full, then Codex,
  then it waits for a reset. You are one of them; do not hand work to the other yourself.
- Codex does not commit or stage (read-only git is fine): the runner commits its work when `cargo test -q` passes. Claude commits as in the Loop.
- Both workers keep `PROGRESS.md` current: the next worker starts from it with no other memory.
- Either worker may do the long, token-heavy work (format surveys, executable lookups), split into bounded steps
  that each leave the suite green; finishing a step is not a reason to stop (see Loop step 4).
  Codex being out of quota is never a Blocker: do the lookup yourself.
