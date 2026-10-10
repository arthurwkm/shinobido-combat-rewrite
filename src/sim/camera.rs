//! Orbit follow camera (engine-free), ported from the reference project's `camera.rs` (owner decision 2026-10-09).
//! `yaw` is the direction the camera looks along, like `Player::heading` (forward = (sin yaw, cos yaw)); `pitch` > 0
//! looks down. Wall hug, execution and climb framings are layered on top.

use super::data::*;

pub struct Camera {
    pub yaw: f32,
    pub pitch: f32,
    /// 0 = normal framing, 1 = pulled in beside the wall (wall hug).
    pub pull: f32,
    /// 0 = normal framing, 1 = close-up side cut of an execution.
    pub cut: f32,
    /// 0 = normal pitch, 1 = lowered to look up a climb (rooftop, movement 28-32 s).
    pub tilt: f32,
    /// Smoothed feet position the camera orbits; `None` until the first `track`.
    pub pivot: Option<[f32; 3]>,
    /// Horizontal velocity of the pivot (m/s), for the idle drift.
    pub vel: [f32; 2],
    /// Seconds since the player last moved the camera.
    pub idle: f32,
    /// Yaw the camera is swinging round to (lock-on pressed with no target).
    pub recenter: Option<f32>,
}

impl Default for Camera {
    fn default() -> Self {
        Camera { yaw: 0.0, pitch: CAM_PITCH, pull: 0.0, cut: 0.0, tilt: 0.0, pivot: None, vel: [0.0; 2], idle: 0.0, recenter: None }
    }
}

fn ease(rate: f32, dt: f32) -> f32 {
    1.0 - (-rate * dt.max(0.0)).exp()
}

fn wrap(a: f32) -> f32 {
    use std::f32::consts::{PI, TAU};
    (a + PI).rem_euclid(TAU) - PI
}

impl Camera {
    /// Orbit by `d` = (yaw, pitch) radians (mouse pixels * sens, or stick * speed * DT); right/down input is +.
    pub fn orbit(&mut self, d: [f32; 2]) {
        self.yaw = wrap(self.yaw + d[0]);
        self.pitch = (self.pitch + d[1]).clamp(CAM_PITCH_MIN, CAM_PITCH_MAX);
    }

    /// Free camera for one render frame: `d` is this frame's manual orbit (as `orbit`), zero when untouched.
    /// Untouched, it finishes a recentre, else after CAM_DRIFT_IDLE s drifts round to follow sideways movement
    /// (running straight at or away from it leaves it be).
    pub fn look(&mut self, d: [f32; 2], dt: f32) {
        if d != [0.0; 2] {
            self.idle = 0.0;
            self.recenter = None;
            self.orbit(d);
            return;
        }
        self.idle += dt;
        if let Some(goal) = self.recenter {
            let k = ease(CAM_RECENTER_RATE, dt);
            self.yaw = wrap(self.yaw + wrap(goal - self.yaw) * k);
            self.pitch += (CAM_PITCH - self.pitch) * k;
            if wrap(goal - self.yaw).abs() < 0.01 && (CAM_PITCH - self.pitch).abs() < 0.01 {
                self.recenter = None;
            }
        } else if self.idle > CAM_DRIFT_IDLE && self.vel[0].hypot(self.vel[1]) > CAM_DRIFT_SPEED {
            let off = wrap(self.vel[0].atan2(self.vel[1]) - self.yaw);
            self.yaw = wrap(self.yaw + off.sin() * CAM_DRIFT_RATE * dt);
        }
    }

    /// Swing round to look along `yaw` (behind a player facing it).
    pub fn recenter_on(&mut self, yaw: f32) {
        self.recenter = Some(yaw);
    }

    /// Lock-on: ease the view toward looking from the player at `target` (both x/z in world frame), and the pitch to
    /// CAM_LOCK_PITCH. `dt` is elapsed render time.
    pub fn follow_lock(&mut self, player: [f32; 3], target: [f32; 3], dt: f32) {
        self.recenter = None;
        let k = ease(CAM_LOCK_RATE, dt);
        let want = (target[0] - player[0]).atan2(target[2] - player[2]);
        self.yaw = wrap(self.yaw + wrap(want - self.yaw) * k);
        self.pitch += (CAM_LOCK_PITCH - self.pitch) * k;
    }

    /// Follow the player's feet at `pos`: tight horizontally, lazier vertically, snapping when far behind.
    /// Returns the smoothed position to pass to `eye`.
    pub fn track(&mut self, pos: [f32; 3], dt: f32) -> [f32; 3] {
        let mut p = self.pivot.unwrap_or(pos);
        if (0..3).map(|i| (pos[i] - p[i]).powi(2)).sum::<f32>().sqrt() > CAM_SNAP {
            p = pos;
        }
        let (flat, lift) = (ease(CAM_FOLLOW_RATE, dt), ease(CAM_LIFT_RATE, dt));
        let next = [p[0] + (pos[0] - p[0]) * flat, p[1] + (pos[1] - p[1]) * lift, p[2] + (pos[2] - p[2]) * flat];
        if dt > 0.0 {
            self.vel = [(next[0] - p[0]) / dt, (next[2] - p[2]) / dt];
        }
        self.pivot = Some(next);
        next
    }

    /// Swing round to face the wall and the player on it (PCSX2 hug3-5: the game's hug view is square-on, so the
    /// sideways stick slides); heading points away from the wall. `dt` is elapsed render time.
    pub fn follow_hug(&mut self, heading: Option<f32>, dt: f32) {
        let blend = ease(CAM_HUG_RATE, dt);
        let want = if heading.is_some() { 1.0 } else { 0.0 };
        self.pull += (want - self.pull) * blend;
        if let Some(heading) = heading {
            self.yaw = wrap(self.yaw + wrap(heading + std::f32::consts::PI - self.yaw) * blend);
        }
    }

    /// Execution close-up: swing to the player's side (heading given) and come in; `None` eases back (yaw is kept).
    pub fn follow_exec(&mut self, heading: Option<f32>, dt: f32) {
        let blend = ease(CAM_EXEC_RATE, dt);
        self.cut += (if heading.is_some() { 1.0 } else { 0.0 } - self.cut) * blend;
        if let Some(h) = heading {
            self.yaw = wrap(self.yaw + wrap(h + CAM_EXEC_ANGLE - self.yaw) * blend);
        }
    }

    /// Ease the climb tilt in or out; `dt` is elapsed render time.
    pub fn follow_climb(&mut self, climbing: bool, dt: f32) {
        let blend = ease(CAM_HUG_RATE, dt);
        self.tilt += (if climbing { 1.0 } else { 0.0 } - self.tilt) * blend;
    }

    /// (eye, focus) in the world frame for a player at `pos`.
    pub fn eye(&self, pos: [f32; 3]) -> ([f32; 3], [f32; 3]) {
        let f = [pos[0], pos[1] + CAM_FOCUS_HEIGHT, pos[2]];
        let pitch = self.pitch + CAM_CLIMB_PITCH * self.tilt + (CAM_EXEC_PITCH - self.pitch) * self.cut;
        let (cp, sp) = (pitch.cos(), pitch.sin());
        let dist = CAM_DIST + (CAM_HUG_DIST - CAM_DIST) * self.pull;
        let dist = dist + (CAM_EXEC_DIST - dist) * self.cut;
        let e = [f[0] - self.yaw.sin() * cp * dist, f[1] + sp * dist, f[2] - self.yaw.cos() * cp * dist];
        (e, f)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sim::DT;

    #[test]
    fn hug_camera_uses_elapsed_time_and_sits_outside_wall() {
        let sample = |hz: usize| {
            let mut c = Camera::default();
            for _ in 0..hz { c.follow_hug(Some(0.0), 1.0 / hz as f32); }
            c
        };
        let a = sample(30);
        let mut b = sample(120);
        assert!((a.pull - b.pull).abs() < 1e-5);
        assert!(wrap(a.yaw - b.yaw).abs() < 1e-5);
        let (eye, focus) = b.eye([0.0; 3]);
        assert!(eye[2] > focus[2], "wall is behind the outward-facing player");
        assert!(wrap(b.yaw - std::f32::consts::PI).abs() < 0.1, "turns square-on to the wall, as in PCSX2: {}", b.yaw);
        let yaw = b.yaw;
        for _ in 0..120 { b.follow_hug(None, DT); }
        assert!(b.pull < 0.001);
        assert_eq!(b.yaw, yaw, "release preserves manual orbit");
    }

    #[test]
    fn execution_cut_comes_in_to_the_side_and_releases() {
        let mut c = Camera::default();
        for _ in 0..120 { c.follow_exec(Some(0.0), DT); }
        let (e, f) = c.eye([0.0; 3]);
        let d = ((e[0] - f[0]).powi(2) + (e[1] - f[1]).powi(2) + (e[2] - f[2]).powi(2)).sqrt();
        assert!((d - CAM_EXEC_DIST).abs() < 1e-2, "close to the pair");
        assert!((e[0] + CAM_EXEC_ANGLE.sin() * CAM_EXEC_PITCH.cos() * CAM_EXEC_DIST).abs() < 1e-2,
            "side follows the camera yaw convention");
        let mut slow = Camera::default();
        for _ in 0..60 { slow.follow_exec(Some(0.0), 1.0 / 30.0); }
        assert!((slow.cut - c.cut).abs() < 1e-5);
        assert!(wrap(slow.yaw - c.yaw).abs() < 1e-5);
        let yaw = c.yaw;
        for _ in 0..120 { c.follow_exec(None, DT); }
        assert!(c.cut < 0.001);
        assert_eq!(c.yaw, yaw, "release preserves orbit");
    }

    #[test]
    fn climb_tilt_lowers_the_eye_and_wall_clips_it() {
        let mut c = Camera::default();
        let (e0, _) = c.eye([0.0; 3]);
        for _ in 0..120 { c.follow_climb(true, DT); }
        assert!(c.eye([0.0; 3]).0[1] < e0[1]);
        let t = super::super::terrain::Terrain::sandbox();
        let f = [0.0, 1.3, 0.0];
        assert_eq!(t.camera_clip(f, f), f);
        let w = t.colliders.iter().find(|b| !b.ramp && b.max[1] > 3.0).unwrap();
        let inside = [(w.min[0] + w.max[0]) / 2.0, 1.5, (w.min[2] + w.max[2]) / 2.0];
        let clipped = t.camera_clip([inside[0], 1.5, w.min[2] - 3.0], inside);
        assert!(clipped[2] < w.min[2], "stopped outside the wall");
    }

    #[test]
    fn camera_sits_behind_orbits_and_locks_on() {
        let mut c = Camera::default();
        let (e, f) = c.eye([0.0; 3]);
        assert!(e[2] < f[2] && e[1] > f[1], "behind and above at yaw 0");
        assert!((((e[0] - f[0]).powi(2) + (e[1] - f[1]).powi(2) + (e[2] - f[2]).powi(2)).sqrt() - CAM_DIST).abs() < 1e-4);
        c.orbit([0.0, 10.0]);
        assert_eq!(c.pitch, CAM_PITCH_MAX);
        c.orbit([std::f32::consts::PI * 1.5, -10.0]);
        assert!(c.yaw < 0.0 && c.pitch == CAM_PITCH_MIN, "yaw wraps, pitch clamps");
        let mut c = Camera::default();
        for _ in 0..120 {
            c.follow_hug(Some(0.0), DT);
        }
        let (e, f) = c.eye([0.0; 3]);
        assert!((((e[0] - f[0]).powi(2) + (e[1] - f[1]).powi(2) + (e[2] - f[2]).powi(2)).sqrt() - CAM_HUG_DIST).abs() < 1e-2, "pulls in");
        let mut c = Camera::default();
        for _ in 0..120 {
            c.follow_lock([0.0; 3], [5.0, 0.0, 0.0], DT);
        }
        assert!((c.yaw - std::f32::consts::FRAC_PI_2).abs() < 1e-2, "looks along +x at the target");
    }

    #[test]
    fn reference_camera_follows_drifts_recentres_and_locks() {
        let mut c = Camera::default();
        assert_eq!(c.track([0.0; 3], DT), [0.0; 3]);
        let p = c.track([0.1, 0.1, 0.0], DT);
        assert!(p[0] / 0.1 > p[1] / 0.1, "horizontal follow is tighter than vertical");
        assert!(c.track([20.0, 0.0, 0.0], DT)[0] > 19.0, "snaps when far behind");
        // Running sideways (+x) with the camera looking along +z: after the idle second it turns towards +x.
        let mut c = Camera::default();
        let mut x = 0.0;
        for _ in 0..180 {
            x += 5.0 * DT;
            c.track([x, 0.0, 0.0], DT);
            c.look([0.0; 2], DT);
        }
        assert!(c.yaw > 0.1, "drifted toward the movement: {}", c.yaw);
        // Running straight away leaves it be.
        let mut c = Camera::default();
        let mut z = 0.0;
        for _ in 0..180 {
            z += 5.0 * DT;
            c.track([0.0, 0.0, z], DT);
            c.look([0.0; 2], DT);
        }
        assert!(c.yaw.abs() < 1e-3);
        // Standing still from here. Manual input cancels a recentre; untouched, it swings round and resets the pitch.
        c.vel = [0.0; 2];
        c.recenter_on(2.0);
        c.look([0.0, 0.5], DT);
        for _ in 0..60 { c.look([0.0; 2], DT); }
        assert!(c.yaw.abs() < 0.1, "cancelled");
        c.recenter_on(2.0);
        for _ in 0..120 { c.look([0.0; 2], DT); }
        assert!((c.yaw - 2.0).abs() < 0.02 && (c.pitch - CAM_PITCH).abs() < 0.02);
        for _ in 0..120 { c.follow_lock([0.0; 3], [0.0, 0.0, 5.0], DT); }
        assert!((c.pitch - CAM_LOCK_PITCH).abs() < 1e-3 && c.yaw.abs() < 1e-3);
    }
}
