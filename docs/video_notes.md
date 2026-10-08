# Video catalogue
Read from `videos/shinobido *.mov` as 2 s contact sheets (`target/vid/`, not committed), so every row is a
coarse observation (hypothesis), not a frame-exact timing. Motion names are guesses from the
`docs/player_motions.md` groups. Finer frame reads are still to do per action when an M9 item needs them.

## HUD (all clips)
- Health: red bar top-left, small spiked emblem at its left end. Always drawn.
- Ki compass: spiked ring top-right with an orange needle; the needle turns with the camera/enemy bearing.
- Eye awareness icon: top-centre, small red rectangle with an eye, only in combat (enemy aware) and stealth
  clips; absent in the movement clip (no enemies). Red in combat; stealth 8–16 s shows it green/teal-ish when unseen
  (hypothesis, colours not sampled).
- Tutorial text box (movement 0 s): "To jump, press the X button" — controls come from the PS2 pad (X = jump).
- Katana drawn in combat/stealth (held low in the right hand, blade back); no sword in movement clip.

## Camera (all clips)
Third person, behind and above the player, ~3 m back, pitched down ~25–30°, no visible lag at 2 s sampling.
Pulls close on a wall/corner (movement 38 s) and in tight corridors. Close-up cinematic cut on executions
(combat 46 s, 54–58 s; stealth 14 s). Rooftop climbs tilt the camera up (movement 28–32 s). Lock-on camera not seen.

## movement (55 s)
| t (s) | action | candidate motions |
|---|---|---|
| 0–2 | tutorial, jump over crate | JUMP01, LAND01 |
| 2–16 | run across yard, turns, halts | RUN01, HALT01, STOP02 |
| 10, 16 | run stop with cape swing / crouch landing | STOP_K01, LAND_A/B01 |
| 20–22 | jump up to wall base, kick off wall | TRI_J01, KABE_U01 |
| 24–26 | climb stacked crates (jump from one to next) | JUMP01, CLIMB_B01 |
| 28–32 | hang on wall below roof, climb up wall | HANG_L/R06, CLIMB06, KABE_U01 |
| 30 | on tiled rooftop, crouched | crouch `_S` set (05) |
| 34 | crouch beside crate | crouch idle |
| 36 | pick up / handle a body or crate | PICK01 |
| 38 | wall hug against stone wall | STOP_A/B06 |
| 40 | crawl/crouch over crate top | CLIMB_B01 |
| 42–44 | carry crate overhead, walk | PICK01, carry `_K` set (05) |
| 46–48 | run to gate, jump | JUMP_F02 |
| 50–54 | run down lane | RUN01 |

## combat (67 s, narrow stone-walled corridor)
| t (s) | action | candidate motions |
|---|---|---|
| 0–10 | face one foot soldier (crouched, sword low), sidestep, approach | WALK_*02, FRONT01/LEFT01 |
| 12 | sword slash exchange | AT_A01 |
| 14, 16 | evade / sheathed-sword step back, enemy slash | BACK01, HI_* |
| 18–22 | enemy swings, player low run | RUN01 |
| 24–26 | attack chain at enemy against wall | AT_A01→AT_C01 |
| 28–30 | enemy knocked off feet / falls | enemy death/hit (set 07) |
| 34 | player leaps up a wall (wall run) | KABE_U01 |
| 36–44 | repeated exchanges, enemy guards (sword up) | GU_*, AT_* |
| 46 | close-up cut: execution start, both locked | execution pair (harakiri) |
| 54–58 | player executes enemy from behind, second enemy watching | execution |
| 60–66 | enemy staggers, then player stands over corpse, sword down | HI_*, SHEATH01 |

## stealth (18.6 s, house garden at night)
| t (s) | action | candidate motions |
|---|---|---|
| 0–4 | walk along path sword drawn; camera very close | WALK_F02 |
| 6–8 | player crouches/rolls on grass; enemy appears top (orange silhouette) | crouch `_S`, UKEMI_A01 |
| 10 | lying low on grass, enemy walking by | crouch/prone |
| 12 | sneak behind enemy on path | crouch walk |
| 14 | stealth kill from behind (close cut) | execution |
| 16 | after kill, standing at wall | idle |

## Open questions (for later frame-exact work)
- Exact eye icon colours per alert state, stealth meter fill: needs full-res crops of the top-centre HUD.
- Lock-on, guard against an attack, execution cue before the input: see Requests in `PROGRESS.md`.
