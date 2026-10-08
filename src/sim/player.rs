use super::{extracted::ACTIONS, World};

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum State {
    Idle,
    Run,
    /// WALK_F02 loop (suffix-02 set; hypothesis: slow stick tilt).
    Walk,
    Halt,
    /// L1 held: STOP_S05 idle, WALK_F05 moving (set 05 `_S` variants).
    Crouch,
    /// Back to a tall wall: IKOU_B06 in, STOP_A06 idle, ONE06 side-step, STOP_B06 peek at a wall end, BACK06 out.
    Hug,
    Attack,
    /// GU_A01 then GU_B01 while held.
    Guard,
    /// GU_Y01 (hypothesis: guard release).
    GuardEnd,
    /// FRONT01 / BACK01 / RIGHT01 / LEFT01 (hypothesis: evasive steps; 30 frames, 1.5-1.6 m of root motion each).
    Dodge,
    /// JUMP01 animation; height from the game's gravity update (jump launch velocity and unit scale are ESTIMATE).
    Jump,
    /// FALL01 loop while off the ground without a jump (horizontal speed carried from the last grounded frame: ESTIMATE).
    Fall,
    /// LAND01 (LAND_B01 / LAND_A01 after longer falls) after touching down.
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
pub const TURN_RATE: f32 = 7.853_981_6;

/// Idle / run / halt on top of `World`. ESTIMATE: the transitions (release while running plays HALT01 once, then
/// idle; pressing forward during HALT01 interrupts it) are guessed; the motions and root motion are from the files.
pub struct Player {
    world: World,
    pub state: State,
    /// Guard button held. Guard is only entered from idle/run/halt, not mid-attack (ESTIMATE).
    pub guard_held: bool,
    /// Crouch button held (L1); only from free locomotion.
    pub crouch_held: bool,
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
    /// The wall in contact (or behind, while hugging) is at least `HUG_WALL_MIN` tall.
    wall_tall: bool,
    /// Sign of the side-step that reached a wall end (0 when none): peeking blocks further movement that way.
    hug_edge: f32,
    hug_dir: f32,
    vy: f32,
    fall_from: f32,
    air_vel: [f32; 2],
}

impl Default for Player {
    fn default() -> Self {
        Self::new()
    }
}

impl Player {
    pub fn new() -> Self {
        Player { world: World::new("STOP01"), state: State::Idle, guard_held: false, crouch_held: false, walk: false, walk_clip: "WALK_F02", run_clip: "RUN01", idle_clip: "STOP01", pos: [0.0; 3], heading: 0.0, target_heading: 0.0, wall_normal: None, wall_tall: false, hug_edge: 0.0, hug_dir: 0.0, vy: 0.0, fall_from: 0.0, air_vel: [0.0; 2] }
    }

    /// `attack` is true on the frame the button goes down.
    pub fn step(&mut self, forward: bool, attack: bool) {
        self.step_on(&super::terrain::Terrain::default(), forward, attack);
    }

    /// Advance on shared terrain. Leaving the ground without a jump plays FALL01, then a landing clip by drop height.
    pub fn step_on(&mut self, terrain: &super::terrain::Terrain, forward: bool, attack: bool) {
        let before = self.pos;
        let (sh, ch) = self.heading.sin_cos();
        self.wall_tall = match if self.state == State::Hug { Some([sh, 0.0, ch]) } else { self.wall_normal } {
            Some(n) => terrain.wall_behind(before, n, super::data::HUG_WALL_MIN),
            None => false,
        };
        let grounded = (before[1] - terrain.ground_height(before)).abs() <= super::data::SUPPORT_EPS;
        self.step_action(forward, attack);
        let jumping = self.state == State::Jump && self.world.frame >= TAKEOFF_FRAME;
        if !grounded && !jumping && matches!(self.state, State::Idle | State::Run | State::Walk | State::Halt | State::Crouch) {
            self.air_vel = [self.pos[0] - before[0], self.pos[2] - before[2]];
            self.fall_from = before[1];
            self.start(State::Fall, "FALL01");
        } else if self.state == State::Fall {
            self.pos[0] += self.air_vel[0];
            self.pos[2] += self.air_vel[1];
        }
        // Special clips (climbs) own the height through root motion.
        let rooted = self.state == State::Special;
        if !jumping && !grounded && !rooted {
            self.vy += gravity(self.vy) * super::DT;
            self.pos[1] += self.vy * super::DT * UNIT_M;
        }
        let (pos, wall, ceiling) = terrain.move_body(before, self.pos, grounded && !jumping);
        self.pos = pos;
        self.wall_normal = wall;
        if ceiling { self.vy = 0.0; }
        if !rooted && self.vy <= 0.0 && (pos[1] - terrain.ground_height(pos)).abs() <= super::data::SUPPORT_EPS {
            self.vy = 0.0;
            if jumping || !grounded {
                let drop = if self.state == State::Fall { self.fall_from - pos[1] } else { 0.0 };
                let clip = if drop >= super::data::LAND_A_DROP { "LAND_A01" } else if drop >= super::data::LAND_B_DROP { "LAND_B01" } else { "LAND01" };
                self.start(State::Land, clip);
            }
        }
    }

    fn step_action(&mut self, forward: bool, attack: bool) {
        // Keep caller-selected clips within their supported locomotion families.
        if !super::data::RUN_CLIPS.contains(&self.run_clip) { self.run_clip = "RUN01"; }
        if !super::data::IDLE_CLIPS.contains(&self.idle_clip) { self.idle_clip = "STOP01"; }
        if attack {
            self.press_attack();
        }
        if matches!(self.state, State::Idle | State::Run | State::Walk | State::Halt | State::Crouch) {
            let d = (self.target_heading - self.heading + std::f32::consts::PI).rem_euclid(std::f32::consts::TAU)
                - std::f32::consts::PI;
            let max = TURN_RATE * super::DT;
            self.heading += d.clamp(-max, max);
        }
        match (self.state, self.guard_held) {
            (State::Idle | State::Run | State::Walk | State::Halt | State::Crouch, true) => self.start(State::Guard, "GU_A01"),
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
        if self.state == State::Fall {
            if self.world.done() { self.start(State::Fall, "FALL01"); }
            return self.advance();
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
        if self.state == State::Hug {
            return self.step_hug(forward);
        }
        if self.state == State::Attack {
            if self.world.done() {
                self.start(State::Idle, self.idle_clip);
            }
            return self.advance();
        }
        if let (true, true, true, Some(n)) = (self.crouch_held, forward, self.wall_tall, self.wall_normal) {
            let (s, c) = self.target_heading.sin_cos();
            if -(s * n[0] + c * n[2]) > 0.7 {
                self.heading = n[0].atan2(n[2]);
                self.hug_edge = 0.0;
                self.start(State::Hug, "IKOU_B06");
                return self.advance();
            }
        }
        if self.crouch_held && !self.guard_held {
            let clip = if forward { "WALK_F05" } else { "STOP_S05" };
            if self.state != State::Crouch || self.world.action.name != clip {
                self.start(State::Crouch, clip);
            } else if self.world.done() {
                self.world = World::new(clip);
            }
            return self.advance();
        }
        if self.state == State::Crouch {
            self.start(State::Idle, self.idle_clip);
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

    /// Hug: the wall is behind the player (heading faces away from it). Stick sideways along the wall side-steps at
    /// `HUG_SPEED`, a wall end stops it and peeks, stick away or releasing crouch plays BACK06.
    fn step_hug(&mut self, forward: bool) {
        let name = self.world.action.name;
        let d = self.target_heading - self.heading;
        let (side, away) = if forward { (d.sin(), d.cos() > 0.7) } else { (0.0, false) };
        let side = if side.abs() > 0.5 { side.signum() } else { 0.0 };
        if name == "BACK06" {
            if self.world.done() { self.start(State::Idle, self.idle_clip); }
            return self.advance();
        }
        if !self.crouch_held || away {
            self.start(State::Hug, "BACK06");
            return self.advance();
        }
        if name == "IKOU_B06" && !self.world.done() {
            return self.advance();
        }
        if !self.wall_tall && self.hug_edge == 0.0 {
            self.hug_edge = self.hug_dir;
        }
        if self.wall_tall { self.hug_edge = 0.0; }
        let moving = side != 0.0 && side != self.hug_edge;
        if moving { self.hug_dir = side; }
        // ESTIMATE: ONE06 / TWO06 are the two side-step directions; which is which is not readable from the files
        let clip = if moving { if self.hug_dir > 0.0 { "ONE06" } else { "TWO06" } } else if self.hug_edge != 0.0 { "STOP_B06" } else { "STOP_A06" };
        if name != clip {
            self.start(State::Hug, clip);
        } else if self.world.done() {
            self.world = World::new(clip);
        }
        if moving {
            let n = self.heading;
            self.pos[0] += n.cos() * side * super::data::HUG_SPEED * super::DT;
            self.pos[2] -= n.sin() * side * super::data::HUG_SPEED * super::DT;
        }
        self.advance();
    }

    /// True while the current attack clip's hit window is open (event types 1/2).
    pub fn attack_active(&self) -> bool {
        self.state == State::Attack && self.world.action.hits.iter().any(|h| (h.0..h.1).contains(&self.world.frame))
    }

    /// Damage argument of the current attack clip's first type-35 event, if any.
    pub fn attack_arg(&self) -> Option<u32> {
        self.world.action.events.iter().find(|e| e.1 == 35).map(|e| e.2)
    }

    pub fn world_action(&self) -> &'static str {
        self.world.action.name
    }

    /// Length in frames of the current clip.
    pub fn world_len(&self) -> u32 {
        self.world.action.frames
    }

    /// Frame of the current clip (for posing the rig).
    pub fn world_frame(&self) -> u32 {
        self.world.frame
    }

    /// Start an evasive step from idle/run/halt. `dir`: "FRONT01", "BACK01", "RIGHT01" or "LEFT01".
    pub fn dodge(&mut self, dir: &str) {
        self.request(State::Dodge, dir);
    }

    /// Take a hit: interrupts any state and plays the given reaction (`HI_BS01`, `HI_US01`, ...) to its end.
    pub fn hit(&mut self, reaction: &str) {
        self.vy = 0.0;
        self.start(State::Hit, reaction);
    }

    /// Also used for RUN_RE01/RUN_LE01 and STOP_K01 until wall-state selection is implemented.
    /// Start a one-shot ninja action from idle/run/walk/halt; it plays to its end, then returns to idle (ESTIMATE: no cancels).
    pub fn special(&mut self, name: &str) {
        self.request(State::Special, name);
    }

    pub fn jump(&mut self) {
        self.request(State::Jump, "JUMP01");
    }

    /// The one gate for player-chosen actions: free locomotion may start anything; any other state only what the
    /// current clip's open cancel window (event types 3/4) names. Returns whether the action started.
    pub fn request(&mut self, state: State, name: &str) -> bool {
        let f = self.world.frame;
        let id = ACTIONS.iter().find(|x| x.name == name).map(|x| x.id);
        let free = matches!(self.state, State::Idle | State::Run | State::Walk | State::Halt | State::Crouch);
        let ok = free || self.world.action.cancels.iter().any(|c| Some(c.0) == id && c.1 <= f && f < c.2);
        if ok {
            self.start(state, name);
        }
        ok
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
            self.request(State::Attack, t.name);
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
