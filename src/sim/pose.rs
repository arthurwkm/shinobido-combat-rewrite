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
        let mut s = Skeleton { parent: [None; NODES], offset: [[0.0; 3]; NODES] };
        for n in 0..NODES {
            let p = 4 + 16 * n;
            s.parent[n] = usize::try_from(u32_at(d, p)? as i32).ok().filter(|&q| q < NODES);
            s.offset[n] = [f32_at(d, p + 4)?, f32_at(d, p + 8)?, f32_at(d, p + 12)?];
        }
        Some(s)
    }

    /// World joint positions for one clip row, hips at the row's position, root at the origin.
    pub fn joints(&self, row: &[f32]) -> [[f32; 3]; NODES] {
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
        pos
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

fn rotate(q: [f32; 4], v: [f32; 3]) -> [f32; 3] {
    let u = [q[0], q[1], q[2]];
    let c = |a: [f32; 3], b: [f32; 3]| [a[1] * b[2] - a[2] * b[1], a[2] * b[0] - a[0] * b[2], a[0] * b[1] - a[1] * b[0]];
    let t = c(u, v).map(|x| 2.0 * x);
    let t2 = c(u, t);
    std::array::from_fn(|i| v[i] + q[3] * t[i] + t2[i])
}
