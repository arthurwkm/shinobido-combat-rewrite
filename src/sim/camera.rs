//! Orbit follow camera (engine-free). `yaw` is the direction the camera looks along, like `Player::heading`
//! (forward = (sin yaw, cos yaw)); parameters are ESTIMATE (data.rs).

use super::data::*;
use super::DT;

pub struct Camera {
    pub yaw: f32,
    pub pitch: f32,
    /// 0 = normal framing, 1 = pulled in beside the wall (wall hug).
    pub pull: f32,
}

impl Default for Camera {
    fn default() -> Self {
        Camera { yaw: 0.0, pitch: CAM_PITCH, pull: 0.0 }
    }
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

    /// Lock-on: ease the view toward looking from the player at `target` (both x/z in world frame).
    pub fn follow_lock(&mut self, player: [f32; 3], target: [f32; 3]) {
        let want = (target[0] - player[0]).atan2(target[2] - player[2]);
        self.yaw = wrap(self.yaw + wrap(want - self.yaw) * (CAM_LOCK_RATE * DT).min(1.0));
    }

    /// Pull in beside the wall; heading points away from it. `dt` is elapsed render time.
    pub fn follow_hug(&mut self, heading: Option<f32>, dt: f32) {
        let blend = 1.0 - (-CAM_HUG_RATE * dt.max(0.0)).exp();
        let want = if heading.is_some() { 1.0 } else { 0.0 };
        self.pull += (want - self.pull) * blend;
        if let Some(heading) = heading {
            let centre = heading + std::f32::consts::PI;
            let left = wrap(centre - CAM_HUG_ANGLE - self.yaw);
            let right = wrap(centre + CAM_HUG_ANGLE - self.yaw);
            let delta = if left.abs() <= right.abs() { left } else { right };
            self.yaw = wrap(self.yaw + delta * blend);
        }
    }

    /// (eye, focus) in the world frame for a player at `pos`.
    pub fn eye(&self, pos: [f32; 3]) -> ([f32; 3], [f32; 3]) {
        let f = [pos[0], pos[1] + CAM_FOCUS_HEIGHT, pos[2]];
        let (cp, sp) = (self.pitch.cos(), self.pitch.sin());
        let dist = CAM_DIST + (CAM_HUG_DIST - CAM_DIST) * self.pull;
        let e = [f[0] - self.yaw.sin() * cp * dist, f[1] + sp * dist, f[2] - self.yaw.cos() * cp * dist];
        (e, f)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
        assert!(eye[0].abs() > 0.5, "view is beside the wall");
        let yaw = b.yaw;
        for _ in 0..120 { b.follow_hug(None, DT); }
        assert!(b.pull < 0.001);
        assert_eq!(b.yaw, yaw, "release preserves manual orbit");
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
            c.follow_lock([0.0; 3], [5.0, 0.0, 0.0]);
        }
        assert!((c.yaw - std::f32::consts::FRAC_PI_2).abs() < 1e-2, "looks along +x at the target");
    }
}
