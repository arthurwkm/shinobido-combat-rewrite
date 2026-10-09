//! Picking up, carrying, putting down and throwing crates and the dead soldier. Timings and distances are ESTIMATE (data.rs).

use super::{data, in_cone, EnemyState, Fight};
use crate::sim::terrain::Terrain;

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Load {
    /// Index into `Terrain::colliders` (listed in `Terrain::crates`).
    Crate(usize),
    /// The dead soldier at this index of `Fight::enemies()`.
    Body(usize),
}

#[derive(Clone, Copy, PartialEq)]
enum Release {
    Put,
    Throw,
}

#[derive(Default)]
pub struct Carry {
    pub load: Option<Load>,
    /// Bottom centre of the load while it is in the arms or in the air.
    pub at: [f32; 3],
    release: Option<Release>,
    /// Position and velocity (m, m/s) of a thrown load.
    flight: Option<[[f32; 3]; 2]>,
}

impl Fight {
    /// Triangle with nothing else to grab: lift the nearest crate in front, or the dead soldier.
    pub fn pick(&mut self, t: &mut Terrain) -> bool {
        let p = &self.player;
        if self.carry.load.is_some() {
            return false;
        }
        let near = |c: [f32; 3], reach: f32| {
            let (dx, dz) = (c[0] - p.pos[0], c[2] - p.pos[2]);
            dx.hypot(dz) <= reach && in_cone(p.heading, dx, dz, data::PICK_CONE)
        };
        // A crate with another crate resting on it stays put.
        let covered = |i: usize| t.crates.iter().any(|&j| j != i && (t.crate_base(j)[1] - t.colliders[i].max[1]).abs() < 0.05
            && (t.crate_base(j)[0] - t.crate_base(i)[0]).abs() < 0.5 && (t.crate_base(j)[2] - t.crate_base(i)[2]).abs() < 0.5);
        let dist = |i: usize| (t.crate_base(i)[0] - p.pos[0]).hypot(t.crate_base(i)[2] - p.pos[2]);
        let crate_ = t.crates.iter().copied()
            .filter(|&i| (t.crate_base(i)[1] - p.pos[1]).abs() < 0.3 && near(t.crate_base(i), data::PICK_REACH) && !covered(i))
            .min_by(|&a, &b| dist(a).total_cmp(&dist(b)));
        let load = match crate_ {
            Some(i) => Load::Crate(i),
            None => match self.enemies().position(|e| e.state == EnemyState::Dead && near(e.pos, data::PICK_REACH)) {
                Some(i) => Load::Body(i),
                None => return false,
            },
        };
        if !self.player.begin_pick() {
            return false;
        }
        if let Load::Crate(i) = load {
            t.place_crate(i, [0.0, -5.0, 0.0]); // parked under the floor while it is in the arms
        }
        self.carry = Carry { load: Some(load), ..Default::default() };
        true
    }

    /// Triangle while carrying puts the load down in front; `throw` (Square, or R1+Triangle) throws it.
    pub fn release_load(&mut self, throw: bool) -> bool {
        let (clip, release) = if throw { ("NAGE_K05", Release::Throw) } else { ("PUT_K05", Release::Put) };
        if self.carry.load.is_none() || self.carry.release.is_some() || !self.player.unload(clip) {
            return false;
        }
        self.carry.release = Some(release);
        true
    }

    /// One frame of the load: in the arms, put down, thrown through the air, or dropped when he was hit.
    pub fn carry_step(&mut self, t: &mut Terrain) {
        let Some(load) = self.carry.load else { return };
        if let Some(f) = self.carry.flight {
            return self.fly(t, load, f);
        }
        let p = &self.player;
        let (s, c) = p.heading.sin_cos();
        let (pos, action, frame) = (p.pos, p.world_action(), p.world_frame());
        let held = p.carrying || self.carry.release.is_some();
        if !held {
            return self.land(t, load, [pos[0] + s * data::HOLD_FWD, pos[1] + 0.3, pos[2] + c * data::HOLD_FWD]);
        }
        self.carry.at = [pos[0] + s * data::HOLD_FWD, pos[1] + data::HOLD_UP, pos[2] + c * data::HOLD_FWD];
        self.follow(load);
        let Some(release) = self.carry.release else { return };
        let (clip, release_at) = if release == Release::Put { ("PUT_K05", data::PUT_FRAME) } else { ("NAGE_K05", data::THROW_FRAME) };
        if action != clip {
            self.carry.release = None; // interrupted
            return self.land(t, load, [pos[0] + s * data::HOLD_FWD, pos[1] + 0.3, pos[2] + c * data::HOLD_FWD]);
        }
        if frame < release_at {
            return;
        }
        self.carry.release = None;
        if release == Release::Put {
            self.land(t, load, [pos[0] + s * data::PUT_DIST, pos[1] + 0.3, pos[2] + c * data::PUT_DIST]);
        } else {
            self.carry.flight = Some([self.carry.at, [s * data::THROW_SPEED, data::THROW_LIFT, c * data::THROW_SPEED]]);
        }
    }

    fn follow(&mut self, load: Load) {
        if let Load::Body(i) = load {
            let (at, heading) = (self.carry.at, self.player.heading);
            self.with_target(i, |f| { f.enemy.pos = at; f.enemy.heading = heading; });
        }
    }

    fn fly(&mut self, t: &mut Terrain, load: Load, [mut pos, mut vel]: [[f32; 3]; 2]) {
        let next = [pos[0] + vel[0] * crate::sim::DT, pos[1] + vel[1] * crate::sim::DT, pos[2] + vel[2] * crate::sim::DT];
        if t.blocked([next[0], next[1] + 0.5, next[2]]) {
            vel[0] = 0.0; // hit a wall: drop straight down
            vel[2] = 0.0;
        } else {
            pos = next;
        }
        vel[1] -= data::GRAVITY_MS2 * crate::sim::DT;
        let mut struck = false;
        if !matches!(load, Load::Body(_)) {
            self.for_each(|f, _| {
                let e = &mut f.enemy;
                if !struck && e.state != EnemyState::Dead && (e.pos[0] - pos[0]).hypot(e.pos[2] - pos[2]) <= data::THROWN_HIT_RADIUS
                    && pos[1] > e.pos[1] - 1.0 && pos[1] < e.pos[1] + 1.8 {
                    e.hit("HI_BS07", data::THROWN_DAMAGE);
                    struck = true;
                }
            });
        }
        if struck {
            vel[0] = 0.0;
            vel[2] = 0.0;
        }
        self.carry.at = pos;
        self.follow(load);
        if vel[1] < 0.0 && pos[1] <= t.ground_height([pos[0], pos[1] + 0.3, pos[2]]) {
            self.carry.flight = None;
            return self.land(t, load, pos);
        }
        self.carry.flight = Some([pos, vel]);
    }

    /// Rest the load on whatever lies under `at` (a probe a little above the feet, so box tops count).
    fn land(&mut self, t: &mut Terrain, load: Load, at: [f32; 3]) {
        let base = [at[0], t.ground_height(at), at[2]];
        match load {
            Load::Crate(i) => t.place_crate(i, base),
            Load::Body(i) => self.with_target(i, |f| f.enemy.pos = base),
        }
        self.carry = Carry::default();
    }

    /// Where the viewer draws crate collider `i`: in the arms or in the air while it is the load, else at rest.
    pub fn crate_base(&self, t: &Terrain, i: usize) -> [f32; 3] {
        if self.carry.load == Some(Load::Crate(i)) { self.carry.at } else { t.crate_base(i) }
    }
}
