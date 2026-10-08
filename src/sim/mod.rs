//! The game as an engine-free state machine (grows in Milestone 4).

#[allow(clippy::excessive_precision)] // generated floats keep the game's full precision
pub mod extracted;
pub mod combat;
pub mod enemy;
pub mod execution;
pub mod input;
pub mod lockon;
pub mod camera;
pub mod stealth;
pub mod player;
pub mod terrain;
pub mod data;

#[cfg(test)]
mod tests;

use extracted::{Action, ACTIONS};

pub const DT: f32 = 1.0 / 60.0;

/// Plays one action at 60 Hz; the character moves by the action's root motion
/// (linear between keys; the .nmt tangents are not decoded yet).
pub struct World {
    pub action: &'static Action,
    pub frame: u32,
    /// Metres relative to where the action started; +z is forward in the character's frame.
    pub pos: [f32; 3],
}

impl World {
    pub fn new(name: &str) -> Self {
        let action = ACTIONS.iter().find(|a| a.name == name).expect("unknown action");
        World { action, frame: 0, pos: [0.0; 3] }
    }

    pub fn done(&self) -> bool {
        self.frame >= self.action.frames
    }

    pub fn step(&mut self) {
        if self.done() {
            return;
        }
        self.frame += 1;
        self.pos = self.root_at(self.frame);
    }

    fn root_at(&self, f: u32) -> [f32; 3] {
        let k = self.action.root;
        if k.is_empty() { return [0.0; 3]; } // static clip: no root keys
        let origin = k[0].1;
        let i = k.iter().rposition(|key| key.0 <= f).unwrap_or(0);
        let (a, b) = (k[i], k[(i + 1).min(k.len() - 1)]);
        let t = if b.0 > a.0 { (f.min(b.0).saturating_sub(a.0)) as f32 / (b.0 - a.0) as f32 } else { 0.0 };
        std::array::from_fn(|c| a.1[c] + (b.1[c] - a.1[c]) * t - origin[c])
    }
}
