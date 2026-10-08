// ESTIMATE: upright collision dimensions match the current viewer capsule, not measured game bounds.
pub const BODY_RADIUS: f32 = 0.3;
pub const BODY_HEIGHT: f32 = 1.7;
// ESTIMATE: collision sampling distance; smaller than the body radius to prevent crossing thin walls.
pub const COLLISION_STEP: f32 = 0.05;
// ESTIMATE: support tolerance absorbs floating-point roundoff in metre coordinates.
pub const SUPPORT_EPS: f32 = 0.0001;

// ESTIMATE: caller selects the run/idle family, as with walk_clip; no decoded wall or fatigue trigger.
// ponytail: RE/LE and STOP_K use one-shot Special; replace caller selection when wall states exist.
pub const RUN_CLIPS: &[&str] = &["RUN01", "RUN_R01", "RUN_L01"];
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
pub const HEAR_FILL: f32 = 0.5;
pub const STEALTH_DECAY: f32 = 0.2;
pub const SUSPICIOUS_AT: f32 = 0.4;
