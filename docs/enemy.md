# Enemy: the foot soldier (Bushi)

Pick (decision, partly hypothesis): **I_BUSHI** with motion **set `07`**.

## Why
- `AI\ActionSet\` has a `Bushi_*` family of 8 behaviours (`Al Ba Bl Fr Km Pt St Wt`, the largest family; `AkaBushi_*` has 5): the common soldier.
- Models: `model\I_BUSHI_BODY.rws` (+ `I_BUSHI_HEAD_H/L`, `I_BUSHI_CF_*` variant). Its HAnim skeleton has 28 nodes, the same as
  `KAGETORA_BODY.rws` (checked with `tools/rws.py skeleton`), and every set-`07` `.nmt` has 24 tracks like the player's, so
  the player rig and animation code is reused. `A_BUSHI1` (red soldier) is the viewer's own sample enemy (`viewer\bodymdl.txt`); its body is only in `kidsModel\`.
- Set `07` is a full parallel of the player's `01` set (`HI_*07` reactions, `GU_A/B/Y07`, `STOP/RUN/HALT/TURN/JUMP/LAND07`)
  plus enemy-only behaviour (`PATROL`, `SEARCH`, `LOOK`, `SURP`, `ALARM`, `DIE_A/B`) and the execution victim `KI_HI*07` (docs/executions.md).
  Hypothesis: the actor-to-set binding lives in `chara\act2mtn.bin` / the executable motion registration (not decoded).

## Motions (frames at 60 fps)
| role | motion |
|---|---|
| idle | STOP07 160 |
| patrol walk | WALK_A07 72, PATROL07 94 |
| run / chase | RUN07 33, RUN_A07 38, HALT07 58, TURN07 36 |
| look around / search | LOOK07 112, SEARCH07 156 |
| alert (noticed) | SURP07 32, ALARM07 50 (ALARM_A/B07) |
| guard | GU_A07 10, GU_B07 70, GU_Y07 25 |
| hit | HI_*07 (parallel to player HI_*01, e.g. HI_BS07 50) |
| death | DIE_A07 156, DIE_B07 168 |
| execution victim | KI_HI*07 (e.g. KI_HIBS07 100) |
| attack | **ESTIMATE**: BLADE07 111 (sword slash by name); no `AT_*07` exists. Alternatives from set `19`: AT_A19 90, AT_B19 75, AT_CA19 |

Other set-`07` motions (CARRY, CLIMB, DRINK, FUNDA, SUPLEX, STOMP, SIT, TALK...) are out of scope.
