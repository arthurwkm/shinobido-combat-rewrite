# Acceptance review (first pass, 2026-10-09)
Evidence = record dir under `target/rec/` (script in `tools/scripts/`) and the clip used. Re-recorded this pass: `acc_multi`
(multi.txt, side by side with stealth.mov 8 s, `target/cmp/acc1.png`: HUD, mannequins, grid sandbox read like the clip/reference).

| Line | Evidence | Verdict |
|---|---|---|
| Walk, run, run stop | run, runcam2 vs movement.mov 12 s | matches (RUN_RE/LE wall-end variant is ESTIMATE) |
| Jumps, flips, air kicks | jmp, flip, airkick | matches |
| High jump onto rooftops | wallrun + ledge (rise 3 m ESTIMATE) | matches (launch height ESTIMATE) |
| Rolls / evasion | roll | matches |
| Attacks and chains | atk, attackmod, uppercut2 vs combat.mov | matches |
| Guard, hit reactions (player) | hit, lock4 | matches |
| Ledge grab/hang/climb | ledge, ledgecam vs movement.mov 28-32 s | matches |
| Wall hug, corner peek | hug, hug05, hug06, hugatk | matches |
| Crouch movement | crouch, dash | matches |
| Pick up, carry, throw crates/bodies | carry, carrybody vs movement.mov 36-46 s | matches |
| Items | items | differs: no caltrop/effect visuals, MAKIBISHI only unit-tested |
| Soldier skeleton, motions, AI | patrol, flee, enemy_check | matches (AI ranges ESTIMATE) |
| Soldier hit reactions, death | react, hit | differs: heavy knock-back and behind-hit never seen in the viewer |
| Lock-on, strafe, camera | lock4 vs strafe clip | matches |
| Executions and cue | exec vs stealth-kill clip | differs: front/above/B/C/D variants and cue look not seen from the pad |
| Stealth meter, eye icon, compass | multi, patrol, acc_multi | matches |
| Viewer layout, HUD both parts, controls | acc_multi, legend shot | matches |
| Mannequin proportions | acc_multi | matches |
