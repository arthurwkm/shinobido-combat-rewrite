//! Device-independent pad state (PS2 layout, docs/controls.md) turned into `Player` calls. The viewer fills it from
//! gamepad or keyboard; edge flags (`attack`, `jump`, `item`) are set by any render frame and consumed by one tick.

use super::data::{STICK_DEAD, STICK_WALK};
use super::player::{AttackMod, Player};
use super::terrain::Terrain;

#[derive(Default, Clone, Copy, Debug)]
pub struct Input {
    /// Left stick or WASD: x right, y forward, camera-relative, length <= 1.
    pub stick: [f32; 2],
    /// Edges: pressed since the last tick.
    pub attack: bool,
    pub jump: bool,
    pub item: bool,
    /// Edge: select the next item (the viewer's tick takes `item` and this before `apply`, `Fight::use_item`).
    pub item_next: bool,
    /// Held: lock-on (R1) guards when idle; crouch (L1) with `jump` dodges (ESTIMATE combinations, docs/controls.md).
    pub lock: bool,
    pub crouch: bool,
    /// Held: dash (Circle); runs with RUN_D05 (set 05 `_D`, ~5.5 m/s against RUN01 3.4).
    pub dash: bool,
    /// Edge: move the lock to the next target (-1 left, 1 right); taken by the viewer's tick.
    pub switch: i8,
    /// Set by the viewer each tick while locked on: face this point and strafe the stick around it.
    pub lock_target: Option<[f32; 3]>,
}

impl Input {
    /// Feed one 60 Hz tick (camera looking along `cam_yaw`), then clear the edges.
    pub fn apply(&mut self, p: &mut Player, terrain: &Terrain, cam_yaw: f32) {
        let [x, y] = self.stick;
        let mag = x.hypot(y);
        let moving = mag >= STICK_DEAD;
        let (s, c) = cam_yaw.sin_cos();
        let mut strafe = false;
        if let (Some(t), false) = (self.lock_target, self.dash) {
            // the lock camera looks at the target, so the stick is relative to the bearing
            let b = (t[0] - p.pos[0]).atan2(t[2] - p.pos[2]);
            p.target_heading = b;
            if moving {
                p.walk_clip = if y.abs() >= x.abs() { if y > 0.0 { "WALK_F02" } else { "WALK_B02" } } else if x > 0.0 { "WALK_R02" } else { "WALK_L02" };
                strafe = true;
            }
        } else if moving {
            // right of a camera looking along +z is -x
            p.target_heading = (s * y - c * x).atan2(c * y + s * x);
        }
        p.walk = strafe || mag < STICK_WALK;
        p.guard_held = self.lock && !strafe; // strafing is the lock-on walk; standing still guards
        p.crouch_held = self.crouch;
        p.run_clip = if self.dash { "RUN_D05" } else { "RUN01" };
        // ESTIMATE: Forward/Back are the stick within 60 degrees of / opposite to the current heading.
        let rel = (p.target_heading - p.heading + std::f32::consts::PI).rem_euclid(std::f32::consts::TAU) - std::f32::consts::PI;
        p.attack_mod = match () {
            // R1 is hide/crouch: R1 + Square is the rising uppercut, R1 + Back + Square the low kick (attacks video).
            _ if moving && rel.abs() > 2.0 && self.crouch => AttackMod::CrouchBack,
            _ if moving && rel.abs() > 2.0 => AttackMod::Back,
            _ if self.crouch => AttackMod::Crouch,
            _ if self.dash && moving => AttackMod::Dash,
            _ if moving && rel.abs() < 1.05 => AttackMod::Forward,
            _ => AttackMod::Plain,
        };
        if self.jump && self.crouch {
            let dir = if !moving || y >= x.abs() { "FRONT01" } else if y <= -x.abs() { "BACK01" } else if x > 0.0 { "RIGHT01" } else { "LEFT01" };
            p.dodge(dir);
        } else if self.jump {
            p.jump();
        }
        p.step_on(terrain, moving, self.attack);
        self.attack = false;
        self.jump = false;
        self.item = false;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sim::player::State;

    #[test]
    fn stick_is_camera_relative_and_edges_fire_once() {
        let (mut p, t, mut i) = (Player::new(), Terrain::default(), Input::default());
        i.stick = [0.0, 1.0];
        i.apply(&mut p, &t, 1.0);
        assert!((p.target_heading - 1.0).abs() < 1e-5, "forward follows the camera");
        i.stick = [1.0, 0.0];
        i.apply(&mut p, &t, 0.0);
        assert!((p.target_heading + std::f32::consts::FRAC_PI_2).abs() < 1e-5, "screen-right is -x");
        assert!(!p.walk);
        i.stick = [0.0, 0.3];
        i.apply(&mut p, &t, 0.0);
        assert!(p.walk, "light deflection walks");
        let mut q = Player::new();
        i = Input { jump: true, ..Default::default() };
        i.apply(&mut q, &t, 0.0);
        assert_eq!(q.state, State::Jump);
        assert!(!i.jump, "edge consumed");
        let mut d = Player::new();
        Input { jump: true, crouch: true, stick: [0.0, -1.0], ..Default::default() }.apply(&mut d, &t, 0.0);
        assert_eq!(d.world_action(), "BACK01");
    }
}

#[cfg(test)]
mod lock_tests {
    use super::*;

    #[test]
    fn locked_stick_strafes_around_the_target() {
        let (mut p, t) = (Player::new(), Terrain::default());
        let mut i = Input { lock: true, lock_target: Some([0.0, 0.0, 5.0]), stick: [1.0, 0.0], ..Default::default() };
        for _ in 0..30 {
            i.apply(&mut p, &t, 0.0);
        }
        assert_eq!(p.world_action(), "WALK_R02");
        assert!(p.heading.abs() < 0.3, "keeps facing the target");
        assert!(p.pos[0] < -0.5, "x {}", p.pos[0]); // WALK_R02 root runs -x at heading 0
    }
}

#[cfg(test)]
mod lock_slope_tests {
    use super::*;

    #[test]
    fn strafe_moves_on_the_sandbox_slope() {
        let (mut p, t) = (Player::new(), Terrain::sandbox());
        p.pos = [0.0, t.ground_height([0.0, 5.0, -6.0]), -6.0];
        let mut i = Input { lock: true, lock_target: Some([0.0, 0.0, 0.0]), stick: [1.0, 0.0], ..Default::default() };
        for _ in 0..60 {
            i.apply(&mut p, &t, 0.0);
        }
        assert!(p.pos[0] < -0.5, "x {} y {}", p.pos[0], p.pos[1]);
    }
}
