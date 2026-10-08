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

## Milestone 8 — Data for the full scope
Goal widened 2026-10-08 (AGENTS.md): full moveset, one enemy, executions, lock-on, stealth, reference-style viewer.
- [x] (`.venv/bin/python tools/inventory.py` → `ok: 134 motions, ...`; groups in docs/player_motions.md, sets in formats.md; items below) Moveset inventory: group every player motion in `docs/player_motions.md` by action (locomotion, run stop, jumps, flips, rolls, attacks, guard, ledge, wall, stealth, items, executions, other) with its sim status; each group not yet in the sim becomes an item in Milestone 9. The player set is wider than suffix `01`: following type-3 cancel targets from the `01` motions reaches sets `02` (walk, jumps), `06` (wall hug, hang, climb), `05` (STOP_ABU/IKOU_B/SHEATH_BU, and the set holds crouch `_S`, dash `_D`, carry `_K`, hug `_H` variants) and `27` (KICK/FUTO); decide per set whether it is the player's and record why in formats.md. Check: a tool asserts every player motion is listed exactly once.
- [x] (`.venv/bin/python tools/rws.py` → `ok: ...`; track = node id + 3, see formats.md; FK bone lengths are constant by construction so confirmed via static hand tracks and limb chains instead) Skeleton order: confirm the RWS HAnim bone order against the `.nmt` tracks (currently unverified), e.g. by posing the rest skeleton with IDLE frame 0 and checking bone lengths stay constant across all player clips.
- [x] (`.venv/bin/python tools/model.py` → `ok: geometry strips give the RW triangle counts, ...`; writes `extracted/model/kagetora_body.gltf`; BODY only, no textures, skin weights ESTIMATE: rigid nearest-used-bone, real weights not found, see formats.md) Kagetora model: geometry, skin and glTF of `model\KAGETORA_BODY.rws`.
- [x] (`.venv/bin/python tools/model.py` → `ok: ... (body, heads, weapon)`; `.venv/bin/python tools/texture.py` → `ok: 32 textures decoded`, GS word sets match exactly, face/cloth/armour/weapon PNGs coherent; 4-bit in-block order is a hypothesis; attach frames recorded in formats.md, bone to attach unknown) Kagetora model, rest: heads and weapon geometry + PS2 native textures, glTF materials by texture name.
- [x] (`.venv/bin/python tools/harakiri.py` → `ok: 46 packs, 4386 entries, ... 19 KI_HI reactions`; packs are stage motion libraries, not executions; victim reactions KI_HI*07 listed in docs/executions.md, pairing/conditions ESTIMATE, formats.md "Execution packs") Executions: decode `motion\harakiri\*.nmp` and list each execution.
- [ ] Enemy pick: choose the most common foot soldier; find its model, skeleton and motion set (idle, patrol, alert, attack, guard, hit, death, execution victim). Write the list to `docs/enemy.md`.
- [ ] Enemy AI data: survey `AI\ActionSet` and `AI\battle` for the chosen enemy (sight/hearing ranges, alert states, attack choice); document in formats.md.
- [ ] Camera: survey `camera\` for the player follow and lock-on camera parameters (distance, height, FOV, lag); document in formats.md.
- [ ] Video catalogue: `docs/video_notes.md`, one line per action seen in `videos/shinobido *.mov` (timestamp, action, matching motion name, camera and HUD state); fill gaps with short emulator captures (AGENTS.md method 3). Feeds the Milestone 9 items, the camera and the HUD.
- [ ] Controls: the PS2 button→action mapping (executable pad handling, tutorial/help text in `VOLUME.DAT`, or ESTIMATE); write `docs/controls.md` with gamepad and keyboard/mouse bindings.

## Milestone 9 — Full player moveset in the sim
- [ ] Static world in the sim (engine-free): boxes and ramps as colliders, ground height, wall contact, ledge detection, falling off edges. Shared with the viewer terrain.
Items per uncovered group (motions in docs/player_motions.md; sim has run/halt/idle/walk/jump/land, AT_A/AT_C chain, guard, 4 evasions, HI_*, 7 specials):
- [ ] Run stop and run variants: STOP_K01, RUN_R/L/RE/LE01, TIRED01, STOP02.
- [ ] Jump variants and falling: FALL01, LAND_A/B01, JUMP_F/B/R/L02, TRI_J/TRI_A01, UP_FR/UP_BA01 (high jump), UKEMI_A01.
- [ ] Flips and jump attacks: FLIP_B01, KICK_J/KICK_JB/THRO_J01, CLIMB_B01.
- [ ] Remaining attacks: AT_CA/AT_D/AT_DA/AT_E01, SOCCER01, BLADE01, AT02, KICK27/FUTO27, CLAW_*.
- [ ] Ledge, wall and corner: KABE_E/KABE_U01, STOP_A/B/ONE/TWO/BACK06, HANG_L/R06, CLIMB06, IKOU_B05/06.
- [ ] Items and carrying: PICK/PUT/OPEN/CLOSE/MAKI/MAKURI*/THROW_BACK01/DRINK_H variants.
- [ ] Remaining hit reactions: DEKA_HI*, NORI_*, BURA_HI, SUI_HI, SHOJI_HI, ITEM_HI, NAGERARE_A01.
- [ ] Talk and idle extras: TOJO_B, TALK_*, KIME_*, FURI, WAIT01.

## Milestone 10 — Enemy and combat interactions
- [ ] Enemy in the sim: state machine (idle, patrol, alert, attack, guard, hit, death) on its own motions.
- [ ] Hits both ways: player hit windows damage the enemy, enemy attacks trigger player HI_* reactions and guard.
- [ ] Lock-on: target select/switch/release, strafe locomotion while locked, facing.
- [ ] Stealth: enemy awareness from sight/sound and alert state, exposed as a stealth meter value.
- [ ] Execution availability: conditions per execution and the "can execute" cue state.
- [ ] Executions: every execution from M8 plays paired (alignment, both motions, enemy dies), with a test each.

## Milestone 11 — Reference-style viewer
Copy the reference's `src/` layout (camera, input, rig, anim, hud, view); compare against `videos/`.
- [ ] `--shot <path>` flag (AGENTS.md Done means): render a fixed scene for a few frames, save the PNG, exit; options to pick the clip/frame and camera angle. Check: the PNG exists and is not blank.
- [ ] Sandbox look of `videos/eldenring viewer.mp4`: flat grid ground, grey primitive blocks, lighting, window title; the reference's `src/view` copied and adapted.
- [ ] Orbit camera (mouse/right stick) with lock-on camera, parameters from M8 camera item; default framing matches the Shinobido clips (behind and above).
- [ ] Input: gamepad with the PS2 layout and keyboard/mouse, from `docs/controls.md`, feeding the sim at 60 Hz.
- [ ] Terrain: tall walls, rooftops with ledges, stairs, slopes, a narrow walled corridor, crates, built from the sim's static world.
- [ ] Kagetora's skinned model playing the sim's current clip and frame (fallback: primitive shapes on the game skeleton).
- [ ] Enemy model and animations; execution cue effect.
- [ ] Debug HUD like the reference: player bars and hit log top-left; action, frame/length, speed, height, lock-on, queued input top-right; target name and health bar at the bottom.
- [ ] Game HUD like the Shinobido clips: health bar top-left, eye awareness icon (alert colour + stealth meter bar) top-centre, ki compass top-right pointing at enemies, execution cue.
- [ ] Review pass over the whole viewer and sim: bugs, dead code, duplication; tests green.

## Requests
Gameplay clips that would settle something; never blocking. The owner drops them in `videos/`.
- Lock-on in use (target switch, strafing) and guard against an attack: not in the current clips.
- A stealth kill from behind and one from above, showing the cue before the input.

## Blockers
(none)

## Log
- 2026-10-08: Turning done: Player.heading/target_heading, rate from 0x10bf80. Next: backward/sideways walk.
- (newest first, one line per session)
- 2026-10-08 owner: goal widened (full moveset, 1 enemy, executions, lock-on, stealth, reference-style viewer with game model); milestones 8-11 added; workers decide instead of asking.
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
- 2026-10-08 Claude: goal refined from the owner's clips: viewer look/HUD/terrain per `videos/eldenring viewer.mp4` and the Shinobido clips (eye awareness icon, ki compass), moveset adds wall run/hug, crouch, carry; player motions span sets 01/02/05/06/27; emulator for attended sessions only (night sandbox blocks screencapture/osascript, tested).
- 2026-10-08 Claude: night config review: Codex download needs no longer become Blockers (they ended nights early), M9 placeholder is no longer a checkbox, .venv python path, build timeout/no-release rules, viewer runs outside the night sandbox via `--shot` (GPU tested), display kept awake.
- 2026-10-08 Claude: M8 moveset inventory done (tools/inventory.py, 134 motions in 9 groups; sets 05/27 beyond the reached motions excluded as hypothesis); M9 group items added.
- 2026-10-08 Claude: M8 skeleton order done: .nmt track = HAnim node id + 3 (ids 0..20), via per-frame ids in rws.py; check added to `tools/rws.py`.
- 2026-10-08 Claude: M8 Kagetora BODY model done (tools/model.py: VIF strips → triangles match RW counts, skin matrices match bone lengths, glTF to extracted/model); skin weights not in file after 3 looks (0x116, V4-32 dummy, 0x1b) → rigid nearest-bone ESTIMATE; textures and HEAD/WEAPON split into a new item.
- 2026-10-08 Claude: M8 Kagetora model rest done: heads/weapon via tools/model.py (generic bone count), tools/texture.py decodes the 32 PS2 native textures (GS replay, palette swap confirmed, 4-bit in-block order hypothesis); decision: no per-texture mip/env handling, viewer uses glTF materials by name.
- 2026-10-08 Claude: M8 executions done: harakiri/*.nmp are stage motion libraries (tools/harakiri.py parses all 46), no paired kill motions exist; decision: victim = KI_HI{B,C,U}*07 (19, docs/executions.md), player = normal AT_* motion, conditions ESTIMATE.
