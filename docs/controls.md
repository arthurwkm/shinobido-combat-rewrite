# Controls

Source: the disc's `US\UI\txd\help\N_CtrlGuide.rws`, the English “Basic Controls”
image. Decoded and visually read at `target/controls-guide-wide.png`; decoding notes
are in `docs/formats.md`, “Control guide”. Cross = jump is independently visible in
`videos/shinobido movement.mov` at 00:00. This is the layout for the future M11 input
item, not the bindings currently implemented by the capsule viewer.

## Basic layout

Face buttons and sticks below are confirmed by the guide's labels and leaders.
Shoulder labels are confirmed; physical L/R interpretation uses the front view of
`img/shinobido controls.png`, confirmed in PCSX2 by the owner on 2026-10-09:
R1 is Hide/Crouch (hold to crouch-walk), L1 is Lock-on. The earlier rear-view reading was wrong. Keyboard/mouse bindings are project choices,
not extracted game data. Gamepad names use physical positions, avoiding the Xbox
“X” versus PlayStation Cross ambiguity.

| Action | PS2 control | Gamepad position | Keyboard/mouse | Behaviour |
|---|---|---|---|---|
| Move | Left stick | Left stick | W/A/S/D | Hold, relative to camera; strafe while locked |
| Camera | Right stick | Right stick | Mouse motion | Orbit |
| Attack | Square | West face | Left mouse | Press; repeated presses request chains |
| Grab | Triangle | North face | E | Context action (also lifts a crate or a dead soldier; Triangle again puts it down, Square throws it) |
| Dash | Circle | East face | Left Shift | Hold |
| Jump | Cross | South face | Space | Press |
| Lock-on | L1 | Left bumper | Right mouse | Hold |
| Hide/Crouch | R1 | Right bumper | Left Ctrl | Hold |
| Use item | L2 or R2 (the guide's leader lines point at both) | Either trigger | Q | Press |
| Equipment screen | Select | Select/Back | Tab | Documented only; menus out of scope |
| Pause menu | Start | Start | Escape | Documented only; menus out of scope |

The guide does not label the D-pad, or stick clicks. Do not infer original-game
functions for those controls. The hold/press choices, movement relative to the
camera, walk/run stick threshold and dash behaviour are **ESTIMATE**: the static
image names actions but does not specify these rules. For keyboard walking, use
Alt + W/A/S/D; normal movement runs and Shift requests dash. Numeric thresholds
must be marked `ESTIMATE` in `src/sim/data.rs` when M11 implements them.

## Attack and move list (owner, 2026-10-08)

Supplied by the owner from play; every row is shown in `videos/shinobido all attacks.mp4` (30 fps) and can be
checked in PCSX2 with `tools/pcsx2.py`. The disc is SCES-53931 (PAL), so the "PAL only" rows apply. Which motion
each row plays is still to be matched (motion names are not evidence of inputs). The owner's R1 inputs below, including R1 + Square, need re-checking against the emulator now that L1 is confirmed as lock-on. Rows retain the owner's wording; existing lock-modified combos use L1 in the viewer.

| Input | Move |
|---|---|
| Square | Slash |
| Square, Forward + Square | Slash to Power Slash |
| Square ×2 | Double Slash |
| Square ×3, wait 1 s, R1 + Triangle | Double Slash to Kick to Power Slash |
| Square ×2, Forward + Square, R1 + Triangle | Double Slash to Jumping Strike to Power Slash |
| Square ×2, Forward + Square ×2 | Double Slash, Jumping Strike to Power Slash (PAL only) |
| Square ×2, Forward + Square, Square ×2 | Double Slash, Jumping Strike, Slash to Kick (PAL only) |
| Back + Square | Kick |
| Square in the air | Downward Strike |
| Square while dashing (Circle held) | Leap forward and slash |
| Square near a downed enemy | Low Kick |
| R1 + Back + Square | Low Kick |
| R1 + Square | Rising Uppercut |
| R1 + Square ×2 | Rising Uppercut to Downward Strike (PAL only) |
| R1 + Triangle | Throws the enemy overhead, leaving him downed |
| R1 + Forward + Cross | Forward roll into the enemy, flips over him: attack or grab from behind |
| Triangle | Grab the enemy; mash Square / Triangle for a chance at a forced stealth kill (easiest from behind or on a low-HP enemy) |
| Cross with the stick pulled back (pressed together; a flick after Cross is a plain jump) | Backward somersault |
| Cross while dashing (Circle held) | Front flip (JUMP_D05 into LAND_D05; emulator 2026-10-10) |

## Context actions for the sandbox — ESTIMATE

The basic guide does not explain these combinations. Use this provisional mapping
until a targeted lookup or visible tutorial supplies better evidence; do not treat
animation names as evidence of button assignments.

| Context | Proposed input | Intended request / reason for estimate |
|---|---|---|
| Guard | Lock-on held, no attack/jump/dash | Defensive stance; guard combination not shown |
| Evasion / roll | Hide/Crouch + Jump + movement | Direction selects evade; combination not shown |
| High jump / wall ascent | Jump by a reachable wall | Geometry selects action; activation not shown |
| Ledge | Move into ledge; Jump to climb, Hide/Crouch to drop | Context navigation; inputs not shown |
| Wall hug / corner peek | Hide/Crouch toward wall; lateral movement at corner | Shares stealth modifier; inputs not shown |
| Pick up / put down / open / close | Grab by object; Grab while carrying to put down | Guide only names Grab |
| Throw carried object | Attack while carrying | Avoids conflicting with put down; input not shown |
| Jump attack / kick | Attack airborne | Context selects motion; input not shown |
| Execution | Attack when execution cue is available | Shares attack; execution input not shown |
| Item actions | Use item | Selected item chooses drink/throw/etc.; selection not shown |

Additional viewer choices: the camera is the reference project's (owner, 2026-10-09): lock-on (L1 / right mouse)
pressed with no target in range swings it behind Kagetora; left alone while moving it drifts round to follow
sideways movement. D-pad right / Tab
cycles sandbox items. Target switch does not exist in Shinobido; the viewer copies Elden
Ring's (owner, 2026-10-09): while locked, flick the right stick, the mouse or the left/right
arrow keys sideways to move the lock to the next target that side.
These are **ESTIMATE**, not claims about the original pad. They keep target and item
selection reachable without implementing an equipment menu. Automatic reactions,
falls, landings and idle variants require no separate button. Other unbound clips
remain available to the planned clip viewer, not through invented gameplay keys.

M11 must sample render-frame edges once and consume them once at the next 60 Hz
simulation tick; held inputs persist across ticks. This avoids the current viewer's
known missed/duplicated `just_pressed` taps, without adding gameplay input buffering.
The existing viewer still uses W to run, Shift to walk, A/D to turn, Space to attack
and G to guard; this documentation item does not change runtime controls.

Chain pairing (ESTIMATE, from the cancel graph, not frame-matched): inside AT_C01 (third slash) Forward + Square picks AT_DA01
(jumping strike, then AT_CA01 power slash), Lock-on (L1) + Square picks FUTO27 (kick, falls back to KICK27); plain Square picks AT_D01.
