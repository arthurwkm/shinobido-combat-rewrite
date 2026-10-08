use super::{extracted::ACTIONS, World};

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum State {
    Idle,
    Run,
    /// WALK_F02 loop (suffix-02 set; hypothesis: slow stick tilt).
    Walk,
    Halt,
    Attack,
    /// GU_A01 then GU_B01 while held.
    Guard,
    /// GU_Y01 (hypothesis: guard release).
    GuardEnd,
    /// FRONT01 / BACK01 / RIGHT01 / LEFT01 (hypothesis: evasive steps; 30 frames, 1.5-1.6 m of root motion each).
    Dodge,
    /// JUMP01 animation; height from the game's gravity update (jump launch velocity and unit scale are ESTIMATE).
    Jump,
    /// LAND01 after touching down.
    Land,
    /// A HI_*01 hit reaction (hypothesis: B/U/C letters are the hit side or height; which one the game picks is ESTIMATE).
    Hit,
    /// One-shot ninja action (SHEATH01, PICK01, THROW_A01/B01, DRINK01, NINJYUTU01, MAKIBISHI01); role per name is a hypothesis.
    Special,
}

/// Gravity update from the executable (0x1102b0, formats.md), in game units.
fn gravity(vy: f32) -> f32 {
    let m = vy.abs();
    if m > 15.0 { -450.0 } else { -100.0 - 0.4 * m }
}
/// ESTIMATE: launch velocity in game units/s; chosen so the airborne time (~51 frames) matches JUMP01's footstep
/// events at frames 5 (take-off) and 56 (landing). The launch value itself was not found in the executable.
const JUMP_V0: f32 = 155.0;
/// ESTIMATE: metres per game unit, so that -450 u/s^2 is 9.8 m/s^2.
const UNIT_M: f32 = 9.8 / 450.0;
/// JUMP01 event (5, 7, ...) marks take-off.
const TAKEOFF_FRAME: u32 = 5;

/// Ground turn rate, rad/s (2.5 pi). From the executable's turn function 0x10bf80, default-state branch (0x10c1f4);
/// other states use other rates (formats.md). Which branch applies to which action is not decoded: locomotion only.
const TURN_RATE: f32 = 7.853_981_6;

/// Idle / run / halt on top of `World`. ESTIMATE: the transitions (release while running plays HALT01 once, then
/// idle; pressing forward during HALT01 interrupts it) are guessed; the motions and root motion are from the files.
pub struct Player {
    world: World,
    pub state: State,
    /// Guard button held. Guard is only entered from idle/run/halt, not mid-attack (ESTIMATE).
    pub guard_held: bool,
    /// Gentle forward input: walk instead of run (ESTIMATE: the game's analog threshold is not decoded).
    pub walk: bool,
    /// Clip used while walking: WALK_F02 (default), WALK_B02, WALK_R02 or WALK_L02 (ESTIMATE: picking by input direction).
    pub walk_clip: &'static str,
    /// RUN01, RUN_R01 or RUN_L01; selection policy is ESTIMATE (data.rs).
    pub run_clip: &'static str,
    /// STOP01, STOP02 or TIRED01; selection policy is ESTIMATE (data.rs).
    pub idle_clip: &'static str,
    /// Metres since creation, in the character's frame (+z forward).
    pub pos: [f32; 3],
    /// Yaw in radians; forward is (sin h, cos h) in x/z. `pos` is in the world frame.
    pub heading: f32,
    /// Direction the stick asks for (radians); the heading turns toward it at `TURN_RATE`.
    pub target_heading: f32,
    pub wall_normal: Option<[f32; 3]>,
    vy: f32,
}

impl Default for Player {
    fn default() -> Self {
        Self::new()
    }
}

impl Player {
    pub fn new() -> Self {
        Player { world: World::new("STOP01"), state: State::Idle, guard_held: false, walk: false, walk_clip: "WALK_F02", run_clip: "RUN01", idle_clip: "STOP01", pos: [0.0; 3], heading: 0.0, target_heading: 0.0, wall_normal: None, vy: 0.0 }
    }

    /// `attack` is true on the frame the button goes down.
    pub fn step(&mut self, forward: bool, attack: bool) {
        self.step_on(&super::terrain::Terrain::default(), forward, attack);
    }

    /// Advance on shared terrain. Falling keeps the current clip until the falling-motion item.
    pub fn step_on(&mut self, terrain: &super::terrain::Terrain, forward: bool, attack: bool) {
        let before = self.pos;
        let grounded = (before[1] - terrain.ground_height(before)).abs() <= super::data::SUPPORT_EPS;
        self.step_action(forward, attack);
        let jumping = self.state == State::Jump && self.world.frame >= TAKEOFF_FRAME;
        if !jumping && !grounded {
            self.vy += gravity(self.vy) * super::DT;
            self.pos[1] += self.vy * super::DT * UNIT_M;
        }
        let (pos, wall, ceiling) = terrain.move_body(before, self.pos, grounded && !jumping);
        self.pos = pos;
        self.wall_normal = wall;
        if ceiling { self.vy = 0.0; }
        if self.vy <= 0.0 && (pos[1] - terrain.ground_height(pos)).abs() <= super::data::SUPPORT_EPS {
            self.vy = 0.0;
            if jumping || !grounded { self.start(State::Land, "LAND01"); }
        }
    }

    fn step_action(&mut self, forward: bool, attack: bool) {
        // Keep caller-selected clips within their supported locomotion families.
        if !super::data::RUN_CLIPS.contains(&self.run_clip) { self.run_clip = "RUN01"; }
        if !super::data::IDLE_CLIPS.contains(&self.idle_clip) { self.idle_clip = "STOP01"; }
        if attack {
            self.press_attack();
        }
        if matches!(self.state, State::Idle | State::Run | State::Walk | State::Halt) {
            let d = (self.target_heading - self.heading + std::f32::consts::PI).rem_euclid(std::f32::consts::TAU)
                - std::f32::consts::PI;
            let max = TURN_RATE * super::DT;
            self.heading += d.clamp(-max, max);
        }
        match (self.state, self.guard_held) {
            (State::Idle | State::Run | State::Walk | State::Halt, true) => self.start(State::Guard, "GU_A01"),
            (State::Guard, false) => self.start(State::GuardEnd, "GU_Y01"),
            (State::Guard, true) if self.world.done() && self.world.action.name == "GU_B01" => {
                self.start(State::Guard, "GU_B01") // loops while held
            }
            (State::Guard, true) if self.world.action.name == "GU_A01" => {
                // GU_A01's cancel window (event type 3) opens the move to GU_B01
                let f = self.world.frame;
                let to = self.world.action.cancels.iter().find(|c| c.1 <= f).and_then(|c| ACTIONS.iter().find(|x| x.id == c.0));
                if let Some(t) = to {
                    self.start(State::Guard, t.name);
                }
            }
            (State::GuardEnd, _) if self.world.done() => self.start(State::Idle, self.idle_clip),
            _ => {}
        }
        if self.state == State::Jump {
            self.advance();
            if self.world.frame >= TAKEOFF_FRAME {
                if self.world.frame == TAKEOFF_FRAME {
                    self.vy = JUMP_V0;
                }
                self.vy += gravity(self.vy) * super::DT;
                self.pos[1] += self.vy * super::DT * UNIT_M;
            }
            return;
        }
        if self.state == State::Land {
            if self.world.done() {
                self.start(State::Idle, self.idle_clip);
            }
            return self.advance();
        }
        if self.state == State::Dodge && self.world.done() {
            self.start(State::Idle, self.idle_clip);
        }
        if matches!(self.state, State::Hit | State::Special) && self.world.done() {
            self.start(State::Idle, self.idle_clip);
        }
        if matches!(self.state, State::Guard | State::GuardEnd | State::Dodge | State::Hit | State::Special) {
            return self.advance();
        }
        if self.state == State::Attack {
            if self.world.done() {
                self.start(State::Idle, self.idle_clip);
            }
            return self.advance();
        }
        let loco = if self.walk { State::Walk } else { State::Run };
        let next = match (self.state, forward) {
            (State::Run | State::Walk, false) => Some(State::Halt),
            (State::Idle | State::Halt, true) => Some(loco),
            (State::Idle, false) if self.world.action.name != self.idle_clip => Some(State::Idle),
            (State::Run, true) if !self.walk && self.world.action.name != self.run_clip => Some(State::Run),
            (State::Walk, true) if self.walk && self.world.action.name != self.walk_clip => Some(State::Walk),
            (State::Run | State::Walk, true) if self.state != loco => Some(loco),
            (State::Halt, false) if self.world.done() => Some(State::Idle),
            _ => None,
        };
        let walk_clip = self.walk_clip;
        let run_clip = self.run_clip;
        let idle_clip = self.idle_clip;
        let clip = |s| match s {
            State::Run => run_clip,
            State::Walk => walk_clip,
            State::Halt => "HALT01",
            _ => idle_clip,
        };
        if let Some(s) = next {
            self.start(s, clip(s));
        } else if self.world.done() && self.state != State::Halt {
            self.world = World::new(clip(self.state));
        }
        self.advance();
    }

    pub fn world_action(&self) -> &'static str {
        self.world.action.name
    }

    /// Start an evasive step from idle/run/halt. `dir`: "FRONT01", "BACK01", "RIGHT01" or "LEFT01".
    pub fn dodge(&mut self, dir: &str) {
        if matches!(self.state, State::Idle | State::Run | State::Walk | State::Halt) {
            self.start(State::Dodge, dir);
        }
    }

    /// Take a hit: interrupts any state and plays the given reaction (`HI_BS01`, `HI_US01`, ...) to its end.
    pub fn hit(&mut self, reaction: &str) {
        self.vy = 0.0;
        self.start(State::Hit, reaction);
    }

    /// Also used for RUN_RE01/RUN_LE01 and STOP_K01 until wall-state selection is implemented.
    /// Start a one-shot ninja action from idle/run/walk/halt; it plays to its end, then returns to idle (ESTIMATE: no cancels).
    pub fn special(&mut self, name: &str) {
        if matches!(self.state, State::Idle | State::Run | State::Walk | State::Halt) {
            self.start(State::Special, name);
        }
    }

    pub fn jump(&mut self) {
        if matches!(self.state, State::Idle | State::Run | State::Walk | State::Halt) {
            self.start(State::Jump, "JUMP01");
        }
    }

    fn start(&mut self, state: State, name: &str) {
        self.state = state;
        self.world = World::new(name);
    }

    fn advance(&mut self) {
        let before = self.world.pos;
        self.world.step();
        let d: [f32; 3] = std::array::from_fn(|i| self.world.pos[i] - before[i]);
        let (s, c) = self.heading.sin_cos();
        self.pos[0] += d[0] * c + d[2] * s;
        self.pos[1] += d[1];
        self.pos[2] += d[2] * c - d[0] * s;
    }

    /// ESTIMATE: the first attack is AT_A01; while an attack's cancel window is open, a press goes to the first
    /// permitted target (the choice between targets in the real game depends on input direction/state, not decoded).
    fn press_attack(&mut self) {
        let f = self.world.frame;
        let target = if self.state == State::Attack {
            let a = self.world.action;
            a.cancels.iter().filter(|c| c.1 <= f && f < c.2).find_map(|c| ACTIONS.iter().find(|x| x.id == c.0))
        } else {
            ACTIONS.iter().find(|x| x.name == "AT_A01")
        };
        if let Some(t) = target {
            self.start(State::Attack, t.name);
        }
    }
}

/// Hit damage from a type-35 event (formats.md, "Damage use of event type 35"):
/// `trunc(trunc(u16(arg0) * power) * mult)`; `power` is the attacker's float stat at `[+0x45c]+0x34`
/// (value not found: caller supplies it, ESTIMATE) and `mult` is 0.5/1.0/2.0 from the sign of the float at `+0x5c`.
pub fn hit_damage(arg0: u32, power: f32, bias: f32) -> i32 {
    let mult = if bias < 0.0 { 0.5 } else if bias <= 0.0 { 1.0 } else { 2.0 };
    (((arg0 & 0xffff) as f32 * power) as i32 as f32 * mult) as i32
}
