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
