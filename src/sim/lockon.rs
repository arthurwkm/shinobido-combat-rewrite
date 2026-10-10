//! Lock-on: target select/switch/release and the facing/strafe it drives. Range and cone are ESTIMATE (data.rs).

use super::{data, player::Player};
use std::f32::consts::{PI, TAU};

fn wrap(a: f32) -> f32 {
    (a + PI).rem_euclid(TAU) - PI
}

fn bearing(from: [f32; 3], to: [f32; 3]) -> f32 {
    (to[0] - from[0]).atan2(to[2] - from[2])
}

#[derive(Default)]
pub struct LockOn {
    /// Index into the caller's target list.
    pub target: Option<usize>,
}

impl LockOn {
    /// Lock the target nearest to the facing direction inside range and cone; none leaves it unlocked.
    pub fn select(&mut self, from: [f32; 3], heading: f32, targets: &[[f32; 3]]) {
        self.target = targets
            .iter()
            .enumerate()
            .filter(|(_, t)| (t[0] - from[0]).hypot(t[2] - from[2]) <= data::LOCK_RANGE)
            .map(|(i, t)| (i, wrap(bearing(from, *t) - heading).abs()))
            .filter(|(_, d)| *d <= data::LOCK_CONE)
            .min_by(|a, b| a.1.total_cmp(&b.1))
            .map(|(i, _)| i);
    }

    /// Move the lock to the next target clockwise (`right`) or counter-clockwise by bearing; no-op without a lock.
    pub fn switch(&mut self, from: [f32; 3], targets: &[[f32; 3]], right: bool) {
        let Some(cur) = self.target.filter(|&i| i < targets.len()) else { return };
        let b0 = bearing(from, targets[cur]);
        let sign = if right { 1.0 } else { -1.0 };
        let next = (0..targets.len())
            .filter(|&i| i != cur && (targets[i][0] - from[0]).hypot(targets[i][2] - from[2]) <= data::LOCK_RANGE)
            .map(|i| (i, (sign * wrap(bearing(from, targets[i]) - b0)).rem_euclid(TAU)))
            .min_by(|a, b| a.1.total_cmp(&b.1));
        if let Some((i, _)) = next {
            self.target = Some(i);
        }
    }

    pub fn release(&mut self) {
        self.target = None;
    }

    /// Validate the retained target before feeding facing, camera or combat.
    pub fn position(&mut self, from: [f32; 3], targets: &[[f32; 3]]) -> Option<[f32; 3]> {
        let t = self.target.and_then(|i| targets.get(i)).copied()
            .filter(|t| t.iter().all(|x| x.is_finite()) && (t[0] - from[0]).hypot(t[2] - from[2]) <= data::LOCK_RANGE);
        if t.is_none() { self.release(); }
        t
    }

    /// Drop the lock when the target is gone or out of range, else face it and strafe by `input`
    /// (x right, z forward, relative to the facing; zero stands still). Returns the `forward` flag for `Player::step`.
    pub fn drive(&mut self, p: &mut Player, targets: &[[f32; 3]], input: [f32; 2]) -> bool {
        let Some(t) = self.position(p.pos, targets) else { return false };
        p.target_heading = bearing(p.pos, t);
        if input == [0.0; 2] {
            return false;
        }
        p.walk = true;
        p.walk_clip = if input[1].abs() >= input[0].abs() {
            if input[1] > 0.0 { "WALK_F02" } else { "WALK_B02" }
        } else if input[0] > 0.0 {
            "WALK_R02"
        } else {
            "WALK_L02"
        };
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn target_position_releases_dead_distant_and_missing_targets() {
        let mut lock = LockOn { target: Some(0) };
        for targets in [vec![[f32::INFINITY; 3]], vec![[0.0, 0.0, data::LOCK_RANGE + 1.0]], vec![]] {
            lock.target = Some(0);
            let mut p = Player::new();
            assert!(!lock.drive(&mut p, &targets, [0.0; 2]));
            assert_eq!(lock.target, None);
        }
        lock.target = Some(0);
        let mut p = Player::new();
        assert!(!lock.drive(&mut p, &[[f32::NAN; 3]], [0.0; 2]));
        assert_eq!(lock.target, None);
        assert!(p.target_heading.is_finite());
    }
}
