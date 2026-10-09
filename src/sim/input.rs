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
    /// Held: lock-on (R1) guards when idle; crouch (L1) with `jump` dodges (ESTIMATE combinations, docs/controls.md).
    pub lock: bool,
    pub crouch: bool,
    /// Held: dash (Circle); runs with RUN_D05 (set 05 `_D`, ~5.5 m/s against RUN01 3.4).
    pub dash: bool,
}

impl Input {
    /// Feed one 60 Hz tick (camera looking along `cam_yaw`), then clear the edges.
    pub fn apply(&mut self, p: &mut Player, terrain: &Terrain, cam_yaw: f32) {
        let [x, y] = self.stick;
        let mag = x.hypot(y);
        let moving = mag >= STICK_DEAD;
        let (s, c) = cam_yaw.sin_cos();
        if moving {
            // right of a camera looking along +z is -x
            p.target_heading = (s * y - c * x).atan2(c * y + s * x);
        }
        p.walk = mag < STICK_WALK;
        p.guard_held = self.lock;
        p.crouch_held = self.crouch;
        p.run_clip = if self.dash { "RUN_D05" } else { "RUN01" };
        // ESTIMATE: Forward/Back are the stick within 60 degrees of / opposite to the current heading.
        let rel = (p.target_heading - p.heading + std::f32::consts::PI).rem_euclid(std::f32::consts::TAU) - std::f32::consts::PI;
        p.attack_mod = match () {
            _ if moving && rel.abs() > 2.0 => AttackMod::Back,
            _ if self.lock => AttackMod::Lock,
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
        if self.item {
            p.special("DRINK01");
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
