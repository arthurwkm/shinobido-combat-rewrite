//! Game skeleton and sampled clips (written by tools/pose.py), and forward kinematics. Engine-free: the viewer
//! only draws the joint positions returned here.

pub const NODES: usize = 28;
/// Floats per frame: hips position (3) then 21 local quaternions (x y z w) for node ids 0..20.
const ROW: usize = 87;
const ROOT: usize = 23;

pub struct Skeleton {
    /// Parent node id per node id (`None` for the root, id 23).
    pub parent: [Option<usize>; NODES],
    /// Rest offset in the parent's frame, metres.
    pub offset: [[f32; 3]; NODES],
    /// Head mesh bounds local to node 10, including hair, metres.
    pub head_min: [f32; 3],
    pub head_max: [f32; 3],
}

pub struct Clips {
    clips: Vec<(String, usize, Vec<f32>)>,
}

fn u32_at(d: &[u8], p: usize) -> Option<u32> {
    Some(u32::from_le_bytes(d.get(p..p + 4)?.try_into().ok()?))
}

fn f32_at(d: &[u8], p: usize) -> Option<f32> {
    Some(f32::from_bits(u32_at(d, p)?))
}

impl Skeleton {
    pub fn parse(d: &[u8]) -> Option<Skeleton> {
        if u32_at(d, 0)? as usize != NODES {
            return None;
        }
        let mut s = Skeleton { parent: [None; NODES], offset: [[0.0; 3]; NODES], head_min: [0.0; 3], head_max: [0.0; 3] };
        for n in 0..NODES {
            let p = 4 + 16 * n;
            s.parent[n] = usize::try_from(u32_at(d, p)? as i32).ok().filter(|&q| q < NODES);
            s.offset[n] = [f32_at(d, p + 4)?, f32_at(d, p + 8)?, f32_at(d, p + 12)?];
        }
        for k in 0..3 {
            s.head_min[k] = f32_at(d, 4 + 16 * NODES + 4 * k)?;
            s.head_max[k] = f32_at(d, 4 + 16 * NODES + 12 + 4 * k)?;
            if !s.head_min[k].is_finite() || !s.head_max[k].is_finite() || s.head_min[k] > s.head_max[k] {
                return None;
            }
        }
        Some(s)
    }

    /// World joint positions for one clip row, hips at the row's position, root at the origin.
    pub fn joints(&self, row: &[f32]) -> [[f32; 3]; NODES] {
        self.pose(row).0
    }

    /// Joint positions and absolute rotations, for parts attached to a joint rather than a segment.
    pub fn pose(&self, row: &[f32]) -> ([[f32; 3]; NODES], [[f32; 4]; NODES]) {
        let mut pos = [[0.0; 3]; NODES];
        let mut rot = [[0.0, 0.0, 0.0, 1.0]; NODES];
        // Parents come before children when walking up from each node; resolve by repeated passes (tree depth < 12).
        let mut done = [false; NODES];
        pos[ROOT] = [0.0; 3];
        done[ROOT] = true;
        for _ in 0..NODES {
            for n in 0..NODES {
                let Some(p) = self.parent[n] else { continue };
                if done[n] || !done[p] {
                    continue;
                }
                let local = if n < 21 { [row[3 + 4 * n], row[4 + 4 * n], row[5 + 4 * n], row[6 + 4 * n]] } else { [0.0, 0.0, 0.0, 1.0] };
                rot[n] = qmul(rot[p], local);
                let o = rotate(rot[p], self.offset[n]);
                pos[n] = if n == 0 { [row[0], row[1], row[2]] } else { [pos[p][0] + o[0], pos[p][1] + o[1], pos[p][2] + o[2]] };
                done[n] = true;
            }
        }
        (pos, rot)
    }
}

impl Clips {
    pub fn parse(d: &[u8]) -> Option<Clips> {
        let n = u32_at(d, 0)? as usize;
        let mut p = 4;
        let mut clips = Vec::with_capacity(n);
        for _ in 0..n {
            let name = std::str::from_utf8(d.get(p..p + 24)?).ok()?.trim_end_matches('\0').to_string();
            let frames = u32_at(d, p + 24)? as usize;
            p += 28;
            let data = (0..frames * ROW).map(|i| f32_at(d, p + 4 * i)).collect::<Option<Vec<_>>>()?;
            p += 4 * frames * ROW;
            clips.push((name, frames, data));
        }
        Some(Clips { clips })
    }

    /// The row for `name` at `frame` (held on the last frame); `None` for clips that were not exported.
    pub fn row(&self, name: &str, frame: u32) -> Option<&[f32]> {
        let (_, frames, data) = self.clips.iter().find(|c| c.0 == name)?;
        let f = (frame as usize).min(frames.checked_sub(1)?);
        Some(&data[f * ROW..(f + 1) * ROW])
    }
}

fn qmul(a: [f32; 4], b: [f32; 4]) -> [f32; 4] {
    [
        a[3] * b[0] + a[0] * b[3] + a[1] * b[2] - a[2] * b[1],
        a[3] * b[1] - a[0] * b[2] + a[1] * b[3] + a[2] * b[0],
        a[3] * b[2] + a[0] * b[1] - a[1] * b[0] + a[2] * b[3],
        a[3] * b[3] - a[0] * b[0] - a[1] * b[1] - a[2] * b[2],
    ]
}

pub(super) fn rotate(q: [f32; 4], v: [f32; 3]) -> [f32; 3] {
    let u = [q[0], q[1], q[2]];
    let c = |a: [f32; 3], b: [f32; 3]| [a[1] * b[2] - a[2] * b[1], a[2] * b[0] - a[0] * b[2], a[0] * b[1] - a[1] * b[0]];
    let t = c(u, v).map(|x| 2.0 * x);
    let t2 = c(u, t);
    std::array::from_fn(|i| v[i] + q[3] * t[i] + t2[i])
}

/// Seconds to cross-fade into a new clip: 6 frames, the most common blend in the transition events (type-3 arg1
/// high half is 6 or 10; formats.md "Transition-category bits"). Without it every clip change snaps.
pub const BLEND_S: f32 = 6.0 / 60.0;

/// Cross-fades a character's displayed pose from whatever was on screen into a newly started clip.
#[derive(Default)]
pub struct Fade {
    clip: &'static str,
    from: Vec<f32>,
    shown: Vec<f32>,
    t: f32,
}

impl Fade {
    /// The row to draw this render frame for `row` of `clip`; `dt` is elapsed render (or record-tick) time.
    pub fn update(&mut self, clip: &'static str, row: &[f32], dt: f32) -> &[f32] {
        if clip != self.clip {
            self.from = if self.shown.len() == row.len() { self.shown.clone() } else { row.to_vec() };
            self.clip = clip;
            self.t = 0.0;
        }
        self.t = (self.t + dt).min(BLEND_S);
        let k = self.t / BLEND_S;
        self.shown = blend(&self.from, row, k);
        &self.shown
    }
}

/// Hips position lerped, quaternions nlerped along the shorter arc.
fn blend(a: &[f32], b: &[f32], k: f32) -> Vec<f32> {
    let mut out: Vec<f32> = a.iter().zip(b).map(|(x, y)| x + (y - x) * k).collect();
    for q in (3..a.len()).step_by(4) {
        let dot: f32 = (0..4).map(|i| a[q + i] * b[q + i]).sum();
        let s = if dot < 0.0 { -1.0 } else { 1.0 };
        let v: [f32; 4] = std::array::from_fn(|i| a[q + i] * (1.0 - k) + s * b[q + i] * k);
        let n = v.iter().map(|x| x * x).sum::<f32>().sqrt().max(1e-6);
        out[q..q + 4].copy_from_slice(&v.map(|x| x / n));
    }
    out
}

#[cfg(test)]
mod fade_tests {
    use super::*;

    #[test]
    fn clip_change_fades_over_blend_time_then_follows_the_clip() {
        let mut f = Fade::default();
        let a = [0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0];
        let b = [1.0, 1.0, 0.0, 0.0, 0.0, 1.0, 0.0];
        assert_eq!(f.update("A", &a, 1.0 / 60.0), a.as_slice(), "first clip shows as is");
        let mid = f.update("B", &b, BLEND_S / 2.0).to_vec();
        assert!((mid[0] - 0.5).abs() < 1e-4, "hips halfway: {}", mid[0]);
        let n: f32 = mid[3..7].iter().map(|x| x * x).sum();
        assert!((n - 1.0).abs() < 1e-4, "quaternion stays unit");
        assert_eq!(f.update("B", &b, BLEND_S), b.as_slice(), "fully in after the blend time");
    }
}
