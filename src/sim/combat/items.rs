//! Item slot: DRINK heals, MAKIBISHI caltrops slow the soldier, NINJYUTU staggers him. Effects, counts and ranges are ESTIMATE (data.rs).

use super::{data, in_cone, EnemyState, Fight};

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Item {
    Drink,
    Makibishi,
    Ninjyutu,
}

impl Item {
    pub const ALL: [Item; 3] = [Item::Drink, Item::Makibishi, Item::Ninjyutu];

    pub fn clip(self) -> &'static str {
        ["DRINK01", "MAKIBISHI01", "NINJYUTU01"][self as usize]
    }
}

pub struct Items {
    pub selected: Item,
    pub count: [u32; 3],
    /// Caltrops lying on the ground (x, z).
    pub caltrops: Vec<[f32; 2]>,
}

impl Default for Items {
    fn default() -> Self {
        Items { selected: Item::Drink, count: data::ITEM_COUNTS, caltrops: vec![] }
    }
}

impl Fight {
    /// Select the next item in the slot.
    pub fn next_item(&mut self) {
        let i = &mut self.items;
        i.selected = Item::ALL[(i.selected as usize + 1) % Item::ALL.len()];
    }

    /// Item button: play the selected item's clip and apply its effect; false when none is left or the player is busy.
    pub fn use_item(&mut self) -> bool {
        let item = self.items.selected;
        if self.items.count[item as usize] == 0 || !self.player.request(super::State::Special, item.clip()) {
            return false;
        }
        self.items.count[item as usize] -= 1;
        let p = &self.player;
        match item {
            Item::Drink => self.player_hp = (self.player_hp + data::DRINK_HEAL).min(data::PLAYER_HP),
            Item::Makibishi => {
                let (s, c) = p.heading.sin_cos();
                self.items.caltrops.push([p.pos[0] + s * data::CALTROP_FWD, p.pos[2] + c * data::CALTROP_FWD]);
            }
            Item::Ninjyutu => {
                let (pos, heading) = (p.pos, p.heading);
                for e in std::iter::once(&mut self.enemy).chain(self.others.iter_mut()) {
                    let (dx, dz) = (e.pos[0] - pos[0], e.pos[2] - pos[2]);
                    if dx.hypot(dz) <= data::NINJYUTU_RANGE && in_cone(heading, dx, dz, data::NINJYUTU_CONE)
                        && !matches!(e.state, EnemyState::Dead | EnemyState::Held | EnemyState::Down | EnemyState::GetUp) {
                        e.hit("HI_BS07", 0);
                    }
                }
            }
        }
        true
    }

    /// The soldier crawls through caltrops at `CALTROP_SLOW` speed.
    pub(super) fn caltrops(&mut self) {
        let e = &mut self.enemy;
        let on = self.items.caltrops.iter().any(|c| (c[0] - e.pos[0]).hypot(c[1] - e.pos[2]) <= data::CALTROP_RADIUS);
        e.slow = if on { data::CALTROP_SLOW } else { 1.0 };
    }
}
