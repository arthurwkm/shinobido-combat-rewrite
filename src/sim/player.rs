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

/// Idle / run / halt on top of `World`. ESTIMATE: the transitions (release while running plays HALT01 once, then
/// idle; pressing forward during HALT01 interrupts it) are guessed; the motions and root motion are from the files.
pub struct Player {
    world: World,
    pub state: State,
    /// Guard button held. Guard is only entered from idle/run/halt, not mid-attack (ESTIMATE).
    pub guard_held: bool,
    /// Gentle forward input: walk instead of run (ESTIMATE: the game's analog threshold is not decoded).
    pub walk: bool,
    /// Metres since creation, in the character's frame (+z forward).
    pub pos: [f32; 3],
    vy: f32,
}

impl Player {
    pub fn new() -> Self {
        Player { world: World::new("STOP01"), state: State::Idle, guard_held: false, walk: false, pos: [0.0; 3], vy: 0.0 }
    }

    /// `attack` is true on the frame the button goes down.
    pub fn step(&mut self, forward: bool, attack: bool) {
        if attack {
            self.press_attack();
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
            (State::GuardEnd, _) if self.world.done() => self.start(State::Idle, "STOP01"),
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
                if self.pos[1] <= 0.0 && self.vy < 0.0 {
                    self.pos[1] = 0.0;
                    self.start(State::Land, "LAND01");
                }
            }
            return;
        }
        if self.state == State::Land {
            if self.world.done() {
                self.start(State::Idle, "STOP01");
            }
            return self.advance();
        }
        if self.state == State::Dodge && self.world.done() {
            self.start(State::Idle, "STOP01");
        }
        if matches!(self.state, State::Guard | State::GuardEnd | State::Dodge) {
            return self.advance();
        }
        if self.state == State::Attack {
            if self.world.done() {
                self.start(State::Idle, "STOP01");
            }
            return self.advance();
        }
        let loco = if self.walk { State::Walk } else { State::Run };
        let next = match (self.state, forward) {
            (State::Run | State::Walk, false) => Some(State::Halt),
            (State::Idle | State::Halt, true) => Some(loco),
            (State::Run | State::Walk, true) if self.state != loco => Some(loco),
            (State::Halt, false) if self.world.done() => Some(State::Idle),
            _ => None,
        };
        let clip = |s| match s {
            State::Run => "RUN01",
            State::Walk => "WALK_F02",
            State::Halt => "HALT01",
            _ => "STOP01",
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
        for c in 0..3 {
            self.pos[c] += self.world.pos[c] - before[c];
        }
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
