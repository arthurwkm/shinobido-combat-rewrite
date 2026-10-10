//! The foot soldier (I_BUSHI, motion set 07, docs/enemy.md) as an engine-free state machine.
//! Transitions are ESTIMATE: the AI files give attack choice tables, not these states.

use super::{data, stealth::Stealth, World, DT};

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum State {
    /// STOP07 loop.
    Idle,
    /// WALK_A07 loop.
    Patrol,
    /// SURP07 once, then Chase.
    Alert,
    /// RUN07 loop toward the target.
    Chase,
    /// One of `ENEMY_ATTACK_CLIPS` once, then Chase.
    Attack,
    /// GU_A07 then GU_B07 while held.
    Guard,
    /// A HI_*07 reaction, then Chase.
    Hit,
    /// DIE_A07, stays on its last frame.
    Dead,
    /// Grabbed by the player: GU_B07 struggle; `Fight` keeps him in front of the player and counts the mashing.
    Held,
    /// Thrown: DROP07 once, then GetUp.
    Down,
    /// UP_FR07 once, then Chase.
    GetUp,
    /// WALK_A07 toward `lead` (a noise or a glimpse), then Search.
    Investigate,
    /// SEARCH07 once at the spot, then Return.
    Search,
    /// WALK_A07 back to the post, then Patrol or Idle.
    Return,
    /// Ran off a ledge while chasing: FALL07 held with the run's momentum until he lands.
    Fall,
    /// LAND07 once, then Chase.
    Land,
}

pub struct Enemy {
    world: World,
    pub state: State,
    pub hp: i32,
    pub guard_held: bool,
    /// World position (m); the root motion moves it along `heading`.
    pub pos: [f32; 3],
    /// Falling velocity in game units/s, using the shared actor gravity update.
    vy: f32,
    /// Horizontal metres per frame carried through a `Fall` (the last grounded frame's run, as the player's FALL01).
    air: [f32; 2],
    /// Yaw in radians; forward is (sin h, cos h) in x/z.
    pub heading: f32,
    /// Frames spent in `Held` and presses made meanwhile.
    pub held_for: u32,
    pub mashes: u32,
    /// Root-motion speed factor (1 normally, less in caltrops).
    pub slow: f32,
    /// Where the soldier is heading when investigating (last place the player was noticed).
    pub lead: [f32; 3],
    /// Patrol legs; `route[0]` is the post he returns to. `patrols` is set by `patrol()`.
    route: [[f32; 3]; 2],
    leg: usize,
    patrols: bool,
    /// Waypoint round the box that stopped him (`Terrain::detour`), steered to before his goal; cleared on arrival.
    detour: Option<[f32; 3]>,
    /// Frames the target has been out of `LOSE_RANGE` while chasing.
    lost: u32,
    /// Frames of guard left to hold before dropping the block on his own.
    guard_left: u32,
    swings: usize,
    /// The current swing has already hit the player (one hit per swing); cleared when it ends.
    pub landed: bool,
    /// What he has noticed of the player: his own meter, so several soldiers each have their own awareness.
    pub stealth: Stealth,
}

impl Enemy {
    pub fn new(pos: [f32; 3]) -> Self {
        Enemy { world: World::new("STOP07"), state: State::Idle, hp: data::ENEMY_HP, guard_held: false, pos, vy: 0.0, air: [0.0; 2], heading: 0.0, held_for: 0, mashes: 0, slow: 1.0, lead: pos, route: [pos; 2], leg: 1, patrols: false, detour: None, lost: 0, guard_left: 0, swings: 0, landed: false, stealth: Stealth::default() }
    }

    pub fn action(&self) -> &'static str {
        self.world.action.name
    }

    pub fn frame(&self) -> u32 {
        self.world.frame
    }

    /// True while the swing clip's hit window is open (event types 1/2).
    pub fn attack_active(&self) -> bool {
        self.state == State::Attack && self.world.action.hits.iter().any(|h| (h.0..h.1).contains(&self.world.frame))
    }

    /// Walk to and fro between the post and a point `PATROL_LEN` ahead.
    pub fn patrol(&mut self) {
        if self.state == State::Idle {
            let (s, c) = self.heading.sin_cos();
            self.route = [self.pos, [self.pos[0] + s * data::PATROL_LEN, self.pos[1], self.pos[2] + c * data::PATROL_LEN]];
            self.leg = 1;
            self.patrols = true;
            self.start(State::Patrol, "WALK_A07");
        }
    }

    /// Something suspicious at `at` (stealth meter rising): walk over and look.
    pub fn suspect(&mut self, at: [f32; 3]) {
        if matches!(self.state, State::Idle | State::Patrol) {
            self.lead = at;
            self.start(State::Investigate, "WALK_A07");
        }
    }

    /// The soldier notices the player (stealth will drive this).
    pub fn alert(&mut self) {
        if matches!(self.state, State::Idle | State::Patrol | State::Investigate | State::Search | State::Return) {
            self.start(State::Alert, "SURP07");
        }
    }

    /// Take `damage` and play `reaction` (`HI_BS07`, ...); at 0 hp play DIE_A07. Guarding takes no damage.
    pub fn hit(&mut self, reaction: &str, damage: i32) {
        if matches!(self.state, State::Dead) {
            return;
        }
        if matches!(self.state, State::Guard | State::Held | State::Down | State::GetUp) {
            return;
        }
        self.hp -= damage;
        if self.hp <= 0 {
            self.hp = 0;
            self.start(State::Dead, "DIE_A07");
        } else {
            self.start(State::Hit, reaction);
        }
    }

    /// Killed by an execution: play the victim clip `reaction` (`KI_HIBS07`, ...) and stay dead.
    pub fn execute(&mut self, reaction: &str) {
        self.hp = 0;
        self.start(State::Dead, reaction);
    }

    pub fn grab(&mut self) {
        self.held_for = 0;
        self.mashes = 0;
        self.start(State::Held, "GU_B07");
    }

    /// Thrown down by the player: `damage`, DROP07, then the get-up.
    pub fn throw(&mut self, damage: i32) {
        self.hp = (self.hp - damage).max(1);
        self.start(State::Down, "DROP07");
    }

    pub fn release(&mut self) {
        self.start(State::Chase, "RUN07");
    }

    fn start(&mut self, state: State, clip: &str) {
        self.state = state;
        self.world = World::new(clip);
        self.detour = None;
    }

    /// Where the current state walks to; `target` is the player's position.
    fn goal(&self, target: [f32; 3]) -> Option<[f32; 3]> {
        match self.state {
            State::Chase | State::Alert => Some(target),
            State::Investigate => Some(self.lead),
            State::Return => Some(self.route[0]),
            State::Patrol => Some(self.route[self.leg]),
            _ => None,
        }
    }

    /// Root motion with wall collision, ramp support and gravity; held pairs own their placement.
    pub fn step_on(&mut self, terrain: &super::terrain::Terrain, target: [f32; 3]) {
        let before = self.pos;
        let grounded = (before[1] - terrain.ground_height(before)).abs() <= data::SUPPORT_EPS;
        self.step(target);
        if self.state == State::Fall {
            self.pos[0] += self.air[0];
            self.pos[2] += self.air[1];
        }
        if self.state == State::Held {
            self.vy = 0.0;
            return;
        }
        if grounded {
            self.vy = 0.0;
        } else {
            self.vy += super::player::gravity(self.vy) * DT;
            self.pos[1] += self.vy * DT * super::player::UNIT_M;
        }
        let (pos, wall, _) = terrain.move_body(before, self.pos, grounded);
        self.pos = pos;
        if let (Some(n), Some(g)) = (wall, self.goal(target)) {
            self.detour = terrain.detour(pos, n, g).or(self.detour);
        }
        if (self.pos[1] - terrain.ground_height(self.pos)).abs() <= data::SUPPORT_EPS {
            self.vy = 0.0;
            if self.state == State::Fall { self.start(State::Land, "LAND07"); }
        } else if self.state == State::Chase {
            self.air = [pos[0] - before[0], pos[2] - before[2]];
            self.start(State::Fall, "FALL07");
        }
    }

    /// Advance animation/AI one 60 Hz frame without terrain (also used for carried bodies).
    /// `target` is the player's position (x/z used).
    pub fn step(&mut self, target: [f32; 3]) {
        let (dx, dz) = (target[0] - self.pos[0], target[2] - self.pos[2]);
        let dist = dx.hypot(dz);
        if let Some(g) = self.goal(target) {
            let g = self.detour.unwrap_or(g);
            let (gx, gz) = (g[0] - self.pos[0], g[2] - self.pos[2]);
            let d = (gx.atan2(gz) - self.heading + std::f32::consts::PI).rem_euclid(std::f32::consts::TAU) - std::f32::consts::PI;
            let max = super::player::TURN_RATE * DT;
            self.heading += d.clamp(-max, max);
            // Reaching a detour waypoint only clears it; the state's own goal is next.
            if gx.hypot(gz) < data::ARRIVE && self.detour.take().is_none() {
                match self.state {
                    State::Investigate => self.start(State::Search, "SEARCH07"),
                    State::Return if self.patrols => self.start(State::Patrol, "WALK_A07"),
                    State::Return => self.start(State::Idle, "STOP07"),
                    State::Patrol => self.leg ^= 1,
                    _ => {}
                }
            }
        }
        if self.state == State::Chase {
            self.lost = if dist > data::LOSE_RANGE { self.lost + 1 } else { 0 };
            if self.lost > data::GIVE_UP {
                self.lost = 0;
                self.lead = target;
                self.start(State::Investigate, "WALK_A07");
            }
        }
        let done = self.world.done();
        self.guard_left = self.guard_left.saturating_sub(1);
        if self.state == State::Held {
            self.held_for += 1;
        }
        match self.state {
            State::Alert if done => self.start(State::Chase, "RUN07"),
            State::Held if done => self.world = World::new("GU_B07"),
            State::Down if done => self.start(State::GetUp, "UP_FR07"),
            State::GetUp | State::Land if done => self.start(State::Chase, "RUN07"),
            State::Hit if done && self.world.action.name == "HI_F07" => self.start(State::GetUp, "UP_BA07"),
            State::Hit | State::Attack if done => self.start(State::Chase, "RUN07"),
            State::Search if done => self.start(State::Return, "WALK_A07"),
            State::Chase if dist <= data::ENEMY_ATTACK_RANGE && self.swings % data::GUARD_EVERY == data::GUARD_EVERY - 1 => {
                self.swings += 1;
                self.guard_left = data::GUARD_HOLD;
                self.start(State::Guard, "GU_A07");
            }
            State::Chase if dist <= data::ENEMY_ATTACK_RANGE => {
                self.swings += 1;
                self.start(State::Attack, data::ENEMY_ATTACK_CLIPS[self.swings % data::ENEMY_ATTACK_CLIPS.len()]);
            }
            State::Idle | State::Patrol | State::Chase | State::Investigate | State::Return | State::Fall if done => self.world = World::new(self.world.action.name),
            State::Idle | State::Chase if self.guard_held => self.start(State::Guard, "GU_A07"),
            State::Guard if !self.guard_held && self.guard_left == 0 => self.start(State::Chase, "RUN07"),
            State::Guard if done => self.start(State::Guard, "GU_B07"),
            _ => {}
        }
        let before = self.world.pos;
        self.world.step();
        let d: [f32; 3] = std::array::from_fn(|i| self.world.pos[i] - before[i]);
        let (s, c) = self.heading.sin_cos();
        self.pos[0] += (d[0] * c + d[2] * s) * self.slow;
        self.pos[1] += d[1];
        self.pos[2] += (d[2] * c - d[0] * s) * self.slow;
    }
}
