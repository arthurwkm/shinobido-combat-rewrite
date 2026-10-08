# Progress

Update this file before stopping every session. Check an item only with evidence (command + result).

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
- [ ] `model\KAGETORA_BODY.rws`: RenderWare chunks, HAnim skeleton (bone IDs, hierarchy) matching the `.nmt` tracks. `tools/rws.py`.
- [ ] `weapon\wpdata.bin` / `wpset.bin`: layout, which entries are the player's weapons, any damage or range fields.
- [ ] List values the files don't hold (movement speeds not in root motion, gravity, cancel rules, input windows). For each: targeted executable lookup or `ESTIMATE`.

## Milestone 3 — Extraction pipeline
- [ ] `tools/extract.py` writes `src/sim/extracted.rs` (ignored by git) with each player action's length, hit windows, cancel windows and root motion.
- [ ] `tools/setup.py` runs every generator; `build.rs` explains clearly when generated files are missing (as in the reference).

## Milestone 4 — Simulation skeleton
- [ ] Cargo project without Bevy: `src/sim/` engine-free 60 Hz `World::step`, `cargo test -q` green.
- [ ] `tools/compare.py`: runs each action through the simulation (via a test binary or `cargo test` output) and compares timings against the extracted table, printing mismatches.

## Milestone 5 — Movement
- [ ] Walk, run, stop; jumps and landings; turning — each with a test against extracted data.

## Milestone 6 — Combat
- [ ] Attack chains, guard, evasion, hit reactions — as found in the game's action data, each with a test.
- [ ] Shinobido-specific actions discovered in Milestones 2–3 (list them here as they are found).

## Milestone 7 — Viewer
- [ ] Bevy 0.19 binary (same profile settings as the reference) drawing the sim with a capsule rig; sim stays engine-free.

## Blockers
- (none)

## Log
- (newest first, one line per session)
- 2026-10-07 night: decoded .nmt (header, events, position/quaternion tracks, root motion), tools/nmt.py. Next: rws.py, wpdata.
- 2026-10-07: setup with owner. Disc mapped, VOLUME.DAT readable, guidelines rewritten for a data-first approach.
