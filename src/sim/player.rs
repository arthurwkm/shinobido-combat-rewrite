use super::{extracted::ACTIONS, World};

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum State {
    Idle,
    Run,
    Halt,
    Attack,
}

/// Idle / run / halt on top of `World`. ESTIMATE: the transitions (release while running plays HALT01 once, then
/// idle; pressing forward during HALT01 interrupts it) are guessed; the motions and root motion are from the files.
pub struct Player {
    world: World,
    pub state: State,
    /// Metres since creation, in the character's frame (+z forward).
    pub pos: [f32; 3],
}

impl Player {
    pub fn new() -> Self {
        Player { world: World::new("STOP01"), state: State::Idle, pos: [0.0; 3] }
    }

    /// `attack` is true on the frame the button goes down.
    pub fn step(&mut self, forward: bool, attack: bool) {
        if attack {
            self.press_attack();
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
                State::Attack => unreachable!(),
            });
        } else if self.world.done() && self.state != State::Halt {
            self.world = World::new(if self.state == State::Run { "RUN01" } else { "STOP01" });
        }
        self.advance();
    }

    pub fn world_action(&self) -> &'static str {
        self.world.action.name
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
