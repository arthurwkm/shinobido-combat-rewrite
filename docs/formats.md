# Formats

Disc: Shinobido: Way of the Ninja, PAL, SCES-53931. Mark each statement **confirmed** (checked
against the data, say how) or **hypothesis** (not yet checked).

## Disc (ISO 9660) — confirmed, `tools/iso.py`

| File | Size | What |
|---|---|---|
| `SCES_539.31` | 5.4 MB | Executable. MIPS R5900 ELF, one segment loaded at `0x100000`. Symbols stripped (empty `.symtab`). Contains script command names (`PlayMotion2`, `PlayMotionWait`, `LoadSCMotion`, ...). |
| `VOLUME.DAT` | 547 MB | All game data, FADEBABE container (below). **The only archive this project needs.** |
| `ACQ.BIN` | 358 MB | No header, entropy 8.0 bits/byte: encrypted or compressed. Out of scope unless something points there. |
| `PAL/**.PSS` | ~2 GB | FMV movies. Out of scope. |
| `SOUND/*.TRS` | ~1.4 GB | Music and sound packs. Out of scope. |
| `IOP/MODULES/*` | small | Sony IOP drivers. Out of scope. |

## VOLUME.DAT — confirmed, `tools/volume.py`

Acquire's FADEBABE container, the same layout as Akiba's Trip (PSP) `DATA1.DAT`
(parser: Scarlet, `Scarlet.IO.ContainerFormats/DATAkiba.cs`, github.com/xdanieldzd/Scarlet).
Big-endian. Header `magic, count, count, data_offset, data_size`; then `count` entries of
`hash, offset, size, 0, path_offset, path_length`. Offsets relative to `data_offset`. 6089 files,
full original paths with `\`.

Relevant folders and types:

- `motion\*.nmt` (~1440 at top level, more in `face\`, `yabumi\`, `harakiri\`, ...): animations, named by
  action: `AT*` attacks (102), `GUARD*`, `JUMP*`, `LAND*`, `RUN*`, `WALK*`, `STOP*`, `TURN*`,
  `STEP*`, `KICK*`, `THROW*`, `NAGE*` (throws), `STAB*`, `SPIN*`, ... Little-endian contents.
- `motion\mtnfile.bin` (20500 B): u32 count `0x4C5` (1221), then u32 offsets: **hypothesis** a motion
  name/ID table, i.e. how the executable refers to motions by number.
- `motion\mtn\*.mtn`: second motion-like type, few files.
- `model\KAGETORA_BODY.rws`, `model\KAGETORA_WEAPON.rws`: the player, RenderWare stream
  (chunk format documented by the GTA modding community: clump, frame list, HAnim skeleton).
- `weapon\wpdata.bin` (6160 B), `weapon\wpset.bin` (472 B): weapon tables.
- `viewer\motion.txt`, `viewer\njmotion.txt`, `viewer\bodymdl.txt`, `viewer\headmdl.txt`:
  Shift-JIS text lists for the developers' viewer; name original paths such as `motion/rws/at01.rws`.
- `AI\ActionSet\*.ass`, `AI\battle\*.bin`: enemy behaviour. Out of scope except as a cross-check.

## .nmt — hypothesis, to confirm in Milestone 2

`motion\AT01.nmt` (14592 B) starts with little-endian floats that are exact multiples of 1/60 s:
`0.8333` (50 f), `-0.01667`, `0.0833` (5 f), `0.2667` (16 f), `0.3` (18 f), and the integer `50` (`0x32`).
Likely an action length plus timing windows (hit, cancel), like Elden Ring's TAE.
Confirm by checking that the length matches the keyframe data's own duration across many files.

## motion\mtnfile.bin and the player's motions — confirmed (index table), hypothesis (suffix)

- **confirmed**: little-endian `u32 count=1221`, then `count` u32 offsets (first = 4888 = 4+4*1221, last
  string ends at file size 20500); strings are NUL-terminated `NAME.nmt`, all 1221 unique. So motion ID =
  index into this table, value = file in `motion\`. Parsed with a short Python snippet; a `tools/` parser comes with `nmt.py`.
- **hypothesis**: the numeric suffix of a motion name selects the character/skeleton set (01..28). Suffix `01`
  (108 entries; `RUN01`, `JUMP01`, `AT_A01`, `GU_A01`, `HI_*01` hit reactions, `NINJYUTU01`, `SHEATH01`) is
  the player: it is first in the table (IDs 1..) and `viewer\motion.txt` lists `at01/jump01/land01/...` first.
  Not yet tied to the skeleton in `model\KAGETORA_BODY.rws`; check when `rws.py` exists.
- Player model files are `model\KAGETORA_*.rws` (also `GOH_*.rws`: another playable, Goh, to check). Names in `AGENTS.md`
  are right.
- Unsuffixed `AT01/AT02/AT03.nmt` exist in `motion\` but have no suffix-form entry; role unknown.

## Player motion sets — hypothesis, `tools/inventory.py` (2026-10-08)

The player set is every suffix-`01` motion (incl. `_H` variants) plus whatever is reached by following type-3 cancel
targets (arg0 = 0-based mtnfile index): 134 motions. Extra sets reached: `02` (WALK_*, JUMP_F/B/R/L, STOP02, AT02),
`06` (wall hug/corner/hang/climb: STOP_A/B, ONE, TWO, BACK, HANG_L/R, CLIMB, SHEATH06), `05` (only IKOU_B, STOP_ABU,
SHEATH_BU), `27` (KICK27, FUTO27, KICK27_H). The rest of sets `05` and `27` (full parallel lists: RUN05, HI_*05, PEER27,
SEARCH_*27 ...) are not reached from the player and look like other characters/NPC behaviours, so they are excluded
(hypothesis; revisit when the enemy is picked). Set `03`/`04` unreached. Check: every motion appears in exactly one
group of `docs/player_motions.md`.

## .nmt layout — confirmed on 1350 of 1441 top-level files by `python tools/nmt.py`

Little-endian. Replaces the earlier ".nmt hypothesis" above for everything listed here.

- **Header, 0x28 bytes**: 5 × f32 (`[1]` = length in s, a multiple of 1/60; `[0]`,`[2]`,`[3]`,`[4]` are other
  times: hypothesis, blend/ease), 2 × u32 flags (unknown), `u32 events_off` (0 or 0x28), `u32 a`, `u32 b`
  with `a + b == file size` in 1426 files; meaning unknown (the "tail" `a` points into the last tracks, so it is
  not a section start).
- **Event list** at `events_off` (when non-zero): 16-byte records `f32 time_s, u32 type, u32 arg0, u32 arg1`,
  ended by a record with `type == 0`. Times are multiples of 1/60 (confirmed on RUN01: footsteps at frames 8 and
  23 of a 30-frame loop). Types seen: 3, 5, 7 (footstep, in every run/walk/step file), 1, 13, 31, 33, 34, 35 in
  attacks. Meaning of each: hypothesis, next step.
- **Common block** after the events (at `0x28` when there are none), 12 bytes of zero first: `u16 frames, u16 bones`
  (bones is 21, tracks are bones + 3), `u16 0, u16 frames`, `u32 0x70`. `frames` equals length × 60 in 1226 files;
  in the rest the last key time equals length × 60 and `frames` is longer (tail hold, hypothesis).
- **Offset table**: u32 entries ended by a 0 word. Entry k + (table start − 0x10) is the start of track k + 1.
  Not needed for reading: tracks follow one another.
- **Tracks**: start 4 bytes after the table's 0 word; `len(entries) + 1` of them, ending exactly at EOF
  (1350 files). Track = 8-byte header (`u16 0, u16 count, u32 0`) + `count` × 16-byte keys of 8 × s16.
  - Track 0 (root motion) and track 2 (hips) are **positions**: A = (x, y, z, start frame), B = (dx, dy, dz, frames
    to next key), values /4096 → metres (hips y ≈ 0.84; hypothesis: tangents for a Hermite curve). Confirmed:
    keys tile from frame 0 with no gaps in 1340 files.
  - Other tracks are **unit quaternions** A = (x, y, z, w)/4096 with B = (start frame, frames to next key, 0, 0);
    tiling and unit length hold for tracks 3..20 in 1285 files.
  - Root motion check: `RUN01` track 0 ends 1.69 m in +z after 30 frames (3.38 m/s), `STOP01` 0 m, `AT_A01` 0.60 m,
    `FRONT01` +1.5 m, `BACK01` −1.6 m, `LEFT01` +1.6 m x, `RIGHT01` −1.6 m x (forward is +z).
- 91 files do not end at EOF (e.g. `ALARM07`, `AT_A05`, `AT_A19`, mostly other characters) and 12 have no
  tracks; the player's (suffix 01) files parsed so far are fine.

## weapon\wpdata.bin and wpset.bin — layout confirmed (`python tools/weapon.py`), meaning hypothesis

- `wpdata.bin` (6160 B) = 55 records × 112 B: 8 × u16 header (`id` 103..180, a small number 0..20, a mask
  like 11/15/27/43/107/127/139, a flag word 0/0x1000/0x3000/0x5000/0x6000/0x7000, then `0, 0xFFFF, 0, 0`) and 6 × vec4 f32
  (identity-like transforms, a (5, 5, 10, 0)-like extent). No integer that looks like damage; the floats are
  geometry, hypothesis: attachment / hit-shape transforms per weapon part.
- `wpset.bin` (472 B) = `u32 count=25`, 25 u32 offsets, then per set u16 `a, b, 0, n`, `n` indices into the 55 wpdata
  records, one pad u16 when `n` is odd. All indices < 55 for all 25 sets. Which set is Kagetora's katana, and what
  `a`/`b` mean (a = b in most sets: left/right hand?) is unknown.

## model\KAGETORA_BODY.rws — skeleton confirmed (`python tools/rws.py`)

RenderWare 3.x stream, little-endian chunks `u32 type, u32 size, u32 libid`. Top level: `0x00` (36 B: strings
`kagetora_body.rws` / `.jat`), `0x16` texture dictionary, `0x10` clump (frame list `0x0E`, geometry list `0x1A`, atomics),
a few Acquire-specific chunks (`0x1b`, `0x1000060`, unread).
- Frame list: 29 frames (56 B each: 3×3 matrix, position, parent s32, flags); frame 0 is the scene root.
  Each frame's extension holds an HAnim chunk `0x11e`; frame 1 carries the hierarchy: 28 nodes
  `(node id, node index, flags)`, ids 0..27 once each, ids 24..27 are tips (flag 1: both feet and both hands).
- Each frame's own `0x11e` extension holds its node id (frame→id: 1→23 root, 2→0 hips, 3→7, 4→4, 5→21, 6→1, ...;
  the hierarchy in frame 1 lists ids in DFS order, so use `rws.frame_ids` + frame parents, not node order).
  Tree by id: 23 root; 0 hips; legs 1→2→3 (+ tip 24) and 4→5→6 (+ tip 25); spine 7→8; head 9→10; arms 11→12→13→14→15
  (+ tip 26) and 16→17→18→19→20 (+ tip 27); ids 21→22 hang off the hips (z 2.4 each, tail/scabbard-like, no tracks).
- `.nmt` track order (**confirmed** by `python tools/rws.py`): track 0 root position, 1 root rotation, 2 hips position,
  3..23 = rotations of node ids 0..20 in id order (track = id + 3). Evidence: 21 rotation tracks = ids 0..20 (21, 22
  have none); tracks 18 and 23 (ids 15, 20 = the hands, last of each arm chain) are static (≤2 keys) in ~115 of 128
  `*01` clips while every other rotation track is static in <10; the id tree's limb chains match the anatomy.
  Quaternion component order inside a key is still unverified.
- Model units differ from motion units (hips frame offset y 9.5 vs 0.84 m in `.nmt` position tracks); scale unknown.

## .nmt event consumer — targeted SCES_539.31 lookup (2026-10-07)

Addresses below are **virtual addresses** in the disc's PAL executable. **Confirmed** means
an instruction/data relationship was checked, not measured in a running game. Gameplay names
inferred from those relationships are explicitly **hypothesis**. This section supersedes the
zero-terminated-event and two-u32-flags descriptions above; the existing parser was not changed.

### Loader, count and per-frame dispatch

- **Confirmed:** ELF PT_LOAD maps file offset `0x80` to VA `0x100000` (VA = file offset +
  `0xfff80`). `motion/` at `0x605ad0` is referenced by the motion-manager constructor
  `0x176050`; initialization at `0x3255bc` uses `motion/mtnfile.bin` at `0x60d9f0` through
  `0x12d280`. `0x1762f0` assembles the prefix and indexed filename; `0x176230` calls the
  NMT initializer `0x175e80`.
- **Confirmed:** `0x175e98–0x175ea8` relocates nonzero header `+0x1c` by adding the file
  base. `0x175eac–0x175eb4` adds header `+0x24` to the base to obtain the animation data.
  Header `+0x1a` is the event count (loaded with signed `lh`), **not flag bits**.
  The low halfword at `+0x18` remains unnamed. `+0x24` is an animation offset, not merely
  the second member of a size sum.
- **Confirmed across all 1441 top-level files:** counted events end at
  `(events_off or 0x28) + 16 * count`; the animation offset is 8 bytes later in 1429 files,
  and immediately there in 12. There are no invalid bounds. A type-zero record does **not**
  terminate runtime traversal: e.g. `SPIN_H25` has later events after type zero. The old
  parser disagrees with the count on 10 files; `THROW01` has 2 counted events but it reads 9.
  The apparent sentinel/common-block zeroes are therefore not a reliable event boundary.
- **Confirmed:** animation object `+0x310` points to the loaded motion wrapper, `+0x318`
  is the event cursor. Getter `0x177300` returns the next event when
  `event.time * object[+0x320] <= object[+0x88]`; `0x177370` uses strict `<` instead.
  Each advances the cursor by 16 and bounds it against `events + count*16`
  (`0x177330–0x177354`, `0x1773a0–0x1773c4`). These getters do not test event type zero.
  `0x1773e0` resets/seeks the cursor; `0x1775d0` stores a time multiplier and its reciprocal
  at `+0x320/+0x324`.
- **Confirmed:** per-frame loop `0x111c34–0x111c60` repeatedly calls `0x177300` and
  dispatcher `0x113220`. The latter reads event `+4` at `0x11326c`, checks type `<69`,
  then jumps through `0x604b50 + 4*type`. After most handlers, `0x114144–0x11414c` forwards
  the record to motion controller handler `0x1c1110`. Seek/replay path `0x1130c0` uses the
  strict getter and skips selected one-shot types before forwarding controller events.

### Event meanings

`actor` is the dispatcher's first argument; `controller = actor[+0x448]`. Argument names
refer to the two raw u32 fields; some handlers reinterpret them as floats or packed halves.
The table covers every type in `AT_A01` plus the other specifically requested types.

| Type | Handler VA | Meaning and evidence / confidence |
|---|---|---|
| 1 | `0x113298` | **Confirmed:** sets actor `+0x280` bit `0x80` and activates selected weapon/body objects. For arg0 `<9`, combines `arg0<<16` with actor `+0x2e4`; arg1 selects a part. Calls `0x1c74e0 → 0x184a20 → 0x158500`, which registers/resets objects through `0x130d40`; arg0 `<8` can also create/register a bone-attached object through `0x1c4510`. **Hypothesis (strong): attack/hit-window open**, with attack mode in arg0 and shape/part selector in arg1. Other arg0 modes take different branches; not a universal boolean. |
| 2 | `0x11333c` | **Confirmed:** queues the record through `0x118920` (capacity four). Flush `0x118980` calls `0x1c7550 → 0x184b20 → 0x1588b0`, disables the objects through `0x130db0`, removes the temporary object through `0x1c4700`, and clears bit `0x80`. **Hypothesis (strong): attack/hit-window close**, deferred until queue processing rather than immediate at dispatch. |
| 3 | `0x11395c` / `0x1c1154` | **Confirmed: enable transition to motion ID arg0.** `0x1c12f0` adds an entry to a 16-entry permission list. Arg1 low/high signed 16-bit halves become floats scaled by animation `+0x324`, with a third float zero. Checker `0x1c11e0` / lookup `0x1c13f0` tests the requested target ID; the values feed transition parameters at controller `+0x68/+0x6c` (`0x1c0e24–0x1c0e50`). **Hypothesis:** the exact blend/start-time interpretation of those parameters. |
| 4 | `0x113984` / `0x1c11a0` | **Confirmed: remove transition permission for motion ID arg0**, through `0x1c1360`. This pairs with type 3, not the hit-shape activation above. |
| 5 | `0x1139ac` / `0x1c11b0` | **Confirmed: enable transition-category mask arg0**, OR into controller `+0x74`. Checker `0x1c1268–0x1c1288` intersects this with target motion header `+0x14` OR controller `+0x180`, allowing the transition when nonzero. Individual category names remain **hypothesis/unresolved**. |
| 6 | `0x1139d4` / `0x1c11c0` | **Confirmed: disable transition-category bits arg0**, AND with complement at controller `+0x74`. |
| 7 | `0x113480` | **Confirmed: effect sub-dispatch**, `0x114430 → 0x1c5340`, indexed by arg0 (16 cases, table `0x606fe0`). **Hypothesis (strong): subtype arg0=1 is footstep/contact feedback**: calls `0x1c5610`, branches on contact state, chooses bone 3/6 using arg1 bit `0x10`, selects an effect/cue using arg1 low nibble and a surface-like byte, and calls `0x119030`. RUN01 uses this subtype at frames 8/23 with arg1 20/4. Type 7 as a whole is **not** synonymous with footstep: subtype 7 calls `0x1c78c0` with arg1 low/high halves. |
| 13 | `0x113dd4` | **Confirmed:** shared handler with 53; nonnegative arg0 goes to `0x119030`, subject to actor state and arg1 flags. Arg1 bit0 controls retained handle behavior, bit1 can suppress dispatch for actor kind 1. **Hypothesis: character-relative sound/cue playback**, via `0x23f2f0 → 0x2408a0`; exact cue identity unverified, no sound assets decoded. |
| 18 | `0x113618` | **Confirmed:** calls `0x116fc0`, which changes actor `+0x288` bit8 and calls `0x1c7400(actor[+0x44c],1,-1)` plus `0x1deac0`; counterpart type19 reverses the bit through `0x117040`. **Hypothesis: weapon draw/equip state**; gameplay name unresolved. It is not the type1/2 activation pair. |
| 22 | `0x11370c` | **Confirmed:** routes arg0/arg1 through actor `+0x458` helpers `0x1e2b90` / `0x1e23f0` (arg1 sign changes branch), optionally notifies actor callback with message `0x1c`. **Hypothesis: equipment/attachment action**; exact meaning unresolved. |
| 23 | `0x113764` | **Confirmed:** conditionally calls `0x1150b0(actor,0)` when actor `+0x4c0` and `+0x460` bit `0x01000000` are present; fallback calls `0x1e2f10`. **Hypothesis: release/end an attachment or paired interaction**; exact meaning unresolved. |
| 29 / 30 | `0x113890` / `0x1138a0` | **Confirmed:** set/clear actor `+0x280` bit1. Consumer `0x110330–0x110350` clamps negative actor `+0x314` to zero. **Hypothesis: suspend/resume falling**, not a demonstrated input window. |
| 31 / 32 | `0x1138b4` / `0x1138cc` | **Confirmed:** set/clear bit `0x4000`; 31 writes constant `1.5` to actor `+0x428` (ignores arg0). Consumers `0x1101c8–0x11022c` and `0x110848–0x11089c` choose movement scale 1.5 or zero from target distance. **Hypothesis: approach-target movement/stop control**, not hit activation. |
| 33 / 34 | `0x1138e0` / `0x113908` | **Confirmed: facing/turn override enable/disable.** Set/clear bit `0x2000`; 33 stores `float(arg0 bits)*2.0944` at actor `+0x42c`. Target-direction consumers `0x112f54–0x112fa4`, `0x113044–0x113094` pass this to angular turning function `0x10bf80`. Exact gameplay units remain **hypothesis**. |
| 35 | `0x11391c` | **Confirmed:** stores truncated `u16(arg0)*actor[+0x45c][+0x34]` at `+0x580`, signed upper half of arg0 at `+0x584`, and arg1 at `+0x588`. Values are consumed at `0x10e5d4–0x10e608` and `0x11404c–0x114088`. **Hypothesis: attack strength/damage parameters**, not a window toggle. |
| 49 / 50 | `0x113c74` / `0x113c84` | **Confirmed:** set/clear bit `0x400`. `0x114f70` uses it to permit assigning actor `+0x468` partner pointer (a force argument can bypass); reciprocal links use `+0x464/+0x468`. **Hypothesis: paired-character/grab receiver permission**, not established as general cancel permission. |
| 53 | `0x113dd4` | **Confirmed:** identical dispatch target and argument handling to type13. **Hypothesis: sound/cue variant distinguished elsewhere or retained authoring distinction**; dispatcher itself does not distinguish them. |

### AT_A01 interpretation, limitations and checks

- **Confirmed data / hypothesis hit interpretation:** type1 at frame14 and type2 at frame21
  bracket the attack-object active phase; nominal interval `[14,21)`. Actual end is the deferred
  queue flush, and collision sampling order was not traced. Do not claim measured hit timing.
- **Confirmed transition permissions:** types3/4 enable IDs30/31 at frame20 and remove them
  at frame24; ID34 is enabled at20 and removed at26. In `mtnfile.bin` these are `AT_C01`,
  `AT_CA01`, `SOCCER01`. Type5 enables mask8 at20, type6 clears it at25; type5 re-enables
  it at38 and enables mask`0x4000` at40. These are permission windows, **not proof that a
  raw button press is buffered for the same interval**. Other controller gates still apply.
- **Confirmed:** type50 is authored at frame64 despite the header length being55. Traversal
  checks current time, so its presence does not prove it executes during normal playback.
- **Unresolved / next:** identify mask category names, raw-input buffering, exact collision
  update order, and the remaining gameplay names above. The bounded lookup found the consumer
  and transition machinery; it did **not** fully name every event. Correct the count-based parser
  in a separately authorized step before using events from the ten affected files. No parser,
  simulation, generated tables, or `PROGRESS.md` edits were made in this docs-only step.
- **Evidence:** `.venv/bin/python extracted/check_nmt_events.py` checks exact dispatcher words,
  21 requested jump targets, all 1441 counted bounds, and all 17 distinct `AT_A01` types:

  ```text
  AT_A01: 24 counted events; 17 types covered; attack markers 14/21 frames
  OK: 21 dispatch targets match ELF; 1441 counted event bounds valid
  Animation gaps: {8: 1429, 0: 12}; old sentinel parser disagrees on 10 files
  ```

Scratch extraction/checks/listings are under ignored `extracted/`; only targeted functions were
read with rabbitizer's R5900 decoder. No whole-program decompilation or prohibited asset decoding.

Baseline checks (unchanged simulation; these do not validate the new event hypotheses):
`cargo test -q` → `test result: ok. 5 passed; 0 failed`;
`.venv/bin/python tools/compare.py` → `105/105 actions within tolerance`.
`git diff --check` passed; tracked changes are limited to this file. No commit was made.

### Motion IDs in events — confirmed by consistency (`tools/extract.py`)

Type 3 arg0 is the **0-based** index into `mtnfile.bin` (the numbering in `docs/player_motions.md`). Checked on the targets:
`AT_A01` → AT_C01, AT_CA01, SOCCER01; `AT_C01` → AT_CA01, AT_D01, AT_DA01, SOCCER01; `GU_A01` → GU_B01;
`RUN01` → STOP02 and WALK_F02/B02/R02/L02. The 1-based reading gives self-cancels and unrelated targets.
So walking lives in the suffix-`02` set (hypothesis: same character, another stance/weapon state), not in the suffix-`01` set.

### Vertical velocity update (gravity) — executable lookup, `0x1102b0–0x110350`

Reached from the actor-bit1 consumer (events 29/30). Actor `+0x314` is vertical velocity.
- **Confirmed (disassembly):** `dt` = float at `[[gp-0x6cec]+8]`. `f0 = func_2E81B8(vy)` (**hypothesis:** fabsf/magnitude).
  If `f0 > 15.0` (`0x41700000`): `a = -450.0` (`0xC3E10000`). Else: `a = -100.0 (0xC2C80000) + (-0.4 (0xBECCCCCD)) * f0`.
  Then `vy += a * dt` (`0x110318–0x11032c`). If actor `+0x280` bit0 is set, `vy` is clamped to a lower bound
  (`0x110330–0x110350`, bound register not traced).
- **Hypothesis:** units are game units (not metres; run is 3.4 m/s in root motion), so the metre value of gravity is unknown.
  Initial jump velocity (set at jump start) not found; no sim use yet.

### Turn function 0x10bf80 (found 2026-10-08)

- **Confirmed (disassembly):** `0x10bf80(actor, angle)` rotates the actor by `angle` (sign picks direction) with the step
  clamped to pi, scaled by actor `+0x420`; the step is `rate * dt` (`dt` = float at `[gp-0x6cec]+8`). `rate` is chosen by
  state flags (actor `+0x280/+0x284/+0x2e8`): 20.944 (flag 0x40 at +0x284), 10.472, 7.854 (2.5 pi, the fall-through
  default at `0x10c1f4`), 1.963, 5.24 etc. **Hypothesis:** 7.854 rad/s is the ordinary ground rate; flag-to-action mapping not decoded.
- **Confirmed (disassembly, 0x10c0d4–0x10c240):** rate selection tests actor `+0x284` bits 0x20 (rate loaded from float at `+0x568`), 0x100, 0x40 (20.944 = 6.667 pi); `+0x280` bit 0x10000; `+0x2e8` bits 0x800, 0x1000 (with `+0x28C`==1: 2.5 pi-class values 7.854 or 3.927 scaled by `+0x364`/`+0x36c`), 0x4000 (1.963 = 0.625 pi); otherwise 7.854. Names of these flags are unknown (**hypothesis:** 0x4000 is a slow/guard-like state), so the sim keeps 7.854 for all locomotion.

### HI_* reaction trigger — partial lookup (2026-10-08, no result)

- **Tried (no hit):** (1) u32/u16/u8 tables of consecutive mtnfile ids 36–56 in the executable: only ASCII digit strings; (2) clusters of `addiu/ori rX,zero,36..56` immediates: all are switches on *actor state* or message type, not motion ids; (3) string search for `HI_`/damage names: only `damage_honoo*.rws` (0x6060c0, effect model).
- **Hypothesis:** actor `+0x2e4` states 0x23–0x2D (plus 0x44/0x45) are the damage/reaction states: the same set is tested together by predicates at `0x3f0c30`, `0x1bfe40`, `0x1de6c0`. The state→motion-id mapping and the hit→state choice were not found; likely needs tracing writers of `+0x2e4` (start from `0x1bfe40`).

### Damage use of event type 35 (found 2026-10-08)

- **Confirmed:** handler `0x11391c` stores `+0x580 = trunc(float(u16 arg0) * float[actor[+0x45c]+0x34])`,
  `+0x584 = arg0 >> 16` (signed), `+0x588 = arg1`. Consumers `0x10e5d4–0x10e608` (query returning two values) and
  `0x11404c–0x114088` (calls `0x11dc90` with `+0x580 * m` as a2, `+0x584` as a3, float 30.0 in f12) both multiply
  `+0x580` by `m = 0x237950(actor[+0x45c])`: 0.5 if float `[+0x5c] < 0`, 1.0 if `== 0`, else 2.0.
- **Hypothesis (strong):** `+0x580` is hit damage (arg0 = per-attack factor, `[+0x34]` = attacker power stat, `m` = bias),
  `+0x584` a hit-type/strength class. AT_A01 has `(frame 0, 35, 7, 0)`. Power stat value and `0x11dc90` not traced: `hit_damage` in `player.rs` takes power as input.

### Walk/run analog threshold — lookup failed (2026-10-08, no result)

- Tried: (1) strings in the executable (no walk/run/stick names; only libpad); (2) immediate loads of mtnfile IDs 168–174 (RUN_R/L, WALK_F/B/R/L02): hits at `0x2a6344–0x2a6374` and `0x11523c` are a motion-registration table and a switch on actor `+0x2e4` state values, not stick logic (state values 0xA7/0xA8 coincide with RUN_RE/LE IDs: **hypothesis** state id ≈ motion id); (3) direct `sw …, 0x2e4(actor)` after an immediate: only `0x10b55c`, so states are set through a helper call. Next idea: find that helper (callers of state setter) and trace the caller using state 3/171.

### Transition-category bits (types 5/6) — empirical, `python tools/categories.py` (2026-10-08)

The category word tested by the checker at `0x1c1268` is the motion header word at `+0x14` (= `flags[0]` in
`tools/nmt.py`, **confirmed** by layout: 5 × f32 precede it). Across the player's motions (**confirmed** by the script's asserts):
- `0x20000` is on every attack (AT_A01/C01/D01/E01, SOCCER01, KICK_J01); `0x80` on every HI_* hit reaction (except HI_F01 `0xa0`), with `0x40000`/`0x80000` stacking on heavier reactions;
  `0x4000` on JUMP01/FLIP_B01/HALT01/RUN01 family; `0x8000` and `0x40000000` on locomotion (STOP01, RUN01, WALK_*); `0x200` on KABE_E01 and RUN_RE/LE; `0x100` UP_BA01; `0x400` BURA_HI01.
- Mask args used by type 5: `0x4000` (after AT_A01 at f40, AT_E01, SOCCER01, LAND01…: allows jump-like motions), `0x200`/`0x400` (JUMP01, FALL01, KABE/TRI_J01), `0x20` (CLIMB_B01, MAKURI_H01), `0x100`, `0x40000000` (SHEATH01).
- Mask `0x8` (all attack chains) and `0x4` (DRINK01) match **no** player header word, so they must hit the controller word `+0x180` instead (**hypothesis**: attack-chain/“combo” permission, set elsewhere).
- Names stay **hypothesis**: `0x20000` attack, `0x80` hit reaction, `0x4000` air/jump, `0x8000`/`0x40000000` ground locomotion.

## Input buffering — partial executable lookup (2026-10-08)

- **Confirmed (code read):** the motion controller keeps **no input buffer**. Request function
  `0x1c0d90` (and `0x1c0e70`) calls the permission checker `0x1c11e0`; on success it stores the
  target in controller `+0x64` with start/blend floats at `+0x68/+0x6c`, clears flags `0x3000` at
  `+0x70`. `0x1c0c70` applies it in the same update: starts the motion, `+0x60 = +0x64`,
  `+0x64 = -1`, zeroes `+0x74` (category mask), `+0x180`, `+0x178` (permission count). A refused
  request returns 0 and leaves nothing pending. Checker order (`0x1c1228–0x1c1288`): pending id
  `+0x64` equal to target → -1; `+0x70` bit `0x2000` or `0x1000` → 1; else category test
  (`+0x74 & (target header +0x14 | +0x180)`), else permission-list lookup `0x1c13f0`.
- **Confirmed:** actor-level wrapper `0x10cb30(actor, slot)` maps an action slot through the
  `actor+0x2f0` halfword table to a motion ID and calls `0x1c11e0` (a pure "may I" test); ~40
  callers (`0x15cdc4`, `0x15e02c`, `0x29a2a8`, `0x381e20`, …). Generic start helper `0x10c900(actor, slot, start)`.
- **Unresolved:** any press-hold/timer buffering lives in the player input struct (`actor+0x450`,
  fields seen `+0x5e0`, `+0x614`, `+0x690`, `+0x734/+0x738`), not traced.

### Walk/run analog threshold — second attempt (2026-10-08, no result)

- **Confirmed (disassembly):** `0x10c900(actor, state, start)` is the state requester: maps `state` through the `actor+0x2f0` halfword table to a motion id, asks the controller (`0x1c0d90`), and on success stores `state` in actor `+0x2ec` (pending). `0x110f90` moves `+0x2ec` into `+0x2e4` (current) and loads `+0x2e8` flags from table `[[gp-0x6d38]+0x830]`. `0x10c9c0` is the variant taking an explicit motion id; constructor `0x10b500` starts in state 1.
- **Tried (no hit):** (1) the ~200 callers of `0x10c900`: the cluster at `0x15cc50–0x15dc00` is a per-state request switch with no direct callers or data references (probably reached through an untraced table); (2) `0x15caf0` callers are a sort, not player code; (3) functions that read `+0x450`, call a state setter and compare floats: none.

### Walk/run analog threshold — third attempt (2026-10-08, no result)

- **Confirmed:** vtable-style refs to the `0x15cc50` cluster exist only as 4-word tables at `0x617734` and `0x61ea14` (`0x15cc50, 0x15d180, 0x15d9c0, 0x15da40`); `0x15d180` is a generic actor update with stick-like compares at +-0.8 (`0xbf4ccccd`/`0x3f4ccccd`, `0x15d31c`) calling `0x1893c0`/`0x1bf880` with +-1/-2: looks like menu/camera selection, not the player's walk/run.
- **Observed:** registration code at `0x2a6330–0x2a63e0` binds motion ids 0xB2, 0xA8, 0xAB, 0xAE, 0x15A, 0xB0 to object slots `+0x88..0x9c` and stores float pairs 0.8/1.35 (`0x3f4ccccd`/`0x3faccccd`) at `+0x30/+0x34` of two of them (**hypothesis**: speed-scale or blend range of a locomotion blender, consumer not found).
- Next idea if retried: find readers of `[obj+0x30]`/`[obj+0x34]` for the slots at `s3+0x8c/0x98`.
