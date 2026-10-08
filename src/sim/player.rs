use super::World;

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum State {
    Idle,
    Run,
    Halt,
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

    pub fn step(&mut self, forward: bool) {
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
            });
        } else if self.world.done() && self.state != State::Halt {
            self.world = World::new(if self.state == State::Run { "RUN01" } else { "STOP01" });
        }
        let before = self.world.pos;
        self.world.step();
        for c in 0..3 {
            self.pos[c] += self.world.pos[c] - before[c];
        }
    }
}
