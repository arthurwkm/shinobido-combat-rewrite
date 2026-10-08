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

// ESTIMATE: BLADE07 carries no attack events (nor does any set-07 motion); swing active window in frames, mid-clip.
pub const ENEMY_ATTACK_HIT: (u32, u32) = (40, 60);
// ESTIMATE: distance (m) at which a chasing soldier starts BLADE07.
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
// ESTIMATE: paired execution clips per kind (player, victim); no paired clips exist in the files (formats.md "Execution packs")
// and the S variant of each KI_HI reaction is taken as the plain kill. Distance (m) the victim is placed from the player.
pub const EXEC_CLIPS: [(&str, &str); 3] = [("KIME_A01", "KI_HIBS07"), ("KIME_B01", "KI_HIUS07"), ("KIME_B01", "KI_HICS07")];
pub const EXEC_DIST: f32 = 0.8;
// ESTIMATE: follow camera from the clips (docs/video_notes.md: ~3 m back, pitched down 25-30 deg); the camera
// class in the executable was not read. Distance (m), default pitch (rad), pitch limits, focus height (m),
// orbit speed for mouse (rad per pixel) and right stick (rad/s), lock-on yaw follow rate (1/s).
pub const CAM_DIST: f32 = 3.0; // ESTIMATE: closer and lower to match combat.mov framing (audit 2026-10-08)
pub const CAM_PITCH: f32 = 0.3;
pub const CAM_PITCH_MIN: f32 = -0.2;
pub const CAM_PITCH_MAX: f32 = 1.2;
pub const CAM_FOCUS_HEIGHT: f32 = 1.3;
pub const CAM_MOUSE_SENS: f32 = 0.004;
pub const CAM_STICK_SPEED: f32 = 2.5;
pub const CAM_LOCK_RATE: f32 = 6.0;
// ESTIMATE: stick deflection (0..1) below which input is ignored, and below which forward input walks instead of running
// (the game's analog threshold is not decoded, PROGRESS M6b).
pub const STICK_DEAD: f32 = 0.2;
pub const STICK_WALK: f32 = 0.55;
