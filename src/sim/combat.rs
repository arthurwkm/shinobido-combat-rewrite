//! Hits both ways between the player and one enemy. Reach, cone and damage values are ESTIMATE (data.rs).

mod carry;
pub mod items;
use super::{data, enemy::{Enemy, State as EnemyState}, player::{hit_damage, Player, State}};

pub struct Fight {
    pub player: Player,
    pub enemy: Enemy,
    pub player_hp: i32,
    /// One hit per swing: set when the current swing has landed, cleared when it ends.
    player_landed: bool,
    enemy_landed: bool,
    /// Roll into the enemy: extra metres per frame along the roll heading on top of FRONT01's root motion.
    roll: Option<[f32; 2]>,
    pub carry: carry::Carry,
    pub items: items::Items,
}

/// True when `to` lies inside the frontal cone of something at `from` facing `heading`, within reach.
fn in_front(from: [f32; 3], heading: f32, to: [f32; 3]) -> bool {
    let (dx, dz) = (to[0] - from[0], to[2] - from[2]);
    if dx.hypot(dz) > data::MELEE_REACH { return false; }
    let d = (dx.atan2(dz) - heading + std::f32::consts::PI).rem_euclid(std::f32::consts::TAU) - std::f32::consts::PI;
    d.abs() <= data::FRONT_CONE
}

/// The soldier's reaction clip: a heavy blow throws him back (HI_F07, then the get-up from his back), a hit from
/// behind staggers him (HI_UC07), a light frontal hit flinches (HI_BC07). ESTIMATE: the B/U/C letters are not decoded.
pub fn soldier_reaction(behind: bool, dmg: i32) -> &'static str {
    if dmg >= data::HEAVY_HIT { "HI_F07" } else if behind { "HI_UC07" } else { "HI_BC07" }
}

/// True when the offset (dx, dz) lies within `cone` radians of `heading`.
fn in_cone(heading: f32, dx: f32, dz: f32, cone: f32) -> bool {
    let d = (dx.atan2(dz) - heading + std::f32::consts::PI).rem_euclid(std::f32::consts::TAU) - std::f32::consts::PI;
    d.abs() <= cone
}

impl Fight {
    pub fn new(player: Player, enemy: Enemy) -> Self {
        Fight { player, enemy, player_hp: data::PLAYER_HP, player_landed: false, enemy_landed: false, roll: None, carry: Default::default(), items: Default::default() }
    }

    /// Advance both one frame, then resolve hits.
    pub fn step(&mut self, forward: bool, attack: bool) {
        self.player.step(forward, attack);
        self.step_enemy();
    }

    /// The enemy half of a frame, for callers that drive the player themselves (the viewer's `Input::apply`).
    pub fn step_enemy(&mut self) {
        self.caltrops();
        self.enemy.step(self.player.pos);
        self.hold();
        self.rolling();
        self.separate();
        self.resolve();
    }

    /// Push the two apart (half each) until their bodies no longer overlap; execution pairs stay put.
    fn separate(&mut self) {
        let (p, e) = (&mut self.player, &mut self.enemy);
        if p.state == State::Special || self.roll.is_some() || self.carry.load == Some(carry::Load::Body) {
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

    /// R1 + Forward + Cross: roll (FRONT01) through the soldier ahead and come up behind him, facing his back.
    pub fn roll(&mut self) -> bool {
        let (p, e) = (&mut self.player, &self.enemy);
        let (dx, dz) = (e.pos[0] - p.pos[0], e.pos[2] - p.pos[2]);
        let dist = dx.hypot(dz);
        if e.state == EnemyState::Dead || dist > data::ROLL_REACH || dist < 1e-3 || !in_cone(p.heading, dx, dz, data::ROLL_CONE)
            || !p.start_dodge("FRONT01") {
            return false;
        }
        // FRONT01 carries 1.5 m forward; the rest is spread over its frames to land ROLL_BEHIND past him.
        let extra = (dist + data::ROLL_BEHIND - 1.5) / p.world_len() as f32;
        p.heading = dx.atan2(dz);
        self.roll = Some([dx / dist * extra, dz / dist * extra]);
        true
    }

    fn rolling(&mut self) {
        let Some(step) = self.roll else { return };
        let p = &mut self.player;
        if p.state != State::Dodge {
            self.roll = None;
            p.heading += std::f32::consts::PI; // turn round to his back
            p.target_heading = p.heading;
            return;
        }
        p.pos[0] += step[0];
        p.pos[2] += step[1];
    }

    /// Start the execution that `execution::available` offers: align the pair, play both clips, the soldier dies.
    pub fn execute(&mut self) -> bool {
        let Some(kind) = super::execution::available(&self.player, &self.enemy) else { return false };
        self.kill(kind);
        true
    }

    fn kill(&mut self, kind: super::execution::Kind) {
        let (pc, ec) = kind.clips();
        let (p, e) = (&mut self.player, &mut self.enemy);
        // ESTIMATE alignment: start the paired clips at the victim's height, including aerial kills.
        let (dx, dz) = (e.pos[0] - p.pos[0], e.pos[2] - p.pos[2]);
        p.heading = dx.atan2(dz);
        p.pos[1] = e.pos[1];
        let (s, c) = p.heading.sin_cos();
        e.pos = [p.pos[0] + s * data::EXEC_DIST, e.pos[1], p.pos[2] + c * data::EXEC_DIST];
        if kind == super::execution::Kind::Front {
            e.heading = p.heading + std::f32::consts::PI; // faces the player
        } else if kind == super::execution::Kind::Back {
            e.heading = p.heading; // victim's back to the player
        }
        p.start_special(pc);
        e.execute(ec);
    }

    /// Triangle: an available execution first, else mash while holding him, else grab the soldier in front.
    /// `throw` (R1+Triangle) throws him instead, held or not.
    pub fn grab(&mut self, throw: bool) -> bool {
        if throw {
            return self.throw();
        }
        if self.execute() {
            return true;
        }
        if self.enemy.state == EnemyState::Held {
            self.mash();
            return true;
        }
        let (p, e) = (&mut self.player, &mut self.enemy);
        let (dx, dz) = (e.pos[0] - p.pos[0], e.pos[2] - p.pos[2]);
        let near = dx.hypot(dz) <= data::GRAB_REACH && in_front(p.pos, p.heading, e.pos);
        if !near || !matches!(e.state, EnemyState::Idle | EnemyState::Patrol | EnemyState::Alert | EnemyState::Chase | EnemyState::Hit | EnemyState::Attack | EnemyState::Guard)
            || !p.request(State::Special, data::GRAB_CLIP) {
            return false;
        }
        e.grab();
        true
    }

    fn throw(&mut self) -> bool {
        let (p, e) = (&mut self.player, &mut self.enemy);
        let held = e.state == EnemyState::Held;
        if !(held || e.state != EnemyState::Dead && in_front(p.pos, p.heading, e.pos) && !matches!(e.state, EnemyState::Down | EnemyState::GetUp))
            || !(matches!(p.state, State::Guard | State::GuardEnd) && { p.start_special(data::THROW_CLIP); true } || p.request(State::Special, data::THROW_CLIP)) {
            return false;
        }
        let (s, c) = p.heading.sin_cos();
        e.pos = [p.pos[0] + s * data::EXEC_DIST, e.pos[1], p.pos[2] + c * data::EXEC_DIST];
        e.heading = p.heading + std::f32::consts::PI;
        e.throw(data::THROW_DAMAGE);
        true
    }

    /// A Square or Triangle press while he is held: enough of them force the from-behind kill.
    pub fn mash(&mut self) {
        let e = &mut self.enemy;
        if e.state != EnemyState::Held {
            return;
        }
        e.mashes += 1;
        let easy = e.hp as f32 <= data::EXEC_FRONT_HP * data::ENEMY_HP as f32;
        if e.mashes >= if easy { data::GRAB_MASH / 2 } else { data::GRAB_MASH } {
            self.kill(super::execution::Kind::Back);
        }
    }

    /// Keep a held soldier in front of the player, and let him break free after `GRAB_HOLD` frames or when the
    /// player walks off.
    fn hold(&mut self) {
        let (p, e) = (&self.player, &mut self.enemy);
        if e.state != EnemyState::Held {
            return;
        }
        let far = (e.pos[0] - p.pos[0]).hypot(e.pos[2] - p.pos[2]) > 2.0 * data::GRAB_REACH;
        if e.held_for > data::GRAB_HOLD || far || p.state == State::Hit {
            e.release();
            return;
        }
        let (s, c) = p.heading.sin_cos();
        e.pos = [p.pos[0] + s * data::EXEC_DIST, p.pos[1], p.pos[2] + c * data::EXEC_DIST];
        e.heading = p.heading + std::f32::consts::PI;
    }

    fn resolve(&mut self) {
        let (p, e) = (&mut self.player, &mut self.enemy);
        if p.attack_active() {
            if !self.player_landed && in_front(p.pos, p.heading, e.pos) {
                self.player_landed = true;
                let dmg = hit_damage(p.attack_arg().unwrap_or(0), data::PLAYER_POWER, 0.0);
                let behind = !in_cone(e.heading, p.pos[0] - e.pos[0], p.pos[2] - e.pos[2], std::f32::consts::FRAC_PI_2);
                e.hit(soldier_reaction(behind, dmg), dmg);
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
