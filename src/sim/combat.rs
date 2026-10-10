//! Hits both ways between the player and one enemy. Reach, cone and damage values are ESTIMATE (data.rs).

mod carry;
pub mod items;
use super::{data, enemy::{Enemy, State as EnemyState}, player::{hit_damage, Player, State}};

pub struct Fight {
    pub player: Player,
    /// The soldier at index 0 of `enemies()`; the rest are in `others`. Per-frame code visits each by swapping it in.
    pub enemy: Enemy,
    pub others: Vec<Enemy>,
    /// Index into `enemies()` that the HUD, the cue and the camera follow (the locked target, else the nearest).
    pub focus: usize,
    pub player_hp: i32,
    /// One hit per swing: set when the current swing has landed, cleared when it ends.
    player_landed: bool,
    /// Roll into the enemy: extra metres per frame along the roll heading on top of the roll's root motion.
    roll: Option<[f32; 2]>,
    pub carry: carry::Carry,
    pub items: items::Items,
    /// From-above kill in progress: the player is falling onto soldier `.0` (an `enemies()` index); the pair starts on landing.
    dive: Option<usize>,
}

/// True when `to` lies inside the frontal cone of something at `from` facing `heading`, within reach.
fn in_front(from: [f32; 3], heading: f32, to: [f32; 3]) -> bool {
    let (dx, dz) = (to[0] - from[0], to[2] - from[2]);
    if (to[1] - from[1]).abs() >= data::BODY_HEIGHT || dx.hypot(dz) > data::MELEE_REACH { return false; }
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
        Fight { player, enemy, others: Vec::new(), focus: 0, player_hp: data::PLAYER_HP, player_landed: false, roll: None, carry: Default::default(), items: Default::default(), dive: None }
    }

    /// Advance both one frame, then resolve hits.
    pub fn step(&mut self, forward: bool, attack: bool) {
        self.player.step(forward, attack);
        self.step_enemy();
    }

    /// The enemy half of a frame, for callers that drive the player themselves (the viewer's `Input::apply`).
    pub fn step_enemy(&mut self) {
        self.step_enemy_on(&super::terrain::Terrain::default());
    }

    /// Advance soldiers against the same terrain used by the player.
    pub fn step_enemy_on(&mut self, terrain: &super::terrain::Terrain) {
        self.rolling();
        self.land_dive();
        self.for_each(|f, i| {
            f.caltrops();
            if f.carry.load == Some(carry::Load::Body(i)) {
                f.enemy.step(f.player.pos);
            } else {
                f.enemy.step_on(terrain, f.player.pos);
            }
            f.hold();
            f.separate(terrain);
            f.resolve(terrain);
        });
    }

    /// Every soldier: the first, then `others`.
    pub fn enemies(&self) -> impl Iterator<Item = &Enemy> {
        std::iter::once(&self.enemy).chain(self.others.iter())
    }

    /// Run `f` with soldier `i` (an index into `enemies()`) swapped into `self.enemy`; out of range runs on soldier 0.
    pub fn with_target<R>(&mut self, i: usize, f: impl FnOnce(&mut Fight) -> R) -> R {
        let i = if i <= self.others.len() { i } else { 0 };
        if i > 0 { std::mem::swap(&mut self.enemy, &mut self.others[i - 1]); }
        let r = f(self);
        if i > 0 { std::mem::swap(&mut self.enemy, &mut self.others[i - 1]); }
        r
    }

    pub fn for_each(&mut self, mut f: impl FnMut(&mut Fight, usize)) {
        for i in 0..=self.others.len() {
            self.with_target(i, |s| f(s, i));
        }
    }

    /// The living soldier nearest the player (0 when none is alive).
    pub fn nearest(&self) -> usize {
        let d = |e: &Enemy| (e.pos[0] - self.player.pos[0]).hypot(e.pos[2] - self.player.pos[2]);
        self.enemies().enumerate().filter(|(_, e)| e.state != EnemyState::Dead)
            .min_by(|a, b| d(a.1).total_cmp(&d(b.1))).map_or(0, |(i, _)| i)
    }

    /// The soldier the player is holding, if any.
    pub fn held(&self) -> Option<usize> {
        self.enemies().position(|e| e.state == EnemyState::Held)
    }

    /// Step each living soldier's awareness; the strongest meter is returned for the HUD's eye icon.
    pub fn stealth_step(&mut self, t: &super::terrain::Terrain) -> f32 {
        let (pos, action) = (self.player.pos, self.player.world_action());
        let mut top = 0.0f32;
        for e in std::iter::once(&mut self.enemy).chain(self.others.iter_mut()) {
            if e.hp > 0 {
                let mut s = std::mem::take(&mut e.stealth);
                s.step(t, e, pos, action);
                e.stealth = s;
            }
            top = top.max(e.stealth.meter);
        }
        top
    }

    /// Push the two apart (half each) until their bodies no longer overlap; execution pairs stay put.
    fn separate(&mut self, terrain: &super::terrain::Terrain) {
        let (p, e) = (&mut self.player, &mut self.enemy);
        if e.state == EnemyState::Dead || (p.pos[1] - e.pos[1]).abs() >= data::BODY_HEIGHT
            || p.state == State::Special || self.roll.is_some() || matches!(self.carry.load, Some(carry::Load::Body(_))) {
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
        let (pp, ep) = (p.pos, e.pos);
        p.pos[0] -= nx * push;
        p.pos[2] -= nz * push;
        e.pos[0] += nx * push;
        e.pos[2] += nz * push;
        p.pos = terrain.move_body(pp, p.pos, (pp[1] - terrain.ground_height(pp)).abs() <= data::SUPPORT_EPS).0;
        e.pos = terrain.move_body(ep, e.pos, (ep[1] - terrain.ground_height(ep)).abs() <= data::SUPPORT_EPS).0;
    }

    /// R1 + Forward + Cross: roll (`data::ROLL_CLIP`) through the soldier ahead and come up behind him, facing his back.
    pub fn roll(&mut self) -> bool {
        let (p, e) = (&mut self.player, &self.enemy);
        let (dx, dz) = (e.pos[0] - p.pos[0], e.pos[2] - p.pos[2]);
        let dist = dx.hypot(dz);
        if e.state == EnemyState::Dead || (p.pos[1] - e.pos[1]).abs() >= data::BODY_HEIGHT || dist > data::ROLL_REACH || dist < 1e-3 || !in_cone(p.heading, dx, dz, data::ROLL_CONE)
            || !p.dodge() {
            return false;
        }
        // The roll's own root motion carries it forward; any shortfall is spread over its frames to land ROLL_BEHIND past him.
        let carried = super::extracted::ACTIONS.iter().find(|a| a.name == data::ROLL_CLIP).and_then(|a| a.root.last()).map_or(0.0, |r| r.1[2]);
        let extra = ((dist + data::ROLL_BEHIND - carried) / p.world_len() as f32).max(0.0);
        p.heading = dx.atan2(dz);
        self.roll = Some([dx / dist * extra, dz / dist * extra]);
        true
    }

    fn rolling(&mut self) {
        let Some(step) = self.roll else { return };
        let p = &mut self.player;
        if p.state != State::Dodge {
            self.roll = None; // the roll itself ends turned round, facing his back
            return;
        }
        p.pos[0] += step[0];
        p.pos[2] += step[1];
    }

    /// Start the execution that `execution::available` offers: align the pair, play both clips, the soldier dies.
    pub fn execute(&mut self) -> bool {
        let Some(kind) = super::execution::available(&self.player, &self.enemy) else { return false };
        if kind == super::execution::Kind::Above && self.dive.is_none() {
            self.dive = Some(self.focus);
            self.player.dive();
        } else {
            self.kill(kind);
        }
        true
    }

    /// Finish a from-above kill once the player has dropped to the soldier's height (the drop is the game's approach, clip 4-30 s).
    fn land_dive(&mut self) {
        let Some(i) = self.dive else { return };
        if !matches!(self.player.state, State::Fall | State::Land) {
            self.dive = None;
            return;
        }
        let (py, ey) = (self.player.pos[1], self.with_target(i, |f| f.enemy.pos[1]));
        if self.with_target(i, |f| f.enemy.state == EnemyState::Dead) {
            self.dive = None;
        } else if py - ey <= 0.05 {
            self.dive = None;
            self.with_target(i, |f| f.kill(super::execution::Kind::Above));
        }
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

    fn resolve(&mut self, terrain: &super::terrain::Terrain) {
        let (p, e) = (&mut self.player, &mut self.enemy);
        if p.attack_active() {
            if !self.player_landed && e.state != EnemyState::Dead && in_front(p.pos, p.heading, e.pos)
                && terrain.sight_clear(p.pos, e.pos) {
                self.player_landed = true;
                let dmg = hit_damage(p.attack_arg().unwrap_or(0), data::PLAYER_POWER, 0.0);
                let behind = !in_cone(e.heading, p.pos[0] - e.pos[0], p.pos[2] - e.pos[2], std::f32::consts::FRAC_PI_2);
                e.hit(soldier_reaction(behind, dmg), dmg);
            }
        } else {
            self.player_landed = false;
        }
        if e.attack_active() {
            if !e.landed && in_front(e.pos, e.heading, p.pos) && terrain.sight_clear(e.pos, p.pos) {
                e.landed = true;
                let guarding = matches!(p.state, State::Guard) && in_front(p.pos, p.heading, e.pos);
                if !guarding && self.player_hp > 0 {
                    self.player_hp = (self.player_hp - data::ENEMY_DAMAGE).max(0);
                    p.hit(data::PLAYER_HIT_CLIP);
                }
            }
        } else {
            e.landed = false;
        }
    }
}
