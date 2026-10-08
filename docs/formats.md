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
