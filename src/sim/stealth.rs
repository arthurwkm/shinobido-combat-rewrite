//! Enemy awareness: sight cone and footstep noise fill a 0..1 meter (the HUD's stealth bar); at 1 the soldier is alerted.
//! All numbers ESTIMATE (data.rs). ponytail: no wall occlusion for sight, add when the viewer terrain feeds line tests.

use super::{data, enemy::Enemy, DT};
use std::f32::consts::{PI, TAU};

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Alert {
    /// Eye icon calm.
    Unaware,
    Suspicious,
    Alerted,
}

#[derive(Default)]
pub struct Stealth {
    /// 0..=1 stealth meter.
    pub meter: f32,
    pub seen: bool,
}

impl Stealth {
    pub fn level(&self) -> Alert {
        if self.meter >= 1.0 {
            Alert::Alerted
        } else if self.meter >= data::SUSPICIOUS_AT {
            Alert::Suspicious
        } else {
            Alert::Unaware
        }
    }

    /// One 60 Hz frame. `action` is the player's current clip (sets the noise); alerting is sticky.
    pub fn step(&mut self, e: &mut Enemy, player: [f32; 3], action: &str) {
        let (dx, dz) = (player[0] - e.pos[0], player[2] - e.pos[2]);
        let dist = dx.hypot(dz);
        let off = ((dx.atan2(dz) - e.heading + PI).rem_euclid(TAU) - PI).abs();
        self.seen = dist <= data::SIGHT_RANGE && off <= data::SIGHT_CONE;
        let noise = if action.starts_with("RUN") {
            data::HEAR_RUN
        } else if action.starts_with("WALK") {
            data::HEAR_WALK
        } else {
            0.0
        };
        let rate = if self.seen {
            data::SIGHT_FILL * (1.0 - dist / data::SIGHT_RANGE)
        } else if dist <= noise {
            data::HEAR_FILL
        } else {
            -data::STEALTH_DECAY
        };
        if self.level() != Alert::Alerted {
            self.meter = (self.meter + rate * DT).clamp(0.0, 1.0);
        }
        if self.level() == Alert::Alerted {
            e.alert();
        }
    }
}
