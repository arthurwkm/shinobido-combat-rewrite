//! Orbit follow camera (engine-free). `yaw` is the direction the camera looks along, like `Player::heading`
//! (forward = (sin yaw, cos yaw)); parameters are ESTIMATE (data.rs).

use super::data::*;
use super::DT;

pub struct Camera {
    pub yaw: f32,
    pub pitch: f32,
}

impl Default for Camera {
    fn default() -> Self {
        Camera { yaw: 0.0, pitch: CAM_PITCH }
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

    /// (eye, focus) in the world frame for a player at `pos`.
    pub fn eye(&self, pos: [f32; 3]) -> ([f32; 3], [f32; 3]) {
        let f = [pos[0], pos[1] + CAM_FOCUS_HEIGHT, pos[2]];
        let (cp, sp) = (self.pitch.cos(), self.pitch.sin());
        let e = [f[0] - self.yaw.sin() * cp * CAM_DIST, f[1] + sp * CAM_DIST, f[2] - self.yaw.cos() * cp * CAM_DIST];
        (e, f)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
            c.follow_lock([0.0; 3], [5.0, 0.0, 0.0]);
        }
        assert!((c.yaw - std::f32::consts::FRAC_PI_2).abs() < 1e-2, "looks along +x at the target");
    }
}
