# Controls

Source: the disc's `US\UI\txd\help\N_CtrlGuide.rws`, the English “Basic Controls”
image. Decoded and visually read at `target/controls-guide-wide.png`; decoding notes
are in `docs/formats.md`, “Control guide”. Cross = jump is independently visible in
`videos/shinobido movement.mov` at 00:00. This is the layout for the future M11 input
item, not the bindings currently implemented by the capsule viewer.

## Basic layout

Face buttons and sticks below are confirmed by the guide's labels and leaders.
Shoulder labels are confirmed; physical L/R interpretation uses the rear view of
its controller drawing (hypothesis). Keyboard/mouse bindings are project choices,
not extracted game data. Gamepad names use physical positions, avoiding the Xbox
“X” versus PlayStation Cross ambiguity.

| Action | PS2 control | Gamepad position | Keyboard/mouse | Behaviour |
|---|---|---|---|---|
| Move | Left stick | Left stick | W/A/S/D | Hold, relative to camera; strafe while locked |
| Camera | Right stick | Right stick | Mouse motion | Orbit |
| Attack | Square | West face | Left mouse | Press; repeated presses request chains |
| Grab | Triangle | North face | E | Context action |
| Dash | Circle | East face | Left Shift | Hold |
| Jump | Cross | South face | Space | Press |
| Lock-on | R1 | Right bumper | Right mouse | Hold |
| Hide/Crouch | L1 | Left bumper | Left Ctrl | Hold |
| Use item | L2 | Left trigger | Q | Press |
| Equipment screen | Select | Select/Back | Tab | Documented only; menus out of scope |
| Pause menu | Start | Start | Escape | Documented only; menus out of scope |

The guide does not label R2, D-pad, or stick clicks. Do not infer original-game
functions for those controls. The hold/press choices, movement relative to the
camera, walk/run stick threshold and dash behaviour are **ESTIMATE**: the static
image names actions but does not specify these rules. For keyboard walking, use
Alt + W/A/S/D; normal movement runs and Shift requests dash. Numeric thresholds
must be marked `ESTIMATE` in `src/sim/data.rs` when M11 implements them.

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

Additional viewer choices: R3 / middle mouse recentres the camera; D-pad left/right
or mouse wheel switches targets while locked and cycles sandbox items otherwise.
These are **ESTIMATE**, not claims about the original pad. They keep target and item
selection reachable without implementing an equipment menu. Automatic reactions,
falls, landings and idle variants require no separate button. Other unbound clips
remain available to the planned clip viewer, not through invented gameplay keys.

M11 must sample render-frame edges once and consume them once at the next 60 Hz
simulation tick; held inputs persist across ticks. This avoids the current viewer's
known missed/duplicated `just_pressed` taps, without adding gameplay input buffering.
The existing viewer still uses W to run, Shift to walk, A/D to turn, Space to attack
and G to guard; this documentation item does not change runtime controls.
