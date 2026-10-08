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

## model\KAGETORA_BODY.rws geometry — confirmed by `python tools/model.py` (2026-10-08)

Clump geometry list holds 2 geometries (`0x0f`): 412 verts/337 tris and 1095 verts/1004 tris (RW struct counts).
Each has extensions `0x50e` (mesh table), `0x510` (PS2 native data), `0x116` (skin).
- `0x50e` payload: `flags=1 (tristrip), numMeshes, totalIndices`, then `numMeshes × (numIndices, materialIndex)`;
  the sum of numIndices equals totalIndices.
- `0x510` payload: u32 platform 4, then per mesh a `(size, 0)` pair and VIF packets. Packet: DMA-like word pair
  (`count | 0x30000000`, running vertex offset), then STMOD/STCYCL(CL 5, WL 1) + five UNPACKs at VU addresses 4 (V4-32, dummy: no
  data bytes follow), 0 `0x68` V3-32 positions, 1 `0x64` V2-32 UV, 2 `0x6e` V4-8 (always 0,0,0,255 here), 3 `0x6a` V3-8
  normals (signed bytes, length ≈127). Packets hold ≤42 vertices; consecutive packets of one mesh overlap by 2 vertices
  (strip continuation), so numIndices = Σcount − 2·(packets−1) (confirmed on all 20 meshes). Each mesh is one triangle strip;
  dropping triangles with two equal positions leaves exactly the struct's triangle count (337 and 1004).
- `0x116` skin payload: platform 4, `numBones 28, numUsedBones, maxWeights 2, pad`, used-bone list, 28 inverse-bind
  matrices (64 B, row-vector, translation in row 3), 28 trailing bytes. Matrix k belongs to node `nodes[k]` of the HAnim
  list (DFS order, not id order): bone lengths from the matrices equal the frame offsets for all 27 bones (confirmed).
  Used-bone entries are matrix indices (hypothesis: fits the body geometry better than node ids).
- **Not found: per-vertex skin weights/indices.** The `V4-32` unpack at address 4 is the likely carrier (the DMA word's
  second half grows by 40 per packet, a running vertex offset into a table not present in the file); the chunks `0x1b`
  (1276 B) and `0x1000060` were not decoded. `tools/model.py` therefore skins rigidly to the nearest used bone segment (ESTIMATE).

### KAGETORA_HEAD_H / HEAD_L / WEAPON geometry — confirmed by `python tools/model.py` (2026-10-08)

Same chunk layout as BODY (one clump with `0x50e`/`0x510`/`0x116` per geometry), so `tools/model.py` reads all four files;
the strip/triangle-count check passes on each. HEAD_H: 3 geometries (42, 42, 584 tris; 7 meshes in the big one), 26-node
HAnim skeleton (face/hair bones); HEAD_L: 3 geometries (42, 42, 568 tris), 3 nodes; WEAPON: 1 geometry (166 tris), 2 nodes.
Each part has its own root frame; head files place frame 0 at y 15.3 / z 0.22 (model units, hypothesis: attach point on the
body's head bone, bone 9 = head per id tree), the weapon at the origin (attach to a hand bone: unknown, hypothesis hand 15/20).
Skin weights are missing here too (same rigid ESTIMATE). `tools/model.py` also resolves each material to its texture
name (material list `0x08` → `0x07` → texture `0x06` → name `0x02`) and writes glTF materials pointing at the PNGs.

### PS2 native textures (`0x16` dictionary, native `0x15`) — `python tools/texture.py` (2026-10-08)

32 textures in BODY (16 + 1 duplicate), HEAD_H (7), HEAD_L (6), WEAPON (2; the second, `kagetpra_env`, is an environment map).
- Chunk `0x15`: `0x01` (8 B: `'PS2\0'`, filter/addressing) · `0x02` name (16 B) · `0x02` mask name (4 B) · `0x01` data · `0x03` ext.
  Data struct = chunk header + 64-byte raster header (u32 `width, height, depth (4|8), rasterFormat (0x2x504 = 8888 palette),
  tex0 lo/hi, tex1 lo/hi, miptbp1 lo/hi, miptbp2 lo/hi, texelBytes, paletteBytes, gpuAligned, skyMip`) then a second chunk header
  and the two GS upload packets back to back: texels (`texelBytes`) then palette (`paletteBytes`). Each packet = GIF tag
  (NLOOP 3, A+D) · regs TRXPOS 0x51, TRXREG 0x52, TRXDIR 0x53 · IMAGE tag (NLOOP = data/16) · data; header 80 B.
  **Confirmed**: `texelBytes` = w·h·depth/8 + 80 (4-bit rows padded to 32 px) and tex0 PSM = 0x13 (PSMT8) / 0x14 (PSMT4).
  tex0 TBP/TBW and TRXREG sizes do not describe the data (TBW is 2 everywhere; TRXREG is w/2×h/2 even for 4-bit): ignored.
- Texels are the raster of a PSMCT32 rect uploaded into GS memory and then read as PSMT8/PSMT4 (the usual swizzle trick).
  Decoding replays GS addressing (block tables, 8x8-word / 16x16-byte / 32x16-nibble column tables, page = 64x32 / 128x64 / 128x128).
  **Confirmed** for all 32 textures: the set of GS words written by a rect of some width (8..128) equals exactly the set of words
  the texture reads. The 8-bit pictures (face, cloth, armour mesh, weapon) come out as coherent images, which confirms the
  8-bit tables and block order. **4-bit hypothesis**: same structure, column table by analogy, coverage exact, but dithered
  skin textures (`Goh_hand`) look noisy and the in-block order is not independently confirmed.
- Palette: 8-bit = 256 words with entries 8..15 and 16..23 of every 32 swapped (CSM1; **confirmed**: face/cloth smoothness
  8.5/4.3 vs 31/26 unswapped); 4-bit = first 16 words in file order (hypothesis). RGBA bytes, alpha 0..0x80 (×2 for PNG).
- Output: `extracted/textures/<part>/<name>.png`; `kagetora_tazuna0` appears twice in BODY (same name, different image: the
  second overwrites the first in the output dir; materials resolve by name so the viewer needs per-material order if it matters).

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

## Execution packs and executions — `tools/harakiri.py` (2026-10-08)

- **Confirmed** (`harakiri.py` → `ok: 46 packs, 4386 entries, all IDs named`): `motion\harakiri\C001..C046.nmp` is
  little-endian `u32 count`, then `count × (u32 motion ID, u32 offset, u32 size)`; every ID is a valid 0-based
  `mtnfile.bin` index and every blob ends inside the file. Offsets are not monotonic and a few entries (IDs 134, 148,
  231, ...) have offsets inside the table and sizes ≈ file size: nested/index entries, not decoded.
- **Confirmed by content**: a pack holds ~50–220 motions of several actor sets (`01` Kagetora, `05`, `06`, `07`
  foot soldier, `12`, `13`, ...) i.e. the per-stage motion library; **it is not a set of paired executions** (the
  name "harakiri" is the game's internal title). `motion\face\harakiri\*.fmp` is the face counterpart.
- **No paired execution motions found.** Names never appear in `VOLUME.DAT` data (scripts refer to IDs);
  `KIME_A01/B01/A05` (IDs 1131/1132/1138) are not referenced by any immediate in the executable (so script-driven,
  out of scope). `AT_SA05` is only reached from `JUMP_*05` (a jump attack).
- **Hypothesis**: kill reactions of the foot soldier (set `07`) are `KI_HI<dir><variant>07` (19 motions, IDs
  257–294, listed in `docs/executions.md`); `dir` B/U/C follows the `HI_B*/HI_U*/HI_C*` hit naming (back / above /
  front, matching the owner's stealth-kill-from-behind/above request). Type 3 events inside `KI_HI*C07` chain the
  `S/A/B/C/D` variants (e.g. KI_HIBC07 → 261, 263). The player side is the normal attack motion (`AT_*`) with the
  victim switched to the matching `KI_HI*`; this is an ESTIMATE until a stealth-kill clip is supplied (PROGRESS Requests).

## Enemy AI: `AI\battle\*.bin`, `AI\ActionSet\*.ass` — `tools/ai.py` (2026-10-08)
All little-endian i32.
- **`AI\battle\Busi*.bin` — layout confirmed** (`python tools/ai.py` → `ok: 10 Busi battle files, 1120 rows`, tables fill each file exactly).
  `u32 ntables` (7), then per table `u32 rows` (16 in every table seen), per row `u32 n` and `n` records of 5 words
  `(action id, weight, class 0/1/2, -1, -1)`. `n` is constant within a table (BusiKatana: 10, 9, 9, 8, 5, 6, 6).
  Meaning is **hypothesis**: an attack/behaviour choice table; `action id` indexes the ActionSet action list, `weight` is a
  percentage-like chance (values 0-180, rows do not sum to 100, so likely a per-id weight/range, not a probability),
  `class` 0/1/2 a situation group, the 7 tables are situations (e.g. by distance) and the 16 rows are enemy "levels" or
  pattern slots (the `BusiKatana01..05` files differ from `BusiKatana` only in weights). No sight or hearing numbers are here.
- **`AI\ActionSet\*.ass` — layout hypothesis**: header `(6, N)` (N = 20/21/22 by variant: `Bushi_Fr`/`_Al` 20, `_St` 22, rest 21), then
  a transition table of small ints: action ids, `-1` for none, `10000` (probably 100.00 %) and weights 30/50/70/300. The
  `Bushi_*` variants differ in 4 (`Wt` vs `Pt`) to 263 (`St`) words, so the family is one table with per-role overrides
  (Pt patrol, Wt wait, St stand, Al alert, Ba back, Bl block, Km, Fr). Exact record boundaries not decoded.
- `ActionSet.asc` starts `(144, 1, 7)`, 4226 words, 80 % zero: a registry of ~65 actions; `Pack\AsPackName.fnt` is 65 × 256-byte names
  (`AI/ActionSet/General_Pt.ass`, ...) and `Pack\AsPack.pac` 65 offsets + the same `.ass` data.
- Sight range, hearing range and alert thresholds are **not in these files**; they are in the executable (or `chara\`).
  Not looked up: the sim uses `ESTIMATE` values for them (Milestone 10 stealth).

## Camera data (`camera\`)
Survey by `tools/camera.py` (`ok: 110 .cmt parsed ...`).
- **Confirmed** (110 files parse, tracks fill each file exactly, key times chain `t + dur = next t`, last key t = length): `camera\*.cmt` are scripted keyframe cameras, not a follow camera. Header: `u32 0`, `u32 0x10000|frames`, `u32 0`, `f32 frames`, then three `u32` track offsets (first is always `0x20`). Each track: `u32 0, u32 n, u32 0, f32 frames`, then `n` keys. Position keys are 8 words `[x y z, t, 0 0 0, dur]` (tangents are 0 or tiny); rotation keys 8 words `[qx qy qz qw, t, dur, 0, 0]` (unit quaternion within 5%); FOV keys 4 words `[fov, t, 0, dur]`, fov 0.28..1.31 rad (hypothesis: radians; time unit assumed to be frames at 30 Hz, not checked). Last key has `dur` 1.
- **Confirmed:** `cmtnfile.bin` = `u32 n` (97), `n` `u32` offsets into a string block that follows, holding NUL-separated upper-case file names; all exist as `.cmt` (case-insensitive). The names are action cameras (CLIMB, BURA, NORIR/NORIL, SUI, SHOJI, ITEM, KUGURI, HEBI, KABUTO, KENOBI, MOSU, KUMA, DEKA, MAE, MAKURI, NUKERARE/SEIKO/TEIKO/STR variants, CRI*/KUBIKIRI for the critical/stealth kill) plus `CAME_*` scene shots. Hypothesis: the game plays one of these when the player starts the matching action (e.g. `climb.cmt` 164 frames for CLIMB), relative to the player.
- **Hypothesis:** `cmractdb.bin` = header `u32 103, 0, 0, 0` then 103 records of 816 bytes starting with an id (38..58, 82..125, 256..1191, ascending; looks like a scene/event id), then floats `pos(3), 40.0, 1.392 1.09 0.737, ...` (a fixed camera with FOV 40 and per-record action sub-blocks of `[x y z 1.0]` points). Not needed for the sandbox.
- **Not in the files:** the third-person follow camera (distance, height, FOV, lag) and the lock-on camera. The executable has camera code (strings `Camera`, `DontLock CharacterCamera` at VA `0x611db0`, referenced from `0x3fb194`; `cmtnfile.bin` loaded near `0x31bf68`..; `cmractdb.bin` near `0x3a8ea8`, `Special stealth kill camera fault!!` near `0x3a4228`) but it is a large vector-math camera class; per AGENTS.md rule 7 follow/lock-on values are `ESTIMATE` in `src/sim/data.rs` / the viewer, tuned against the Shinobido clips (video catalogue item).

## Control guide — disc help texture (2026-10-08)

- **Confirmed (VOLUME directory and visual read):** `US\UI\txd\help\N_CtrlGuide.rws`
  is a 263568-byte RenderWare texture dictionary with the English Basic Controls
  diagram; no script VM decoding is needed. Square Attack, Triangle Grab, Circle
  Dash, Cross Jump, left stick Move, right stick Camera, Select Equipment Screen,
  Start Pause Menu. Cross Jump also appears in the movement clip's 00:00 tutorial.
- **Confirmed (image leaders):** lower drawing image-left button 1 is Lock-on;
  image-right 1 is Hide/Crouch; image-right 2 is Use Item. **Hypothesis:** this is
  a rear/underside view (grips upward, sticks underneath), so these mean R1, L1,
  L2 respectively; the drawing's numerals have no legible L/R letters. The other
  button 2 is unlabelled. `docs/controls.md` records these qualifications and
  explicitly estimated context combinations; the image does not prove timing,
  hold/toggle semantics, guard or execution inputs.
- **Confirmed for this image:** native header at file offset `0x7c` is 512×512,
  8-bit; pixel packet at `0xc8` reports a 256×256 PSMCT32 upload. Existing model
  decoder `tools/texture.py` assumes one horizontal GS page and repeats columns
  on this wider image. A temporary decoder reusing its block/column and CLUT
  tables produces a coherent full diagram. No change to model decoding here.
  Reproduction: read only this entry with `volume.read_entry`; read `<16I` at
  `0x7c`; pass packet sizes header[12]/[13] to `texture.packet` starting at `0xc8`.
  For upload words, page index is `(y // 32) * (upload_width // 64) + x // 64`,
  times 2048 words, plus the existing within-page BLOCK32/COL32 offset. For PSMT8
  sampling, page index is `(y // 64) * (width // 128) + x // 128`, times 8192
  bytes, plus the existing BLOCK32/COL8 offset. Use the existing 256-entry CLUT
  permutation and alpha conversion. Both address sets cover the 262144-byte
  raster; unlike the narrow decoder, horizontal pages are distinct.
- Evidence: `.venv/bin/python target/decode_control_guide.py` →
  `target/controls-guide-wide.png: 512x512; upload 256x256 PSMCT32, 4 pages across`;
  PNG visually inspected. Temporary decoder and image stay in ignored `target/`.
  This is a targeted image decode, not a generalized wide-texture parser.

### Run/stop variants — motion consistency survey (2026-10-08)

- **Confirmed from parsed tracks:** RUN_R01/L01 last keys are frame 27 with +1.899902 m Z displacement
  and zero X/Y; 21 of 22 rotation tracks have equal first/last poses. TIRED01 (70 frames) and STOP02
  (80 frames) have zero root displacement and all 22 rotation endpoints match. These support looping
  playback (**hypothesis**); header floats 2/3 span 0..duration for each, but their loop semantics are unconfirmed.
- **Confirmed:** RUN_RE01/LE01 are 24 frames with only −0.006104 m Y root displacement;
  STOP_K01 is 36 frames with zero root displacement. Their endpoint poses differ and header floats
  2/3 are −1/−1. One-shot playback is a **hypothesis**.
- **Confirmed:** STOP_K01 enables TWO06/ONE06/BACK06 at frame 1. STOP_K01 and the R/L/RE/LE
  variants share category bit 0x200 with KABE_E01. **Hypothesis:** these are wall-related variants,
  not ordinary steering animations; RUN_R/L have no lateral root translation. HALT01 remains the
  ordinary release clip. Its track duration is 32 frames versus 38 in the header, a known duration mismatch.
- **ESTIMATE (src/sim/data.rs):** callers select supported run/idle loops, as with walk_clip; changes
  start the selected clip at frame zero, with no guessed fatigue or wall trigger. RUN_RE/LE and STOP_K
  use the existing one-shot Special path, returning to the selected idle. Wall-driven selection remains
  in the wall/ledge item. STOP02 is extracted alongside WALK_*02; RUN01 permits it at frame 0 and HALT01
  at frame 3. No cancel requirement is imposed on idle selection (ESTIMATE).
- Checks: parsed the seven motions with `tools.nmt.parse`; `cargo test -q` covers loop displacement,
  one-shot duration, selection during playback, interruption/re-entry and invalid selection fallback;
  `tools/compare.py` checks all 110 extracted clips against their source root displacement and duration.

## Pose export (`tools/pose.py`, 2026-10-08) — viewer rig data
Writes `extracted/anim/skeleton.bin` (28 nodes: parent id, rest offset x `SCALE` 0.84/9.5 to metres, ESTIMATE) and `clips.bin`
(183 player/enemy clips sampled per frame: hips position, 21 local quaternions for node ids 0..20). Quaternions are used
as stored (x, y, z, w), local to the parent, offset applied in the parent's frame; position tracks are interpolated linearly
and quaternions by normalised lerp between keys (Hermite tangents ignored). Hypothesis, checked by FK on STOP01 frame 0
(`sim::pose::Skeleton::joints`): spine rises, head ~1.26 m, ankles ~0.24 m above hips-0.84 ground, arms hang; the conjugate
order puts the limbs through the body, so the stored order stands. Hips position is relative to the root (no double
counting with root motion). Clips with another track count (DRINK_S05 66, SHEATH_BU05 68, LOOK07 22) are not exported.
