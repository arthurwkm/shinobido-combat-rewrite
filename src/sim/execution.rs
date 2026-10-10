//! When an execution is available and which kind (the viewer's execution cue). Conditions are ESTIMATE (data.rs);
//! the kind letters follow the KI_HI<B|U|C>*07 victim reactions (formats.md "Execution packs").

use super::{data, enemy::{Enemy, State}, player::Player};
use std::f32::consts::{PI, TAU};

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Kind {
    /// KI_HIB*07: soldier has not noticed the player, who is behind it.
    Back,
    /// KI_HIU*07: unaware soldier, player directly overhead.
    Above,
    /// KI_HIC*07: staggered, nearly dead soldier, player in front.
    Front,
}

impl Kind {
    /// ESTIMATE pairing (data.rs): the player clip and the victim clip.
    pub fn clips(self) -> (&'static str, &'static str) {
        data::EXEC_CLIPS[self as usize]
    }
}

/// The execution the player could start now, if any. Shows as the cue.
pub fn available(p: &Player, e: &Enemy) -> Option<Kind> {
    if e.state == State::Dead || p.carrying || matches!(p.state, super::player::State::Special | super::player::State::Hit) {
        return None;
    }
    let (dx, dz) = (p.pos[0] - e.pos[0], p.pos[2] - e.pos[2]);
    if dx.hypot(dz) > data::EXEC_REACH {
        return None;
    }
    let off = ((dx.atan2(dz) - e.heading + PI).rem_euclid(TAU) - PI).abs();
    let unaware = matches!(e.state, State::Idle | State::Patrol);
    if unaware && p.pos[1] - e.pos[1] >= data::EXEC_ABOVE_MIN {
        Some(Kind::Above)
    } else if (p.pos[1] - e.pos[1]).abs() >= data::BODY_HEIGHT {
        None
    } else if unaware && off >= data::EXEC_BEHIND_MIN {
        Some(Kind::Back)
    } else if e.state == State::Hit && (e.hp as f32) <= data::EXEC_FRONT_HP * data::ENEMY_HP as f32 && off <= data::FRONT_CONE {
        Some(Kind::Front)
    } else {
        None
    }
}
