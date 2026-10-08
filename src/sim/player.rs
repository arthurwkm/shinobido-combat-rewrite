use super::{extracted::ACTIONS, World};

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum State {
    Idle,
    Run,
    Halt,
    Attack,
    /// GU_A01 then GU_B01 while held.
    Guard,
    /// GU_Y01 (hypothesis: guard release).
    GuardEnd,
    /// FRONT01 / BACK01 / RIGHT01 / LEFT01 (hypothesis: evasive steps; 30 frames, 1.5-1.6 m of root motion each).
    Dodge,
}

/// Idle / run / halt on top of `World`. ESTIMATE: the transitions (release while running plays HALT01 once, then
/// idle; pressing forward during HALT01 interrupts it) are guessed; the motions and root motion are from the files.
pub struct Player {
    world: World,
    pub state: State,
    /// Guard button held. Guard is only entered from idle/run/halt, not mid-attack (ESTIMATE).
    pub guard_held: bool,
    /// Metres since creation, in the character's frame (+z forward).
    pub pos: [f32; 3],
}

impl Player {
    pub fn new() -> Self {
        Player { world: World::new("STOP01"), state: State::Idle, guard_held: false, pos: [0.0; 3] }
    }

    /// `attack` is true on the frame the button goes down.
    pub fn step(&mut self, forward: bool, attack: bool) {
        if attack {
            self.press_attack();
        }
        match (self.state, self.guard_held) {
            (State::Idle | State::Run | State::Halt, true) => self.start(State::Guard, "GU_A01"),
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
        let next = match (self.state, forward) {
            (State::Run, false) => Some(State::Halt),
            (State::Idle | State::Halt, true) => Some(State::Run),
            (State::Halt, false) if self.world.done() => Some(State::Idle),
            _ => None,
        };
        if let Some(s) = next {
            self.state = s;
            self.world = World::new(match s {
                State::Idle => "STOP01",
                State::Run => "RUN01",
                State::Halt => "HALT01",
                State::Attack | State::Guard | State::GuardEnd | State::Dodge => unreachable!(),
            });
        } else if self.world.done() && self.state != State::Halt {
            self.world = World::new(if self.state == State::Run { "RUN01" } else { "STOP01" });
        }
        self.advance();
    }

    pub fn world_action(&self) -> &'static str {
        self.world.action.name
    }

    /// Start an evasive step from idle/run/halt. `dir`: "FRONT01", "BACK01", "RIGHT01" or "LEFT01".
    pub fn dodge(&mut self, dir: &str) {
        if matches!(self.state, State::Idle | State::Run | State::Halt) {
            self.start(State::Dodge, dir);
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
