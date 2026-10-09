# Emulator A/B

Same pad script in PCSX2 (`.venv/bin/python tools/pcsx2.py video <script> <name>`, game window recorded at 60 fps,
the PAL game renders 25 fps) and in the viewer (`target/debug/viewer --record target/rec/<name> --script <file> --solo`),
tiled with `tools/emucmp.py`. Scripts in `tools/scripts/emu/` (`<name>.txt` for PCSX2, `<name>_viewer.txt` for the viewer).
The emulator is the ground truth over the clips and estimates. Test spot: open grass field (owner's save, 2026-10-09).

| Action | Script | Game (emulator) | Viewer | Verdict |
|---|---|---|---|---|
| Standing jump | `jump` | airborne 18 frames at 25 fps = 0.72 s; fast rise ~0.16 s, ~0.3 s hang, fast fall; feet ~1.5 m up | before: apex 0.57 m, ~1.0 s. After `JUMP_V0` 120 (exe 0x604a28) and `UNIT_M` 0.84/9.5: apex 1.49 m, lands at the same script tick (84) | matches (timing, height); camera differs: the game's rises with him |
| Run, stop, turn | `loco` | camera stays low behind; running towards the camera (stick back) he turns and faces it, camera does not swing behind | camera swings behind him; frames him higher and farther | differs: camera (M18 camera item) |
| Running jump | `loco` (570–720) | airborne 18 frames at 25 fps like the standing jump, run speed carried; running again ~2 game frames after touchdown | before: LAND01 (26 f) always played out. Now a held stick leaves LAND01 from its type-5 event frame 2; RUN01 at the same tick (690) | matches |
