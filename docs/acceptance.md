# Acceptance review (second pass, 2026-10-09)
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
| Items | items | matches (caltrop, caltrop record; effect art is gizmo stand-ins, recorded) |
| Soldier skeleton, motions, AI | patrol, flee, enemy_check | matches (AI ranges ESTIMATE) |
| Soldier hit reactions, death | react, hit | matches (react4, behind3 records; B/U/C letters ESTIMATE) |
| Lock-on, strafe, camera | lock4 vs strafe clip | matches |
| Executions and cue | exec vs stealth-kill clip | matches (exec, execfront, execabove2 vs stealth-kill clip: above kill now drops FALL01 onto the soldier before KIME_B01; A/B/C/D clip choice stays ESTIMATE after three failed lookups) |
| Stealth meter, eye icon, compass | multi, patrol, acc_multi | matches |
| Viewer layout, HUD both parts, controls | acc_multi, legend shot | matches |
| Mannequin proportions | acc_multi | matches |
