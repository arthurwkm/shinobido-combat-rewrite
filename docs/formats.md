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
- 28 nodes − 4 tips = 24 animated nodes = the 24 tracks of a `.nmt` (**hypothesis** for the mapping: track 0 root
  position, 1 root rotation, 2 hips position, 3..23 the 21 `bones` rotations in hierarchy order; counts match, the
  per-bone order is not verified yet).
- Model units differ from motion units (hips frame offset y 9.5 vs 0.84 m in `.nmt` position tracks); scale unknown.
