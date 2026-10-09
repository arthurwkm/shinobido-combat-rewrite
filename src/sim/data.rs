// ESTIMATE: upright collision dimensions match the current viewer capsule, not measured game bounds.
pub const BODY_RADIUS: f32 = 0.3;
pub const BODY_HEIGHT: f32 = 1.7;
// ESTIMATE: collision sampling distance; smaller than the body radius to prevent crossing thin walls.
pub const COLLISION_STEP: f32 = 0.05;
// ESTIMATE: support tolerance absorbs floating-point roundoff in metre coordinates.
pub const SUPPORT_EPS: f32 = 0.0001;

// ESTIMATE: caller selects the run/idle family, as with walk_clip; no decoded wall or fatigue trigger.
// ponytail: RE/LE and STOP_K use one-shot Special; replace caller selection when wall states exist.
pub const RUN_CLIPS: &[&str] = &["RUN01", "RUN_R01", "RUN_L01", "RUN_D05"];
pub const IDLE_CLIPS: &[&str] = &["STOP01", "STOP02", "TIRED01"];

// ESTIMATE: fall drop (m) from which the landing clip is LAND_B01 / LAND_A01 instead of LAND01; no decoded rule.
pub const LAND_B_DROP: f32 = 1.5;
pub const LAND_A_DROP: f32 = 4.0;

// Set 07 has no swing (BLADE07 slides the body 2 m and carries DIE_A07's event 26: a death clip). The katana
// fighter's swings are set 19 (AT_A19/AT_B19 with their own hit events 20-40 / 3-42; 26-track clips, docs/formats.md).
// ESTIMATE: that set 19 belongs to the soldier (it has DRAW19/GUARD_*19/AT_* and the AI's katana tables), alternated per swing.
pub const ENEMY_ATTACK_CLIPS: &[&str] = &["AT_A19", "AT_B19"];
// ESTIMATE: player reaction to the soldier's swing; HI_BC01 is a 40-frame flinch in place (HI_BS01 is a 160-frame
// paired reaction whose hips start 1.3 m away). Picking by hit side/height is an open M14 item.
pub const PLAYER_HIT_CLIP: &str = "HI_BC01";
// ESTIMATE: horizontal air speed (m/tick) while the stick is held in a jump or fall: RUN01's 1.69 m / 30 f.
/// ESTIMATE: stick turned this far (rad) from the take-off heading counts as the back flick (about 130 degrees).
pub const AIR_BACK_ANGLE: f32 = 2.3;
pub const AIR_SPEED: f32 = 1.69 / 30.0;
// ESTIMATE: attack-object damage (event arg0 x power) from which the soldier is thrown back instead of flinching.
pub const HEAVY_HIT: i32 = 4;
// ESTIMATE: distance (m) at which a chasing soldier starts its swing.
pub const ENEMY_ATTACK_RANGE: f32 = 1.5;
// ESTIMATE: soldier hit points; the executable's health values were not looked up.
pub const ENEMY_HP: i32 = 100;
// ESTIMATE: player hit points, attacker power stat for `hit_damage`, enemy swing damage, melee reach (m) and the
// half-angle (rad) of the frontal cone for hits and guard; none found in the files.
pub const PLAYER_HP: i32 = 100;
pub const PLAYER_POWER: f32 = 1.0;
pub const ENEMY_DAMAGE: i32 = 20;
pub const MELEE_REACH: f32 = 2.0;
pub const FRONT_CONE: f32 = std::f32::consts::FRAC_PI_2;
// ESTIMATE: lock-on pick range (m) and half-angle (rad) around the facing; the camera/target-select code was not read.
pub const LOCK_RANGE: f32 = 12.0;
pub const LOCK_CONE: f32 = std::f32::consts::FRAC_PI_2;
// ESTIMATE: stealth numbers; sight/hearing ranges are not in the AI files (formats.md "Enemy AI"), none read from the executable.
// Sight range (m) and half-angle (rad); meter fill per second when seen at point-blank (scaled down with distance);
// hearing radius (m) while running / walking; meter fill per second when heard; meter decay per second unperceived;
// meter thresholds for Suspicious and Alerted.
pub const SIGHT_RANGE: f32 = 12.0;
pub const SIGHT_CONE: f32 = std::f32::consts::FRAC_PI_3;
pub const SIGHT_FILL: f32 = 1.5;
pub const HEAR_RUN: f32 = 8.0;
pub const HEAR_WALK: f32 = 3.0;
// ESTIMATE: crouching (STOP_S05 / WALK_*05) is quieter and less visible: hearing radius (m) and sight-range factor.
pub const HEAR_CROUCH: f32 = 1.0;
pub const SIGHT_CROUCH: f32 = 0.6;
pub const HEAR_FILL: f32 = 0.5;
pub const STEALTH_DECAY: f32 = 0.2;
pub const SUSPICIOUS_AT: f32 = 0.4;
// ESTIMATE: execution conditions (no stealth-kill clip yet, PROGRESS Requests). Max horizontal reach (m) of any execution;
// minimum height (m) above the soldier for the from-above kill; minimum angle (rad) off the soldier's facing for the
// from-behind kill; hp fraction at or below which a staggered soldier can be finished from the front.
pub const EXEC_REACH: f32 = 1.5;
pub const EXEC_ABOVE_MIN: f32 = 2.0;
pub const EXEC_BEHIND_MIN: f32 = 2.4;
pub const EXEC_FRONT_HP: f32 = 0.25;
// ESTIMATE: placeholder execution pairs; HI_* phase continuity now argues against the KIME mapping
// (formats.md "Execution phase audit"). Distance (m) ahead; align player height to victim height, never lift the victim.
pub const EXEC_CLIPS: [(&str, &str); 3] = [("KIME_A01", "KI_HIBS07"), ("KIME_B01", "KI_HIUS07"), ("KIME_B01", "KI_HICS07")];
pub const EXEC_DIST: f32 = 0.8;
// ESTIMATE: follow camera from the clips (docs/video_notes.md: ~3 m back, pitched down 25-30 deg); the camera
// class in the executable was not read. Distance (m), default pitch (rad), pitch limits, focus height (m),
// orbit speed for mouse (rad per pixel) and right stick (rad/s), lock-on yaw follow rate (1/s).
pub const CAM_DIST: f32 = 3.0; // ESTIMATE: closer and lower to match combat.mov framing (audit 2026-10-08)
pub const CAM_HUG_DIST: f32 = 4.2; // ESTIMATE: movement.mov 38 s: player ~10% of frame height, camera farther than the 3 m follow, looking along the wall
// ESTIMATE: execution close-up (combat.mov 46 s, 54-58 s show the pair large and seen from the side): distance, pitch,
// side angle off the player's heading, blend rate.
pub const CAM_EXEC_DIST: f32 = 2.4;
pub const CAM_EXEC_PITCH: f32 = 0.2;
pub const CAM_EXEC_ANGLE: f32 = 1.2;
pub const CAM_EXEC_RATE: f32 = 6.0;
pub const CAM_HUG_RATE: f32 = 4.0;
// ESTIMATE: side view 60 degrees from the outward wall normal; awaiting movement 38 s comparison.
pub const CAM_HUG_ANGLE: f32 = std::f32::consts::FRAC_PI_3;
pub const CAM_PITCH: f32 = 0.3;
pub const CAM_PITCH_MIN: f32 = -0.2;
pub const CAM_PITCH_MAX: f32 = 1.2;
pub const CAM_FOCUS_HEIGHT: f32 = 1.3;
pub const CAM_MOUSE_SENS: f32 = 0.004;
pub const CAM_STICK_SPEED: f32 = 2.5;
pub const CAM_LOCK_RATE: f32 = 6.0;
// ESTIMATE: target switch is not in Shinobido; copied from Elden Ring's flick (owner, 2026-10-09). A sideways
// right stick past SWITCH_FLICK (or mouse moving SWITCH_MOUSE_PX in one frame) switches once; back under SWITCH_REARM re-arms.
pub const SWITCH_FLICK: f32 = 0.8;
pub const SWITCH_REARM: f32 = 0.3;
pub const SWITCH_MOUSE_PX: f32 = 40.0;
// ESTIMATE: stick deflection (0..1) below which input is ignored, and below which forward input walks instead of running
// (the game's analog threshold is not decoded, PROGRESS M6b).
pub const STICK_DEAD: f32 = 0.2;
pub const STICK_WALK: f32 = 0.55;
// ESTIMATE: least wall height (m) for a hug; direct entry/exit and contact-driven transitions
// have no event windows in the set-05 hug clips. LOOK/BACK sequencing follows matching endpoint poses.
pub const HUG_WALL_MIN: f32 = 2.0;

// ESTIMATE: height (m) gained over KABE_U01 (36 f); the clip has no root Y and the launch impulse binding was not found (formats.md "Wall ascent and impulse").
pub const WALL_RUN_RISE: f32 = 3.0;
// ESTIMATE: horizontal speed (m/tick) away from the wall after TRI_J01 / KABE_E01.
pub const KICK_SPEED: f32 = 0.05;

// ESTIMATE: ledge catch window. The hands reach the top while it is HANG_MIN..LEDGE_REACH above the feet, within
// LEDGE_GRAB_DIST of the face; hanging feet sit CLIMB06's root rise below the top, HANG_OFFSET out from the face
// (CLIMB06's forward root motion then ends the body on the top). No catch rule was found in the files.
pub const LEDGE_REACH: f32 = 2.4;
pub const HANG_MIN: f32 = 1.5;
pub const LEDGE_GRAB_DIST: f32 = 0.5;
pub const HANG_OFFSET: f32 = 0.15;
// ESTIMATE: ground vault (CLIMB_B01, hips rise ~1.8 m in the clip; movement 24-26 s, 40 s). A box top VAULT_MIN..VAULT_MAX
// above the feet, within VAULT_DIST of the face, is vaulted by Jump; lower tops are jumped, higher ones are wall-run.
pub const VAULT_MIN: f32 = 1.2;
pub const VAULT_MAX: f32 = 2.3;
pub const VAULT_DIST: f32 = 0.8;

// ESTIMATE: grab and throw (docs/controls.md: Triangle grabs, mash Square/Triangle for a stealth kill, R1+Triangle throws
// him downed). No paired grab clips exist in the files (formats.md "Execution packs"): the player reaches with PICK01,
// the held soldier struggles in GU_B07, the throw is THROW_BACK01 with DROP07 (40 f fall: no thrown-victim clip exists in set 07, NAGE_A07 is the soldier throwing; hips 1.45 to 0.33) then UP_FR07 get-up.
// Reach (m), frames before he breaks free, presses to force the kill (halved from behind or at low hp), throw damage.
pub const GRAB_CLIP: &str = "PICK01";
pub const THROW_CLIP: &str = "THROW_BACK01";
pub const GRAB_REACH: f32 = 1.2;
pub const GRAB_HOLD: u32 = 180;
pub const GRAB_MASH: u32 = 4;
pub const THROW_DAMAGE: i32 = 30;

// ESTIMATE: roll into the enemy (R1 + Forward + Cross, docs/controls.md). No roll-over clip exists beyond FRONT01, so the
// forward evade is stretched to pass him: starts within ROLL_REACH m and ROLL_CONE rad of the heading, ends ROLL_BEHIND m past him.
pub const ROLL_REACH: f32 = 4.0;
pub const ROLL_CONE: f32 = 0.8;
pub const ROLL_BEHIND: f32 = 1.0;

// ESTIMATE: pick up, carry, put down and throw crates and the dead soldier (movement.mov 36-44 s; no pairing in the files).
// PICK01 lifts, STOP_K05/RUN_K05 carry, PUT_K05 (60 f) sets down at frame PUT_FRAME, NAGE_K05 (40 f) throws at THROW_FRAME.
// Reach is measured to the object's centre; the load rides HOLD_FWD ahead and HOLD_UP above the feet.
pub const PICK_REACH: f32 = 1.6;
pub const PICK_CONE: f32 = 0.9;
pub const HOLD_FWD: f32 = 0.1;
pub const HOLD_UP: f32 = 1.7;
pub const PUT_FRAME: u32 = 30;
pub const PUT_DIST: f32 = 0.9;
pub const THROW_FRAME: u32 = 14;
pub const THROW_SPEED: f32 = 8.0;
pub const THROW_LIFT: f32 = 1.0;
pub const THROWN_HIT_RADIUS: f32 = 0.8;
pub const THROWN_DAMAGE: i32 = 25;
pub const GRAVITY_MS2: f32 = 9.8;

// ESTIMATE: items (docs/controls.md: the item button uses the selected item; nothing in the files says what each does).
// Starting counts [DRINK, MAKIBISHI, NINJYUTU], hp restored by DRINK, caltrops dropped CALTROP_FWD ahead that slow the
// soldier to CALTROP_SLOW within CALTROP_RADIUS, NINJYUTU staggers him within range (m) and cone (rad) without damage.
pub const ITEM_COUNTS: [u32; 3] = [3, 5, 2];
pub const DRINK_HEAL: i32 = 40;
pub const CALTROP_FWD: f32 = 1.0;
pub const CALTROP_RADIUS: f32 = 1.2;
pub const CALTROP_SLOW: f32 = 0.4;
pub const NINJYUTU_RANGE: f32 = 4.0;
pub const NINJYUTU_CONE: f32 = 0.8;

// ESTIMATE: AI\battle tables are not decoded to behaviour; patrol leg length (m), arrival radius (m).
pub const PATROL_LEN: f32 = 5.0;
pub const ARRIVE: f32 = 0.5;
// ESTIMATE: chase is abandoned after GIVE_UP frames (5 s) with the player beyond LOSE_RANGE (m).
pub const LOSE_RANGE: f32 = 15.0;
pub const GIVE_UP: u32 = 300;
// ESTIMATE: every GUARD_EVERY-th time he is in range he blocks for GUARD_HOLD frames instead of swinging.
pub const GUARD_EVERY: usize = 3;
pub const GUARD_HOLD: u32 = 90;
