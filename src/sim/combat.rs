//! Hits both ways between the player and one enemy. Reach, cone and damage values are ESTIMATE (data.rs).

use super::{data, enemy::Enemy, player::{hit_damage, Player, State}};

pub struct Fight {
    pub player: Player,
    pub enemy: Enemy,
    pub player_hp: i32,
    /// One hit per swing: set when the current swing has landed, cleared when it ends.
    player_landed: bool,
    enemy_landed: bool,
}

/// True when `to` lies inside the frontal cone of something at `from` facing `heading`, within reach.
fn in_front(from: [f32; 3], heading: f32, to: [f32; 3]) -> bool {
    let (dx, dz) = (to[0] - from[0], to[2] - from[2]);
    if dx.hypot(dz) > data::MELEE_REACH { return false; }
    let d = (dx.atan2(dz) - heading + std::f32::consts::PI).rem_euclid(std::f32::consts::TAU) - std::f32::consts::PI;
    d.abs() <= data::FRONT_CONE
}

impl Fight {
    pub fn new(player: Player, enemy: Enemy) -> Self {
        Fight { player, enemy, player_hp: data::PLAYER_HP, player_landed: false, enemy_landed: false }
    }

    /// Advance both one frame, then resolve hits.
    pub fn step(&mut self, forward: bool, attack: bool) {
        self.player.step(forward, attack);
        self.step_enemy();
    }

    /// The enemy half of a frame, for callers that drive the player themselves (the viewer's `Input::apply`).
    pub fn step_enemy(&mut self) {
        self.enemy.step(self.player.pos);
        self.separate();
        self.resolve();
    }

    /// Push the two apart (half each) until their bodies no longer overlap; execution pairs stay put.
    fn separate(&mut self) {
        let (p, e) = (&mut self.player, &mut self.enemy);
        if p.state == State::Special {
            return;
        }
        let (dx, dz) = (e.pos[0] - p.pos[0], e.pos[2] - p.pos[2]);
        let d = dx.hypot(dz);
        let min = 2.0 * data::BODY_RADIUS;
        if d >= min {
            return;
        }
        let (nx, nz) = if d > 1e-4 { (dx / d, dz / d) } else { (0.0, 1.0) };
        let push = (min - d) / 2.0;
        p.pos[0] -= nx * push;
        p.pos[2] -= nz * push;
        e.pos[0] += nx * push;
        e.pos[2] += nz * push;
    }

    /// Start the execution that `execution::available` offers: align the pair, play both clips, the soldier dies.
    pub fn execute(&mut self) -> bool {
        let Some(kind) = super::execution::available(&self.player, &self.enemy) else { return false };
        let (pc, ec) = kind.clips();
        let (p, e) = (&mut self.player, &mut self.enemy);
        if p.state == State::Special {
            return false;
        }
        // Face the victim; the victim is put EXEC_DIST ahead of the player (above kills keep their height).
        let (dx, dz) = (e.pos[0] - p.pos[0], e.pos[2] - p.pos[2]);
        p.heading = dx.atan2(dz);
        let (s, c) = p.heading.sin_cos();
        e.pos = [p.pos[0] + s * data::EXEC_DIST, if kind == super::execution::Kind::Above { p.pos[1] } else { e.pos[1] }, p.pos[2] + c * data::EXEC_DIST];
        if kind == super::execution::Kind::Front {
            e.heading = p.heading + std::f32::consts::PI; // faces the player
        } else if kind == super::execution::Kind::Back {
            e.heading = p.heading; // victim's back to the player
        }
        p.special(pc);
        e.execute(ec);
        true
    }

    fn resolve(&mut self) {
        let (p, e) = (&mut self.player, &mut self.enemy);
        if p.attack_active() {
            if !self.player_landed && in_front(p.pos, p.heading, e.pos) {
                self.player_landed = true;
                let dmg = hit_damage(p.attack_arg().unwrap_or(0), data::PLAYER_POWER, 0.0);
                e.hit("HI_BS07", dmg);
            }
        } else {
            self.player_landed = false;
        }
        if e.attack_active() {
            if !self.enemy_landed && in_front(e.pos, e.heading, p.pos) {
                self.enemy_landed = true;
                let guarding = matches!(p.state, State::Guard) && in_front(p.pos, p.heading, e.pos);
                if !guarding && self.player_hp > 0 {
                    self.player_hp = (self.player_hp - data::ENEMY_DAMAGE).max(0);
                    p.hit(data::PLAYER_HIT_CLIP);
                }
            }
        } else {
            self.enemy_landed = false;
        }
    }
}
