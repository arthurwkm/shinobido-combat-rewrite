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
- **One enemy** (the most common foot soldier): its own skeleton and motions, a simple AI driven by
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
  - Characters are simple mannequins like the reference's: primitive shapes (capsules, spheres, boxes)
    on the game's own skeleton, playing the extracted animations, limbs joined at the joints, roughly
    the clip's proportions, a plain katana prop. Kagetora and the soldier differ by colour. No textured
    or skinned game meshes (owner decision 2026-10-08).
  - HUD in two parts. Debug readout like the reference: player bars and a hit log top-left; action,
    frame/length, speed, height, lock-on and queued input top-right; target name and health bar at the
    bottom. Game HUD like the Shinobido clips: health bar top-left, the eye awareness icon top-centre
    (its colour shows the alert state, the bar under it is the stealth meter), the ki compass top-right
    pointing at enemies, and the execution cue.

Out of scope: other enemy types, bosses, missions, story, cutscenes, full game stages, sound, menus.

## The spirit of the goal
The checklist serves the goal, not the other way round. The goal is met when someone who knows the
game picks up the pad in the viewer and Kagetora **moves, fights and sneaks like he does in the
Shinobido clips**, with every listed action reachable from the controls in the situation where the game
uses it. Treat these as not done, whatever the checkbox says:
- an action that only plays as a canned clip from a test or a debug call, not from the pad in context
  (a wall hug happens at a wall, a ledge grab when a ledge is in reach, a carry after picking something up);
- a feature never seen working in the viewer (built and unit-tested is not enough);
- a look that differs visibly from the clips or the reference viewer (wrong pose, wrong camera, missing HUD
  element, a mannequin whose pose doesn't read like the clip) without a recorded reason;
- an `ESTIMATE` that could still be read from the files or the executable but wasn't tried three times;
- a player action with no row in `docs/emucompare.md` (not compared with the running game in PCSX2), or a row
  whose verdict is "differs" with no fix and no recorded reason.
When the checklist and the spirit disagree, follow the spirit and add the missing work as items.

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
3. **The running game in PCSX2** (the owner leaves it open with Shinobido loaded in an open area): the ground
   truth for anything you can play — which input does what, timings, heights, distances. See "Emulator A/B" below.
4. **Gameplay videos in `videos/`** (owner-supplied, 60 fps: `shinobido movement/combat/stealth.mov`,
   `shinobido all attacks.mp4` with on-screen button icons, `shinobido lock in strafe fight.mov`,
   `shinobido lock in lock off stealth kill from above.mov`, `shinobido run jump.mov`), for what you cannot
   reproduce in PCSX2 (soldier fights, kills, the HUD in play). Read frames with `ffmpeg` into `target/` and look
   at small tiles; never commit videos or frames.
5. **`ESTIMATE`** in `src/sim/data.rs`, with a one-line reason.

Not allowed: whole-program decompilation, splat/matching-decomp projects, m2c, decoding `ACQ.BIN`,
movies (`PAL/`), sound (`SOUND/`), or the script VM.

Confirm a format hypothesis by consistency across many files (e.g. header length equals keyframe duration,
left/right variants mirror, values land on 1/60 s), and where it shows in play, by an emulator A/B. Mark each
statement in `docs/formats.md` confirmed (with how) or hypothesis. Worked example (2026-10-09): the jump's
launch 120.0 (exe) with 1 game unit = 0.84/9.5 m predicted 0.76 s air / 1.49 m apex; PCSX2 showed 0.72 s / ~1.5 m.

## Emulator A/B (the main check for movement and combat)
Every player action is checked against the real game: the same pad script in PCSX2 and in the viewer, frames
side by side, measured, fixed, and a row in `docs/emucompare.md`. `docs/movelist.md` is the complete move list
with each move's status; every `todo` row needs an A/B. Tools (all run unattended; `tools/pcsx2.py` is excluded
from the sandbox in `night-settings.json`):
- `.venv/bin/python tools/pcsx2.py video <script> <name>` plays a timed pad script in PCSX2 (key events to the
  owner's bindings) while `screencapture` records the game window to `target/pcsx2/<name>.mov` (60 fps capture;
  the PAL game renders 25 fps). Also `shot <name>` (window capture) and `play <script>`.
  Script lines: `<frame> down|up|tap <buttons>`, `<frame> shot`, `<frame> load`; frames are 60 Hz from the start;
  buttons `square cross circle triangle l1 l2 r1 r2 l3 r3 start select up down left right lup ldown lleft lright
  rup rdown rleft rright space esc`. Keyboard sticks are full tilt only (no walk). The recording lasts until 2 s
  after the last line: end a script with a late `shot` line to record longer.
- PCSX2 pauses when it loses focus; the tool brings it forward and taps Space when two captures are identical.
  Never tap `esc` in a script (pause menu). If a recording shows another app or a frozen frame, re-run it.
- Positions: there is no reset, Kagetora stays where the last script left him. Take a `shot` first to see
  where he is; plan scripts that work from there (open grass field with a hut, stone walls, a roof to climb).
  Never save over the owner's save-state slots.
- `.venv/bin/python tools/emucmp.py <game.mov> <viewer rec dir> <out.jpg> <tick>...` tiles game vs viewer at the
  same script ticks (viewer: `target/debug/viewer --record target/rec/<name> --script <file> --solo --every <n>`).
  For timing, sample the game video at 25 fps (`ffmpeg -vf fps=25,...,drawtext=text='%{n}'`) and count frames.
- Scripts live in `tools/scripts/emu/<name>.txt` (PCSX2) and `<name>_viewer.txt` (viewer copy), committed.
- Input facts found so far (keep them): R1 is hide/crouch and L1 is lock-on/block (front view of
  `img/shinobido controls.png`, confirmed in PCSX2 and by the PAL controls listing); the backward somersault is
  Cross pressed together with the stick pulled back; crouch-walking off an edge catches the ledge; Cross while
  hanging climbs. The owner's attack table in `docs/controls.md` writes some combos with "R1": re-check each in
  PCSX2 before trusting it.

## Owner decisions (do not revisit)
- Camera: the reference project's camera (`ELDEN-RING-Combat-Rewrite/src/camera.rs`), not Shinobido's.
- Target switch: Elden Ring's sideways flick (Shinobido has none).
- Time base: keep 60 Hz although the disc is PAL (25 fps game).
- Mannequins: primitive shapes on the game skeleton at Kagetora's model height (1.57 m); no textured meshes.
- Viewer modes: `--solo` (Kagetora alone, for movement A/B), `--one` (one soldier), `--patrol`.

## Disk
- Read from the ISO by offset (`tools/iso.py`, `tools/volume.py`); never extract the whole disc.
- Extract into `extracted/` only the files a step needs (e.g. `motion\`, `model\KAGETORA*`, `weapon\`).
- Check `df -h .` before large extractions or builds; stop with a Blocker below 5 GB free.

## Done means
- `cargo test -q` passes and `.venv/bin/python tools/compare.py` reports every extracted action within tolerance.
- Movement and combat items: an emulator A/B row in `docs/emucompare.md` with the scripts, what was measured on
  both sides, and a verdict; the item's `docs/movelist.md` row set to matches.
- Viewer items: `cargo build -q --bin viewer` succeeds and `cargo clippy -q --all-targets` is clean.
- **Seen working.** Every item that changes what the viewer shows or how it plays is checked in the
  viewer, unattended, with no owner needed (tested 2026-10-08: Apple M1 GPU found under
  `night-settings.json`):
  - `target/debug/viewer --shot target/shots/<name>.png [options]` for a still, and
    `target/debug/viewer --record target/rec/<name> --script <file> [options]` for a played sequence (scripted
    pad input, frames saved, exits by itself) once the record item exists.
  - Run them as plain commands: no pipes, `&&`, `;`, `cd`, `2>&1` or redirects. Only that exact form runs
    outside the sandbox; any other form fails with "Unable to find a GPU". Never run the viewer without
    `--shot` or `--record`, because it then never exits.
  - Compare the result with the same action in PCSX2 (Emulator A/B above) or, where it can't be played there,
    frames of the Shinobido clips, and with `videos/eldenring viewer.mp4`: side by side in one image, and look at it.
  - Write in the item's evidence the shot or record name, the clip timestamp, and what still differs.
    Fix the differences, or record why they stay.
  - An item whose result was not seen may not be checked. "Unverified" is never a done state.
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
9. **Acceptance review.** When no unchecked item is left before the "Acceptance review" item in
   `PROGRESS.md`, work on that item: hold the whole result up against the goal and its spirit (above).
   Record a scripted sequence for every in-scope line, compare it with the clips, and fill `docs/acceptance.md`
   with one row per line: evidence, then a verdict, either "matches" or "differs: how".
   Every gap becomes a new unchecked item inserted before the Acceptance review item; commit, and the
   loop goes on. Check the Acceptance review item only after a full pass that finds no gap.
   Exit without a commit only when that item is checked or every remaining item is blocked: the runner
   reads a session with no commit as "done or blocked" and ends the night.

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
  PS2, RenderWare or Acquire file formats, and Shinobido's controls and move lists (cite sources in `docs/`). No accounts, payments, sign-ups, uploads or unrelated sites.
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
- Codex's sandbox has no GPU and no screen, so Codex can run neither the viewer nor PCSX2. When an item needs
  either, Codex does the code and tests, leaves it unchecked, appends `(needs viewer check, for Claude)` or
  `(needs emulator check, for Claude)` to its line, and moves on to the next item it can do. Claude does those
  checks first in its next session. Claude spends its window on what only it can do: emulator A/B and viewer checks.
- Both workers keep `PROGRESS.md` current: the next worker starts from it with no other memory.
- Either worker may do the long, token-heavy work (format surveys, executable lookups), split into bounded steps
  that each leave the suite green; finishing a step is not a reason to stop (see Loop step 6).
  Codex being out of quota is never a Blocker: do the lookup yourself.
