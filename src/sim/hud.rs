//! Debug readout text and hit log (engine-free; the viewer only draws the strings).

use super::{combat::Fight, data, input::Input, stealth::Alert};

const LOG_LINES: usize = 6;

#[derive(Default)]
pub struct Debug {
    pub log: Vec<String>,
    tick: u32,
    last: Option<(i32, i32, [f32; 3])>,
    /// Horizontal speed in m/s over the last tick.
    pub speed: f32,
}

impl Debug {
    /// Call once per sim tick after the step: logs hp losses and measures speed.
    pub fn observe(&mut self, f: &Fight) {
        self.tick += 1;
        if let Some((php, ehp, pos)) = self.last {
            if f.player_hp < php {
                self.push(format!("{:>5} enemy {} hits you  -{}", self.tick, f.enemy.action(), php - f.player_hp));
            }
            if f.enemy.hp < ehp {
                self.push(format!("{:>5} {} hits enemy  -{}", self.tick, f.player.world_action(), ehp - f.enemy.hp));
            }
            self.speed = (f.player.pos[0] - pos[0]).hypot(f.player.pos[2] - pos[2]) / super::DT;
        }
        self.last = Some((f.player_hp, f.enemy.hp, f.player.pos));
    }

    fn push(&mut self, line: String) {
        self.log.push(line);
        if self.log.len() > LOG_LINES {
            self.log.remove(0);
        }
    }

    /// Top-right block: action, frame/length, speed, height, lock-on, queued input.
    pub fn readout(&self, f: &Fight, input: &Input) -> String {
        let p = &f.player;
        let queued: Vec<&str> = [(input.attack, "attack"), (input.jump, "jump"), (input.item, "item")]
            .iter().filter(|q| q.0).map(|q| q.1).collect();
        format!(
            "action  {}\nframe   {}/{}\nspeed   {:.2} m/s\nheight  {:.2} m\nlock-on {}\nitem    {:?} x{}\nqueued  {}",
            p.world_action(), p.world_frame(), p.world_len(), self.speed, p.pos[1],
            match (input.lock, input.lock_target) { (_, Some(_)) => "locked", (true, _) => "held", _ => "off" },
            f.items.selected, f.items.count[f.items.selected as usize],
            if queued.is_empty() { "-".into() } else { queued.join(" ") },
        )
    }
}

/// Fraction 0..1 for the bars.
pub fn player_bar(f: &Fight) -> f32 {
    f.player_hp as f32 / data::PLAYER_HP as f32
}

pub fn enemy_bar(f: &Fight) -> f32 {
    f.enemy.hp.max(0) as f32 / data::ENEMY_HP as f32
}

/// Eye awareness icon colour (RGB). ESTIMATE: calm teal as seen in the stealth clip, amber, red (docs/video_notes.md).
pub fn eye_colour(a: Alert) -> [f32; 3] {
    match a {
        Alert::Unaware => [0.2, 0.7, 0.6],
        Alert::Suspicious => [0.95, 0.7, 0.15],
        Alert::Alerted => [0.85, 0.12, 0.1],
    }
}

/// Ki compass needle: bearing of the living enemy relative to the camera, radians, 0 = straight ahead, clockwise +.
pub fn compass(f: &Fight, cam_yaw: f32) -> Option<f32> {
    use std::f32::consts::{PI, TAU};
    if f.enemy.hp <= 0 { return None; }
    let (dx, dz) = (f.enemy.pos[0] - f.player.pos[0], f.enemy.pos[2] - f.player.pos[2]);
    Some((dx.atan2(dz) - cam_yaw + PI).rem_euclid(TAU) - PI)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sim::{enemy::Enemy, player::Player};

    #[test]
    fn log_records_hp_loss_and_readout_shows_queue() {
        let mut f = Fight::new(Player::new(), Enemy::new([0.0, 0.0, 1.0]));
        let mut d = Debug::default();
        d.observe(&f);
        f.player_hp -= 10;
        f.enemy.hp -= 5;
        d.observe(&f);
        assert_eq!(d.log.len(), 2);
        assert!(d.log[0].contains("-10") && d.log[1].contains("-5"));
        for _ in 0..10 { f.player_hp -= 1; d.observe(&f); }
        assert_eq!(d.log.len(), LOG_LINES, "log is capped");
        let r = d.readout(&f, &Input { jump: true, lock: true, ..Default::default() });
        assert!(r.contains("queued  jump") && r.contains("lock-on held") && r.contains("frame   0/"));
        assert!(compass(&f, 0.0).unwrap().abs() < 1e-5, "enemy dead ahead");
        assert!((compass(&f, std::f32::consts::FRAC_PI_2).unwrap() + std::f32::consts::FRAC_PI_2).abs() < 1e-5, "camera turned right: enemy to the left");
        assert_ne!(eye_colour(Alert::Unaware), eye_colour(Alert::Alerted));
        f.enemy.hp = 0;
        assert!(compass(&f, 0.0).is_none());
        f.enemy.hp = 95;
        assert!((player_bar(&f) - 0.8).abs() < 1e-5 && (enemy_bar(&f) - 0.95).abs() < 1e-5);
    }
}
