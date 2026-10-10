# Kagetora's moves: complete list

The web has no complete Shinobido move list (searched 2026-10-09; no GameFAQs FAQ or manual move list turned up).
What exists:
- PS2 PAL listing [psxdatacenter SCES-53931](https://psxdatacenter.com/psx2/games2/SCES-53931.html): basic controls only.
  Move left stick, camera right stick, Jump X, Run (dash) Circle, Attack Square, Grab Triangle, **Hide/crouch R1,
  Block L1**, Use item L2 or R2. This confirms the R1/L1 assignment found in the emulator (R1 crouch, L1 lock-on/block).
- `img/shinobido controls.png` (in-game Basic Controls screen), the owner's attack table in `docs/controls.md`,
  and the gameplay clips in `videos/`.
- `motion\` (VOLUME.DAT): every player motion, grouped in `docs/player_motions.md`.

This file is the working list: every move, its input, the motion it plays, and its emulator A/B status
(`docs/emucompare.md`). Status: **matches** (A/B done), **todo** (not yet tested in PCSX2).

| Move | Input (game) | Status |
|---|---|---|
| Walk / run | Left stick (partial / full tilt) | run matches; walk untestable with keyboard sticks |
| Stop | Release stick | matches |
| Reverse while running | Stick back while running | matches |
| Dash | Circle held | matches |
| Standing jump | Cross | matches |
| Running jump | Cross while running | matches (arc, landing back into the run) |
| Dash front flip | Cross while dashing (Circle held): JUMP_D05 / LAND_D05 | matches |
| Backward somersault | Cross with the stick pulled back, pressed together | matches |
| Wall run to ledge | Run at a wall, Cross on contact | matches |
| **Wall grab / double jump on a tall wall** | Run at a wall too tall to top, Cross, then Cross again on the wall | **todo**: KABE_U01 / TRI_J01 / KABE_E01 in the sim; not compared |
| Ledge hang, shimmy | Catch a ledge; stick left/right | matches |
| Ledge climb-up | Cross while hanging | matches |
| Crouch walk-off ledge catch | R1 held, walk off an edge | matches |
| Drop from ledge | R1 while hanging | todo |
| Fall / landing from height | Walk off without R1 | matches (~2 m drop) |
| Crouch / crouch walk | R1 held | partly (seen in edge grab) |
| Wall hug, corner peek | R1 into a wall | todo |
| Roll / evade | todo: input per owner's table ("R1 + Forward + Cross") needs re-check now R1 is crouch | todo |
| Attacks and chains | Square, Forward/Back + Square, air Square, dash Square (owner's table) | todo |
| Lock-on, strafe, guard | L1 | todo |
| Grab, throw, mash kill | Triangle (+ R1/L1 per owner's table, re-check) | todo |
| Pick up, carry, throw crates/bodies | Triangle near a crate/body | todo |
| Items | L2 / R2 | todo |
| Executions / stealth kills | Attack when the cue shows (behind, above) | todo |
