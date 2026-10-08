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
        self.enemy.step(self.player.pos);
        self.resolve();
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
                    p.hit("HI_BS01");
                }
            }
        } else {
            self.enemy_landed = false;
        }
    }
}
