# Progress

Update this file in every commit. Check an item only with evidence (command + result). One item = one feature = one session.

## Milestone 0 — Environment
- [x] Versions: cargo 1.99.0, rustc 1.99.0, Python 3.14.7 (`.venv`), Claude Code 2.1.293. 33 GB free.
- [x] `git init`; `.venv/` with `rabbitizer` and `spimdisasm` (`requirements.txt`).

## Milestone 1 — Read the disc
- [x] `tools/iso.py`: lists and reads ISO 9660 files by offset. Evidence: `python tools/iso.py | tail -1` → `358262701  1725977 ACQ.BIN`.
- [x] `tools/volume.py`: lists and extracts VOLUME.DAT (FADEBABE) by path. Evidence: `python tools/volume.py | wc -l` → `6089`.
- [x] `docs/formats.md` seeded: disc layout, VOLUME.DAT, folders, `.nmt` timing hypothesis.

## Milestone 2 — Understand the player's data
- [x] Find which motions belong to Kagetora (the player): `viewer\njmotion.txt`, `viewer\motion.txt`, the motion name prefixes. Write the list (names only) to `docs/player_motions.md`.
- [x] (evidence: formats.md, 1221 unique names, parsed) `motion\mtnfile.bin`: confirm or reject "motion ID → name table". Note how IDs map to `.nmt` files.
- [x] `.nmt` header + events: header length, event list (frame, type, args). Evidence: `python tools/nmt.py` → `1350 tracks end at EOF`, all 105 player (suffix 01) files parse; footsteps of RUN01 at frames 8/23. TODO inside: meaning of event types (3/4 and 5/6 look like start/end pairs by arg id; 1, 49, 50, 53 unknown) needs an executable lookup; header flags/`a`,`b` fields unknown.
- [x] (evidence: nmt.py check, root motion RUN01 1.69 m/30 f) `.nmt` keyframes: bone tracks, frame rate, compression; which bone carries root motion.
- [x] (evidence: `python tools/rws.py` → `ok: 29 frames, 28 HAnim nodes ... matches 24 .nmt tracks`; per-bone order unverified) `model\KAGETORA_BODY.rws`: RenderWare chunks, HAnim skeleton (bone IDs, hierarchy) matching the `.nmt` tracks. `tools/rws.py`.
- [x] (layout only: `python tools/weapon.py` → `ok: 55 wpdata records, 25 wpset entries`; no damage/range fields found, see formats.md) `weapon\wpdata.bin` / `wpset.bin`: layout, which entries are the player's weapons, any damage or range fields.
- [x] (docs/missing_values.md) List values the files don't hold (movement speeds not in root motion, gravity, cancel rules, input windows). For each: targeted executable lookup or `ESTIMATE`.

## Milestone 3 — Extraction pipeline
- [x] (`cargo test -q` → 6 passed; frames, root keys, raw events, hit windows from types 1/2 and cancel windows from 3/4; AT_A01 → hit [14,21), cancels into 30/31/34 at 20–24/26; windows are hypothesis-level per formats.md) `tools/extract.py` writes `src/sim/extracted.rs` (ignored by git) with each player action's length, hit windows, cancel windows and root motion.
- [x] (evidence: removed extracted.rs, `cargo build` prints the setup.py hint, `python tools/setup.py` + `cargo test -q` → 3 passed) `tools/setup.py` runs every generator; `build.rs` explains clearly when generated files are missing (as in the reference).

## Milestone 4 — Simulation skeleton
- [x] (`cargo test -q` → 4 passed; World plays one action's root motion only, no state machine yet) Cargo project without Bevy: `src/sim/` engine-free 60 Hz `World::step`, `cargo test -q` green.
- [x] (`python tools/compare.py` → `105/105 actions within tolerance`; checks frames and root displacement only) `tools/compare.py`: runs each action through the simulation (via a test binary or `cargo test` output) and compares timings against the extracted table, printing mismatches.

## Milestone 5 — Movement
Each item is one feature with a test against extracted data.
- [x] (`src/sim/player.rs`, `cargo test -q` → 5 passed; run = 1.69 m per 30-frame cycle, HALT01 root 0 m; transitions are ESTIMATE) Run, halt, idle.
- [x] (WALK_F02 loop, 1.65 m / 80 f, `walk` flag stands in for the analog threshold: ESTIMATE; `cargo test -q` → 10 passed, compare.py 109/109) Forward walk.
- [x] (JUMP01 then LAND01, gravity from the executable, launch velocity 155 u/s and 9.8/450 m per unit are ESTIMATE, tuned so airborne time ≈ JUMP01 take-off/landing events at frames 5/56; `cargo test -q` → 11 passed) Jump and land.
- [x] (no TURN motion exists in the player set; heading turns toward a target at 2.5 pi rad/s, default branch of executable turn function 0x10bf80 (formats.md); which branch applies per action is ESTIMATE; `cargo test -q` → 12 passed, compare.py 109/109) Turning.
- [x] (`Player::walk_clip` selects WALK_B/R/L02; root -1.55 z / -1.6 x / +1.6 x per 80 f; choosing the clip from input direction is ESTIMATE; `cargo test -q` → 13 passed, compare.py 109/109) Backward and sideways walk.

## Milestone 6 — Combat
Each item is one feature, as found in the game's action data, with a test.
- [x] (AT_A01 → AT_C01 only inside the cancel window; cancel target IDs are 0-based mtnfile indices (formats.md); target choice among several is ESTIMATE (first); `cargo test -q` → 7 passed) Attack chain.
- [x] (GU_A01 → (cancel window to) GU_B01 loops while held, release plays GU_Y01 (role of GU_Y01 is a hypothesis); `cargo test -q` → 8 passed) Guard.
- [x] (FRONT01/BACK01/RIGHT01/LEFT01, 30 f, root motion +1.5/-1.6/-1.6 x/+1.6 x m; role is a hypothesis; `cargo test -q` → 9 passed) Evasion.
- [x] (`Player::hit(name)` plays any HI_* clip then returns to idle, interrupting any state; which reaction the game picks per hit is ESTIMATE/unresolved; `cargo test -q` → 14 passed, compare.py 109/109) Hit reactions (HI_*).
- [x] (`Player::special(name)` plays SHEATH01/PICK01/THROW_A01/THROW_B01/DRINK01/NINJYUTU01/MAKIBISHI01 to the end then idle, no cancels (ESTIMATE); `cargo test -q` → 15 passed, compare.py 109/109. Not done, add as items if wanted: jump kicks KICK_J/THRO_J, SOCCER01, wall/climb KABE_*/CLIMB_B01, MAKURI*, UKEMI_A01) Shinobido-specific actions discovered in Milestones 2–3.

## Milestone 6b — Executable lookups
Targeted lookups in `SCES_539.31` (AGENTS.md method step 2); record each in formats.md with addresses and wire it into the sim with a test.
Context: Event meanings found by Codex (formats.md, ".nmt event consumer"): types 1/2 attack window open/close, 3/4 transition-to-motion permission (cancel window), 5/6 category mask on/off, 7 effects (sub 1 footstep), 29-35 control flags. Still unresolved: mask category names, input buffering, damage use of type 35 (hypothesis). Codex quota resets 2026-10-08 02:53.
- [x] (rate table decoded at 0x10c0d4–0x10c240, formats.md; default 7.854 rad/s already in `TURN_RATE`; flag names unknown so other rates are not wired; `cargo test -q` green) Turn-rate constants for Turning.
- [x] (SKIPPED per owner: enemy attacks are out of scope; `Player::hit(name)` stays caller-chosen) What triggers each HI_* reaction.
- [x] (formats.md "Damage use of event type 35"; `player::hit_damage`; power stat value unknown, caller-supplied; `cargo test -q` green) Damage use of event type 35.
- [x] (CLOSED as ESTIMATE after 3 failed lookups; `Player` `walk` flag stays caller-chosen, no threshold wired) Walk/run analog threshold.
- [x] (formats.md "Transition-category bits"; `python tools/categories.py` → `ok`; names are hypotheses, masks 0x4/0x8 match no header so go via controller +0x180; not wired into sim) Mask category names (event types 5/6).
- [x] (SKIPPED per owner: controller has no buffer, not needed for the goal) Input buffering rules.

## Milestone 7 — Viewer
- [x] (`src/bin/viewer.rs`: capsule at `Player.pos`/`heading`, 60 Hz fixed step, W run, Shift walk, A/D turn, Space attack, G guard; `cargo clippy -q --all-targets` clean, `cargo test -q` → 16 passed; window not run headless, the Bevy build takes ~27 min cold) Bevy 0.19 binary (same profile settings as the reference) drawing the sim with a capsule rig; sim stays engine-free.

## Blockers
(none)

## Log
- 2026-10-08: Turning done: Player.heading/target_heading, rate from 0x10bf80. Next: backward/sideways walk.
- (newest first, one line per session)
- 2026-10-08 night (12): Claude added jump/land to Player with ESTIMATE launch velocity/unit scale; 11 tests pass. Left: turning, hit reactions, viewer (Bevy go-ahead).
- 2026-10-08 night (11): Claude found the gravity update at 0x1102b0 (a=-450 if |vy|>15 else -100-0.4|vy|, vy+=a*dt), in formats.md "Vertical velocity update"; jump launch velocity and unit scale still unknown, no sim change.
- 2026-10-08 night (10): Claude tried a float-constant scan for gravity (0.98/9.8/0.163 etc.): only hit is a 0.95-1.08 scale table at 0x609fe0, not gravity; blind scan is not viable, need a code-path lookup from the JUMP01 motion id. No code changes; exited.
- 2026-10-07 night (9): all remaining items blocked on lookups; exited.
- 2026-10-07 night (8): forward walk from the suffix-02 set. Left: jump/land/turn (need lookups), hit reactions, viewer.
- 2026-10-07 night (7): evasive steps in Player. Remaining M6: hit reactions (trigger/damage rules unknown). M5: jump/land/turn/walk need lookups.
- 2026-10-07 night (6): guard states in Player. Next: hit reactions (HI_*), evasion, suffix-02 walk.
- 2026-10-07 night (5): Player attack chain through cancel windows; motion IDs are 0-based. Next: guard, hit reactions, suffix-02 walk.
- 2026-10-07 night (4): Codex disassembled the event dispatcher (0x113220); nmt.py now reads the event count from header +0x1a (1356 files tile to EOF, was 1350). Hit/cancel windows now in extracted.rs. Next: M6 attack chain in Player using `cancels` (ids are mtnfile.bin indices: map to names), then guard/hit reactions.
- 2026-10-07 night (3): run/halt/idle state machine in player.rs. Jump/turn need gravity and turn data (lookups); stopping.
- 2026-10-07 night (2): setup.py, World::step (root motion playback), compare.py. All remaining items need event meanings (Blocker) or are M5+.
- 2026-10-07 night: decoded .nmt (header, events, position/quaternion tracks, root motion), tools/nmt.py. rws.py, weapon.py, extract.py, Cargo skeleton done (see items). setup.py done. Next: event-type dispatch in the executable (Blocker below).
- 2026-10-07: setup with owner. Disc mapped, VOLUME.DAT readable, guidelines rewritten for a data-first approach.
- 2026-10-08 Claude: backward/sideways walk done (walk_clip); M5 complete.
- 2026-10-08 Claude: hit reactions done (Player::hit, HI_* clips play out to idle).
- 2026-10-08 Claude: Shinobido-specific one-shot actions done (Player::special); M6 complete, M7 blocked on Bevy go-ahead.
- 2026-10-08: Owner approved Bevy for M7; lookups moved from Blockers into Milestone 6b.
- 2026-10-08 Claude: turn-rate selection table documented (formats.md); 6b item 1 done.
- 2026-10-08 Claude: HI_* trigger lookup blocked after 3 failed approaches; partial findings in formats.md.
- 2026-10-08 Claude: event type 35 damage formula documented and wired (hit_damage); 6b damage item done.
- 2026-10-08: walk/run analog threshold lookup failed 3 ways; blocked.
- 2026-10-08 Claude: mask category bits tabulated empirically (tools/categories.py); 6b mask item done.
- 2026-10-08 Claude: input buffering lookup blocked; controller has no buffer, player input struct untraced.
- 2026-10-08 Claude: M7 viewer binary (capsule rig) builds; sim untouched.
- 2026-10-08 Claude: owner skipped HI_* and input buffering (items closed); walk/run lookup failed again, blocked.
- 2026-10-08 Claude: walk/run lookup 3rd attempt (vtable refs, 0x2a6330 registration) failed; re-blocked, flag stays ESTIMATE.
- 2026-10-08 Claude: owner said go with recommendations; walk/run threshold closed as ESTIMATE (walk flag caller-chosen), Blocker removed. All milestones done.
