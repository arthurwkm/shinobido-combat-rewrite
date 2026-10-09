//! The foot soldier (I_BUSHI, motion set 07, docs/enemy.md) as an engine-free state machine.
//! Transitions are ESTIMATE: the AI files give attack choice tables, not these states.

use super::{data, World, DT};

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
}

pub struct Enemy {
    world: World,
    pub state: State,
    pub hp: i32,
    pub guard_held: bool,
    /// World position (m); the root motion moves it along `heading`.
    pub pos: [f32; 3],
    /// Yaw in radians; forward is (sin h, cos h) in x/z.
    pub heading: f32,
    /// Frames spent in `Held` and presses made meanwhile.
    pub held_for: u32,
    pub mashes: u32,
    /// Root-motion speed factor (1 normally, less in caltrops).
    pub slow: f32,
    swings: usize,
}

impl Enemy {
    pub fn new(pos: [f32; 3]) -> Self {
        Enemy { world: World::new("STOP07"), state: State::Idle, hp: data::ENEMY_HP, guard_held: false, pos, heading: 0.0, held_for: 0, mashes: 0, slow: 1.0, swings: 0 }
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

    pub fn patrol(&mut self) {
        if self.state == State::Idle {
            self.start(State::Patrol, "WALK_A07");
        }
    }

    /// The soldier notices the player (stealth will drive this).
    pub fn alert(&mut self) {
        if matches!(self.state, State::Idle | State::Patrol) {
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
    }

    /// Advance one 60 Hz frame; `target` is the player's position (x/z used).
    pub fn step(&mut self, target: [f32; 3]) {
        let (dx, dz) = (target[0] - self.pos[0], target[2] - self.pos[2]);
        let dist = dx.hypot(dz);
        if matches!(self.state, State::Chase | State::Alert) {
            let d = (dx.atan2(dz) - self.heading + std::f32::consts::PI).rem_euclid(std::f32::consts::TAU) - std::f32::consts::PI;
            let max = super::player::TURN_RATE * DT;
            self.heading += d.clamp(-max, max);
        }
        let done = self.world.done();
        if self.state == State::Held {
            self.held_for += 1;
        }
        match self.state {
            State::Alert if done => self.start(State::Chase, "RUN07"),
            State::Held if done => self.world = World::new("GU_B07"),
            State::Down if done => self.start(State::GetUp, "UP_FR07"),
            State::GetUp if done => self.start(State::Chase, "RUN07"),
            State::Hit if done && self.world.action.name == "HI_F07" => self.start(State::GetUp, "UP_BA07"),
            State::Hit | State::Attack if done => self.start(State::Chase, "RUN07"),
            State::Chase if dist <= data::ENEMY_ATTACK_RANGE => {
                self.swings += 1;
                self.start(State::Attack, data::ENEMY_ATTACK_CLIPS[self.swings % data::ENEMY_ATTACK_CLIPS.len()]);
            }
            State::Idle | State::Patrol | State::Chase if done => self.world = World::new(self.world.action.name),
            State::Idle | State::Chase if self.guard_held => self.start(State::Guard, "GU_A07"),
            State::Guard if !self.guard_held => self.start(State::Idle, "STOP07"),
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
