//! Shared engine-free terrain geometry. Metres, Y up; ramps rise along +Z.
use super::data::{BODY_HEIGHT, BODY_RADIUS, COLLISION_STEP, SUPPORT_EPS};

#[derive(Clone, Copy, Debug)]
pub struct Collider {
    pub min: [f32; 3],
    pub max: [f32; 3],
    pub ramp: bool,
}

impl Collider {
    pub fn new(min: [f32; 3], max: [f32; 3], ramp: bool) -> Self {
        assert!((0..3).all(|i| min[i].is_finite() && max[i].is_finite() && min[i] < max[i]));
        Self { min, max, ramp }
    }

    fn contains(&self, p: [f32; 3], margin: f32) -> bool {
        [0, 2].iter().all(|&i| p[i] > self.min[i] - margin && p[i] < self.max[i] + margin)
    }

    pub fn top(&self, z: f32) -> f32 {
        if self.ramp {
            self.min[1] + (self.max[1] - self.min[1]) * ((z - self.min[2]) / (self.max[2] - self.min[2])).clamp(0.0, 1.0)
        } else { self.max[1] }
    }
}

/// An infinite floor at Y=0 plus solid boxes and wedges; the viewer can use the same collider list.
#[derive(Default)]
pub struct Terrain {
    pub colliders: Vec<Collider>,
}

impl Terrain {
    pub fn ground_height(&self, p: [f32; 3]) -> f32 {
        self.colliders.iter().filter(|b| b.contains(p, 0.0))
            .map(|b| b.top(p[2])).filter(|&y| y <= p[1] + SUPPORT_EPS).fold(0.0, f32::max)
    }

    /// Nearest box face along a horizontal ray, with a reachable, unobstructed top.
    pub fn ledge(&self, p: [f32; 3], direction: [f32; 2], reach: f32) -> Option<[f32; 3]> {
        let length = direction[0].hypot(direction[1]);
        if length == 0.0 || !reach.is_finite() || reach <= 0.0 { return None; }
        let mut best = None;
        let mut distance = reach;
        for b in &self.colliders {
            if b.ramp || b.max[1] <= p[1] || b.max[1] > p[1] + reach { continue; }
            let mut near = 0.0_f32;
            let mut far = reach;
            for (i, d) in [(0, direction[0] / length), (2, direction[1] / length)] {
                if d.abs() < f32::EPSILON {
                    if p[i] < b.min[i] || p[i] > b.max[i] { far = -1.0; }
                } else {
                    let a = (b.min[i] - p[i]) / d;
                    let c = (b.max[i] - p[i]) / d;
                    near = near.max(a.min(c));
                    far = far.min(a.max(c));
                }
            }
            if near > 0.0 && near <= far && near <= distance {
                let top = [p[0] + direction[0] / length * near, b.max[1], p[2] + direction[1] / length * near];
                if self.colliders.iter().any(|other| other.contains(top, BODY_RADIUS)
                    && other.min[1] < top[1] + BODY_HEIGHT && other.top(top[2]) > top[1] + SUPPORT_EPS) { continue; }
                distance = near;
                best = Some(top);
            }
        }
        best
    }

    /// Resolve a feet-centred upright body; small swept steps prevent tunnelling and allow wall sliding.
    // ponytail: square horizontal bounds approximate the capsule; refine when corner motion is calibrated.
    pub fn move_body(&self, from: [f32; 3], to: [f32; 3], grounded: bool) -> ([f32; 3], Option<[f32; 3]>, bool) {
        let delta: [f32; 3] = std::array::from_fn(|i| to[i] - from[i]);
        let steps = (delta.iter().copied().map(f32::abs).fold(0.0, f32::max) / COLLISION_STEP).ceil().max(1.0) as usize;
        let mut p = from;
        let mut wall = None;
        let mut ceiling = false;
        for _ in 0..steps {
            let step_origin = p;
            for axis in [0, 2] {
                let old = p;
                p[axis] += delta[axis] / steps as f32;
                for b in &self.colliders {
                    if !b.contains(p, BODY_RADIUS) || p[1] + BODY_HEIGHT <= b.min[1] || p[1] >= b.top(p[2]) - SUPPORT_EPS { continue; }
                    // A supported body follows the ramp surface, but cannot enter its tall side.
                    if b.ramp && grounded && b.contains(p, 0.0) && (old[1] - b.top(old[2])).abs() <= SUPPORT_EPS {
                        p[1] = b.top(p[2]);
                    } else if delta[axis] != 0.0 {
                        // Allow leaving the expanded footprint after stepping off a top.
                        if b.contains(old, BODY_RADIUS)
                            && (old[axis] < b.min[axis] && delta[axis] < 0.0
                                || old[axis] > b.max[axis] && delta[axis] > 0.0) { continue; }
                        p[axis] = old[axis];
                        let mut normal = [0.0; 3];
                        normal[axis] = -delta[axis].signum();
                        wall = Some(normal);
                    }
                }
            }
            let old_y = p[1];
            p[1] += delta[1] / steps as f32;
            for b in &self.colliders {
                if !b.contains(p, 0.0) { continue; }
                let top = b.top(p[2]);
                if old_y >= top - SUPPORT_EPS && p[1] < top { p[1] = top; }
                if old_y + BODY_HEIGHT <= b.min[1] + SUPPORT_EPS && p[1] + BODY_HEIGHT > b.min[1] {
                    p[1] = b.min[1] - BODY_HEIGHT;
                    ceiling = true;
                }
            }
            p[1] = p[1].max(0.0);
            // Descending ramps retain support without snapping down box edges.
            if grounded && delta[1] <= 0.0 {
                for b in &self.colliders {
                    if b.ramp && b.contains(p, 0.0) && (step_origin[1] - b.top(step_origin[2])).abs() <= SUPPORT_EPS { p[1] = b.top(p[2]); }
                }
            }
        }
        (p, wall, ceiling)
    }
}

/// Ramps steeper than this (rise over run) are drawn as stairs by the viewer; the sim still sees a ramp.
pub const STAIRS_SLOPE: f32 = 0.3;

impl Terrain {
    /// The viewer sandbox: tall walls with a corner, a tiled-roof building, a slope, stairs,
    /// a narrow corridor and crates. Sizes are ESTIMATE (from the clips, not the game files).
    pub fn sandbox() -> Self {
        let b = |min, max| Collider::new(min, max, false);
        let colliders = vec![
            b([-10.0, 0.0, 8.0], [-4.0, 6.0, 8.5]),      // tall wall
            b([-4.5, 0.0, 8.0], [-4.0, 6.0, 14.0]),      // returns at a corner to peek around
            b([6.0, 0.0, 6.0], [14.0, 3.5, 14.0]),       // rooftop, high-jump ledge
            Collider::new([-3.0, 0.0, -10.0], [3.0, 1.0, -4.0], true), // slope
            Collider::new([8.0, 0.0, -12.0], [10.0, 2.4, -4.0], true), // stairs
            b([-21.0, 0.0, -20.0], [-20.5, 4.0, -4.0]),  // corridor, 3 m wide
            b([-17.5, 0.0, -20.0], [-17.0, 4.0, -4.0]),
            b([2.0, 0.0, 3.0], [3.0, 1.0, 4.0]),         // crates
            b([3.2, 0.0, 3.0], [4.2, 1.0, 4.0]),
            b([2.6, 1.0, 3.0], [3.6, 2.0, 4.0]),
        ];
        Self { colliders }
    }
}
