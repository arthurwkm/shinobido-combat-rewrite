# AGENTS.md — Shinobido: Way of the Ninja, combat rewrite

Goal: recreate how the player (Kagetora) moves and fights in Shinobido: Way of the Ninja
(PS2, Acquire, 2005), with timings, motion and parameters read from the game's own files instead of
tuned by eye, and a playable viewer like the reference project's. **This is not a decompilation.**

In scope:
- **Kagetora's whole moveset**, each action with the game's own animation: walk, run, run stop,
  jumps (including the high jump onto rooftops), flips, rolls/evasion, attacks and chains, guard,
  hit reactions, ledge grab/hang/climb, wall run up, wall hug and corner peek, crouch movement,
  picking up, carrying and throwing crates and bodies, items, and every other player action found
  in the motion data. The player's motions span several suffix sets (`01`, `02`, `05`, `06`, `27`,
  reached through the cancel graph), not only `01`.
- **One enemy** (the most common foot soldier): its own model and motions, a simple AI driven by
  the game's AI files (`AI\`) where readable, attacks that hit the player, hit reactions, death.
- **Combat interactions**: lock-on (target select, strafing, lock-on camera), hits both ways, guard.
- **Executions**: every player execution (`motion\harakiri\`, paired player/enemy motions), the
  conditions that allow each, and the visual cue the game shows when an enemy can be executed.
- **Stealth**: enemy awareness (sight, sound, alert states) and the on-screen stealth meter it drives.
- **Viewer** (Bevy), like `src/` of the reference and its clip `videos/eldenring viewer.mp4`:
  - Third-person camera behind and above the player, as in the Shinobido clips; orbit with mouse/right
    stick, lock-on camera. Controls mapped to the game's PS2 pad layout (gamepad plus keyboard/mouse).
  - Sandbox look of the reference: flat grid ground, plain grey primitive blocks, simple lighting.
    Terrain holds what the moveset needs: tall walls (wall run, wall hug, corners to peek around),
    tiled rooftops with ledges reachable by the high jump, stairs, slopes, a narrow walled corridor
    (the combat clip's arena), and crates to climb, carry and throw.
  - Kagetora's own skinned model from `model\KAGETORA_*.rws` playing the extracted animations
    (fallback: primitive shapes on the game skeleton, as the reference's mannequin does), and the
    enemy's own model.
  - HUD in two parts. Debug readout like the reference: player bars and a hit log top-left; action,
    frame/length, speed, height, lock-on and queued input top-right; target name and health bar at the
    bottom. Game HUD like the Shinobido clips: health bar top-left, the eye awareness icon top-centre
    (its colour shows the alert state, the bar under it is the stealth meter), the ki compass top-right
    pointing at enemies, and the execution cue.

Out of scope: other enemy types, bosses, missions, story, cutscenes, full game stages, sound, menus.

Reference design (read-only, never edit): `/Users/arthursantos/arthurwkm_workspace/ELDEN-RING-Combat-Rewrite`.
Copy its architecture: `src/sim/` is the whole game as an engine-free state machine, tested
headless with `cargo test`; Python tools in `tools/` read the game files and generate
`src/sim/extracted.rs`; every value not read from the game is marked `ESTIMATE` in `src/sim/data.rs`.
The viewer copies its `src/` layout (camera, input, rig, anim, hud, view). `videos/` holds a clip of
the reference viewer: match that look and feel.

The game disc image (`*.iso`) is in this folder. It is the only source of game data.
What is known about it is in `docs/formats.md`; read it before any format work and keep it current.

## Method: where values come from, in this order
1. **Data files in `VOLUME.DAT`** (motions `motion\*.nmt`, executions `motion\harakiri\`, models,
   `camera\`, `AI\`, weapon tables). Use `tools/volume.py`.
2. **Targeted lookups in the executable `SCES_539.31`**, only for a specific value the files don't hold
   (e.g. run speed, gravity, cancel rules, sight range). Find it by searching for the float constant, or for code
   that references a known string/file name, with `rabbitizer`/`spimdisasm` (installed in `.venv`).
   Disassemble only the functions you need; write findings to `docs/formats.md` with addresses.
3. **Gameplay videos in `videos/`** (owner-supplied: `shinobido movement/combat/stealth.mov`, 60 fps), for
   what only shows on screen: which animation plays for an input, camera behaviour, effects, HUD. Read
   frames with `ffmpeg` into `target/` and look at small tiles; never commit videos or frames.
   The emulator (owner's PCSX2 at `~/Downloads/PCSX2-v2.6.3.app`) is for sessions with the owner present
   only: the night sandbox blocks `screencapture` and `osascript` (tested 2026-10-08). Unattended, use the
   videos; when they don't settle something, add a Request in `PROGRESS.md` and continue with an `ESTIMATE`.
4. **`ESTIMATE`** in `src/sim/data.rs`, with a one-line reason.

Not allowed: whole-program decompilation, splat/matching-decomp projects, m2c, decoding `ACQ.BIN`,
movies (`PAL/`), sound (`SOUND/`), or the script VM.

There is no measured ground truth from the running game. Confirm a format hypothesis by consistency
across many files instead (e.g. header length equals keyframe duration, left/right variants mirror,
values land on 1/60 s). Mark each statement in `docs/formats.md` confirmed (with how) or hypothesis.

## Disk
- Read from the ISO by offset (`tools/iso.py`, `tools/volume.py`); never extract the whole disc.
- Extract into `extracted/` only the files a step needs (e.g. `motion\`, `model\KAGETORA*`, `weapon\`).
- Check `df -h .` before large extractions or builds; stop with a Blocker below 5 GB free.

## Done means
- `cargo test -q` passes and `.venv/bin/python tools/compare.py` reports every extracted action within tolerance.
- Viewer items: `cargo build -q --bin viewer` succeeds and `cargo clippy -q --all-targets` is clean.
- Viewer look: run `target/debug/viewer --shot target/shots/<name>.png` as a plain command (no pipes,
  `&&` or redirects: only that exact form runs outside the night sandbox, which has no GPU). It must exit
  by itself after saving. Compare the PNG with frames of `videos/eldenring viewer.mp4` and the Shinobido
  clips, and name the shot in the item's evidence. Never run the viewer without `--shot`: it never exits.
- Show evidence: the command you ran and the last lines of its output. Never claim success without it.

## Loop
1. Session start: run `cargo test -q`, `cargo clippy -q --all-targets` and `.venv/bin/python tools/compare.py`.
   If anything fails, fixing it is the first job (commit the fix), before any new item.
2. Read `PROGRESS.md` (skip its Log). Work on the first unchecked item of the current milestone.
   Each checklist item is one feature.
3. Write or extend a failing test, make it pass, run the full suite. For format work with no Rust yet,
   the test is a check in the Python tool that fails when the parse is wrong.
4. Review before every commit: read your own `git diff` for bugs (wrong units, off-by-one frames,
   unhandled states, dead code, panics on missing data) and fix them. If you notice a bug in earlier
   code, fix it too, with a test, and say so in the Log.
5. Commit locally after each green step (`git commit`), once clippy is warning-free.
   Update `PROGRESS.md` in the same commit: the item's done (with evidence) and next. Never push; there is no remote.
6. Feature boundary: when an item is checked off, add one Log line, commit and stop. The next
   feature starts in a fresh session. Otherwise continue with the next step; never stop just to report.
   Before checking off a milestone's last item, do one review and cleanup pass over that milestone's
   code (bugs, dead code, duplication), tests green.
7. Decide, don't ask. Every design or data decision is yours: take your recommendation, note it in the
   Log, mark guessed values `ESTIMATE`. If the same approach fails 3 times, close the item with the best
   `ESTIMATE`, record what was tried in `docs/formats.md`, and move on.
8. Blockers are only for these two cases: the next step would need work outside the project goal
   (including installing or configuring system software), or changing files outside this folder.
   Record it under Blockers in `PROGRESS.md`, commit and stop.
9. If the milestone list is done, or every remaining item is blocked, exit without a commit:
   the runner reads a session with no commit as "done or blocked" and ends the night.

## Scope (hard limits)
- Only create or change files inside this folder. The `.iso` is read-only: never modify, move or rename it.
- Python only through `.venv/` in this folder (`.venv/bin/python`). PyPI packages go into `.venv` only
  and are listed in `requirements.txt`.
- Rust crates only from crates.io, declared in `Cargo.toml`. Bevy only in the viewer, never in `src/sim/`.
- Never install, upgrade or configure system software: no `brew`, `sudo`, `rustup`, global npm/pip,
  Python or Rust version changes, emulators, or edits to shell profiles or OS settings.
- Never edit agent configuration: `.claude/`, `.codex/`, `night-settings.json`, `night.sh` unless when explicitly prompted to,
  `NIGHT_PROMPT.md`, `CLAUDE.md`, `AGENTS.md`, or anything in the home folder.
- Network: crates.io and PyPI for this project's dependencies; web search and docs only to research
  PS2, RenderWare or Acquire file formats. No accounts, payments, sign-ups, uploads or unrelated sites.
- Game-derived output (extracted files, tables, assets, listings, video frames) lives in `extracted/`,
  `target/` or the generated paths in `.gitignore` and is never committed. Code and format documentation
  are committed; documentation describes layouts and may quote a few bytes or values, never whole tables.

## Token discipline
- Builds: dev profile only (never `--release`: disk is tight, deps are already optimized). A Bevy rebuild
  can pass 2 minutes: give such commands a 10-minute timeout instead of retrying them.
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
  that each leave the suite green; finishing a step is not a reason to stop (see Loop step 6).
  Codex being out of quota is never a Blocker: do the lookup yourself.
