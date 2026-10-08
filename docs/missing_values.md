# Values the data files do not give directly

Source order: files (done below), then a targeted executable lookup, then `ESTIMATE`.

| Value | In files? | Plan |
|---|---|---|
| Action length, frames | yes: `.nmt` header/`frames` | extract |
| Root motion per action (run 3.38 m/s, step ±1.6 m, attack lunge 0.6–1.9 m) | yes: `.nmt` track 0 | extract; sample per frame |
| Footstep frames | yes: `.nmt` event type 7 | extract |
| Hit / cancel / input windows | partly: event types 3/4, 5/6 (start/end pairs by arg), 1, 49, 50, 53 | executable lookup: the event type dispatch (find the code reading `.nmt` events; type numbers are switch cases) |
| Damage, hit reach, hit stun | not found: `wpdata.bin` holds geometry only | executable lookup, else `ESTIMATE` |
| Gravity, jump arc, fall speed | `JUMP01` root track is 0 m (the jump is simulated) | executable lookup for a float near -9.8 or a per-frame constant, else `ESTIMATE` |
| Walk vs run speed selection, turn rate | run/step speeds come from root motion; analog thresholds and turn rate do not | `ESTIMATE` unless found in the controller code |
| Input buffering, combo cancel rules | no | executable lookup around the attack-chain code, else `ESTIMATE` |
