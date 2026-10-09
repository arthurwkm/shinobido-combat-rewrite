use super::{extracted::ACTIONS, World};

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum State {
    Idle,
    Run,
    /// WALK_F02 loop (suffix-02 set; hypothesis: slow stick tilt).
    Walk,
    Halt,
    /// L1 held: STOP_S05 idle, WALK_F05 moving (set 05 `_S` variants).
    Crouch,
    /// Back to a tall wall: set-05 idle, lateral root motion and directional corner peek.
    Hug,
    Attack,
    /// GU_A01 then GU_B01 while held.
    Guard,
    /// GU_Y01 (hypothesis: guard release).
    GuardEnd,
    /// FRONT01 / BACK01 / RIGHT01 / LEFT01 (hypothesis: evasive steps; 30 frames, 1.5-1.6 m of root motion each).
    Dodge,
    /// JUMP01 animation; height from the game's gravity update (jump launch velocity and unit scale are ESTIMATE).
    Jump,
    /// FALL01 loop while off the ground without a jump (horizontal speed carried from the last grounded frame: ESTIMATE).
    Fall,
    /// LAND01 (LAND_B01 / LAND_A01 after longer falls) after touching down.
    Land,
    /// A HI_*01 hit reaction (hypothesis: B/U/C letters are the hit side or height; which one the game picks is ESTIMATE).
    Hit,
    /// One-shot ninja action (SHEATH01, PICK01, THROW_A01/B01, DRINK01, NINJYUTU01, MAKIBISHI01); role per name is a hypothesis.
    Special,
    /// KABE_U01: running up a tall wall (jump while running into it); height gain is ESTIMATE (data.rs).
    WallRun,
    /// TRI_J01 / KABE_E01: pushed off the wall, falling away from it with gravity.
    WallKick,
    /// TWO06 catch, then STOP_A06 / HANG_L06 / HANG_R06 hanging from a ledge (jump climbs, crouch drops).
    Hang,
    /// CLIMB06 onto the ledge top.
    Climb,
    /// CLIMB_B01: from the ground over a crate-high box (VAULT_MIN..VAULT_MAX), landing beyond it or on its top.
    Vault,
    /// KICK_J01 / KICK_JB01 / FLIP_B01 played in the air: gravity and momentum go on, FALL01 follows.
    AirAct,
    /// STOP_K05 / RUN_K05 with a crate or body in the arms; only put-down and throw are open.
    Carry,
}

/// Gravity update from the executable (0x1102b0, formats.md), in game units.
fn gravity(vy: f32) -> f32 {
    let m = vy.abs();
    if m > 15.0 { -450.0 } else { -100.0 - 0.4 * m }
}
/// Launch velocity in game units/s: the impulse Y float 120.0 at 0x604a28 (formats.md "Wall ascent and impulse").
/// Emulator A/B 2026-10-09 (`tools/scripts/emu/jump.txt`): a standing jump is airborne 18 frames at 25 fps (0.72 s)
/// with a ~0.3 s hang at the top; this and `gravity` give 0.76 s and the same hang.
const JUMP_V0: f32 = 120.0;
/// Metres per game unit = model units (hips 9.5) per motion metres (hips 0.84), formats.md. Gives a 1.49 m apex,
/// matching the emulator's jump (feet about his own height up); the old 9.8/450 guess gave 0.57 m.
const UNIT_M: f32 = 0.84 / 9.5;
/// JUMP01 event (5, 7, ...) marks take-off.
const TAKEOFF_FRAME: u32 = 5;
/// ESTIMATE: frames after pressing Jump in which a back flick of the stick starts FLIP_B01.
const FLIP_WINDOW: u32 = 20;
/// KABE_U01 length in frames (extracted.rs).
const ACTIONS_KABE_U_FRAMES: f32 = 36.0;

/// Ground turn rate, rad/s (2.5 pi). From the executable's turn function 0x10bf80, default-state branch (0x10c1f4);
/// other states use other rates (formats.md). Which branch applies to which action is not decoded: locomotion only.
pub const TURN_RATE: f32 = 7.853_981_6;

/// Idle / run / halt on top of `World`. ESTIMATE: the transitions (release while running plays HALT01 once, then
/// idle; pressing forward during HALT01 interrupts it) are guessed; the motions and root motion are from the files.
pub struct Player {
    world: World,
    pub state: State,
    /// Guard button held. Guard is only entered from idle/run/halt, not mid-attack (ESTIMATE).
    pub guard_held: bool,
    /// Crouch button held (L1); only from free locomotion.
    pub crouch_held: bool,
    /// Gentle forward input: walk instead of run (ESTIMATE: the game's analog threshold is not decoded).
    pub walk: bool,
    /// Clip used while walking: WALK_F02 (default), WALK_B02, WALK_R02 or WALK_L02 (ESTIMATE: picking by input direction).
    pub walk_clip: &'static str,
    /// RUN01, RUN_R01 or RUN_L01; selection policy is ESTIMATE (data.rs).
    pub run_clip: &'static str,
    /// STOP01, STOP02 or TIRED01; selection policy is ESTIMATE (data.rs).
    pub idle_clip: &'static str,
    /// Katana out: idle uses STOP02 (the drawn set, video_notes 56 s), sheathing back plays SHEATH01. Set 01 = sheathed is an ESTIMATE.
    pub drawn: bool,
    /// Metres since creation, in the character's frame (+z forward).
    pub pos: [f32; 3],
    /// Yaw in radians; forward is (sin h, cos h) in x/z. `pos` is in the world frame.
    pub heading: f32,
    /// Direction the stick asks for (radians); the heading turns toward it at `TURN_RATE`.
    pub target_heading: f32,
    pub wall_normal: Option<[f32; 3]>,
    /// The wall in contact (or behind, while hugging) is at least `HUG_WALL_MIN` tall.
    wall_tall: bool,
    /// Sign of the side-step that reached a wall end (0 when none): peeking blocks further movement that way.
    hug_edge: f32,
    hug_dir: f32,
    vy: f32,
    fall_from: f32,
    air_vel: [f32; 2],
    /// Top height of the ledge being hung from.
    hang_top: f32,
    /// Set by dropping from a ledge, cleared on the ground: no instant re-catch.
    grab_lock: bool,
    /// Height above the feet of the box top in front, when the body is grounded and close enough to vault it.
    vault_top: Option<f32>,
    vault_base: f32,
    vault_h: f32,
    /// Heading at take-off / fall start: a stick turned away from it in the air is the back flick.
    air_heading: f32,
    /// Direction/modifier held with the attack button (set by `Input`); picks the attack, see `press_attack`.
    pub attack_mod: AttackMod,
    /// A load is in the arms (PICK01 plays, then `State::Carry`); cleared by a hit or by the put/throw.
    pub carrying: bool,
}

/// Modifier held with Square (docs/controls.md "Attack and move list").
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum AttackMod {
    #[default]
    Plain,
    Forward,
    Back,
    /// R1 held: rising uppercut.
    Lock,
    /// Circle held while running: leap forward and slash.
    Dash,
}

impl Default for Player {
    fn default() -> Self {
        Self::new()
    }
}

impl Player {
    pub fn new() -> Self {
        Player { world: World::new("STOP01"), state: State::Idle, guard_held: false, crouch_held: false, walk: false, walk_clip: "WALK_F02", run_clip: "RUN01", idle_clip: "STOP01", drawn: false, pos: [0.0; 3], heading: 0.0, target_heading: 0.0, wall_normal: None, wall_tall: false, hug_edge: 0.0, hug_dir: 0.0, vy: 0.0, fall_from: 0.0, air_vel: [0.0; 2], hang_top: 0.0, grab_lock: false, vault_top: None, vault_base: 0.0, vault_h: 0.0, air_heading: 0.0, attack_mod: AttackMod::Plain, carrying: false }
    }

    /// `attack` is true on the frame the button goes down.
    pub fn step(&mut self, forward: bool, attack: bool) {
        self.step_on(&super::terrain::Terrain::default(), forward, attack);
    }

    /// Advance on shared terrain. Leaving the ground without a jump plays FALL01, then a landing clip by drop height.
    pub fn step_on(&mut self, terrain: &super::terrain::Terrain, forward: bool, attack: bool) {
        let before = self.pos;
        let (sh, ch) = self.heading.sin_cos();
        self.wall_tall = match if self.state == State::Hug { Some([sh, 0.0, ch]) } else { self.wall_normal } {
            Some(n) => terrain.wall_behind(before, n, super::data::HUG_WALL_MIN),
            None => false,
        };
        let grounded = (before[1] - terrain.ground_height(before)).abs() <= super::data::SUPPORT_EPS;
        if grounded { self.grab_lock = false; }
        self.vault_top = terrain.ledge(before, [sh, ch], super::data::VAULT_MAX)
            .filter(|t| grounded && (t[0] - before[0]).hypot(t[2] - before[2]) <= super::data::VAULT_DIST)
            .map(|t| t[1] - before[1])
            .filter(|&dy| dy >= super::data::VAULT_MIN);
        if matches!(self.state, State::Hang | State::Climb | State::Vault) {
            return self.step_ledge(terrain, forward);
        }
        self.try_grab(terrain);
        if self.state == State::Hang { return; }
        self.step_action(forward, attack);
        let jumping = self.state == State::Jump && self.world.frame >= TAKEOFF_FRAME;
        if !grounded && !jumping && matches!(self.state, State::Idle | State::Run | State::Walk | State::Halt | State::Crouch) {
            self.air_vel = [self.pos[0] - before[0], self.pos[2] - before[2]];
            self.fall_from = before[1];
            self.air_heading = self.heading;
            self.start(State::Fall, "FALL01");
        } else if matches!(self.state, State::Jump | State::Fall | State::AirAct) {
            // Momentum from the last grounded tick; a held stick steers along the heading (ESTIMATE, data.rs).
            if forward && self.state != State::AirAct {
                let speed = self.air_vel[0].hypot(self.air_vel[1]).max(super::data::AIR_SPEED);
                let (s, c) = self.heading.sin_cos();
                self.air_vel = [s * speed, c * speed];
            }
            self.pos[0] += self.air_vel[0];
            self.pos[2] += self.air_vel[1];
        }
        // Special clips (climbs) own the height through root motion.
        let rooted = matches!(self.state, State::Special | State::WallRun);
        if !jumping && !grounded && !rooted {
            self.vy += gravity(self.vy) * super::DT;
            self.pos[1] += self.vy * super::DT * UNIT_M;
        }
        let (pos, wall, ceiling) = terrain.move_body(before, self.pos, grounded && !jumping);
        if !matches!(self.state, State::Jump | State::Fall) {
            self.air_vel = [pos[0] - before[0], pos[2] - before[2]];
        }
        self.pos = pos;
        self.wall_normal = wall;
        if ceiling { self.vy = 0.0; }
        if !rooted && self.vy <= 0.0 && (pos[1] - terrain.ground_height(pos)).abs() <= super::data::SUPPORT_EPS {
            self.vy = 0.0;
            if jumping || !grounded {
                let drop = if matches!(self.state, State::Fall | State::AirAct) { self.fall_from - pos[1] } else { 0.0 };
                let clip = if drop >= super::data::LAND_A_DROP { "LAND_A01" } else if drop >= super::data::LAND_B_DROP { "LAND_B01" } else { "LAND01" };
                self.start(State::Land, clip);
            }
        }
    }

    fn step_action(&mut self, forward: bool, attack: bool) {
        // Keep caller-selected clips within their supported locomotion families.
        if !super::data::RUN_CLIPS.contains(&self.run_clip) { self.run_clip = "RUN01"; }
        if !super::data::IDLE_CLIPS.contains(&self.idle_clip) { self.idle_clip = "STOP01"; }
        if attack {
            self.press_attack();
        }
        if matches!(self.state, State::Idle | State::Run | State::Walk | State::Halt | State::Crouch | State::Jump | State::Fall | State::Carry) {
            let d = (self.target_heading - self.heading + std::f32::consts::PI).rem_euclid(std::f32::consts::TAU)
                - std::f32::consts::PI;
            let max = TURN_RATE * super::DT;
            self.heading += d.clamp(-max, max);
        }
        match (self.state, self.guard_held) {
            (State::Idle | State::Run | State::Walk | State::Halt | State::Crouch, true) => self.start(State::Guard, "GU_A01"),
            (State::Guard, false) => self.start(State::GuardEnd, "GU_Y01"),
            (State::Guard, true) if self.world.done() && self.world.action.name == "GU_B01" => {
                self.start(State::Guard, "GU_B01") // loops while held
            }
            (State::Guard, true) if self.world.action.name == "GU_A01" => {
                // GU_A01's cancel window (event type 3) opens the move to GU_B01
                let f = self.world.frame;
                let to = self.world.action.cancels.iter().find(|c| c.1 <= f).and_then(|c| ACTIONS.iter().find(|x| x.id == c.0));
                if let Some(t) = to {
                    self.start(State::Guard, t.name);
                }
            }
            (State::GuardEnd, _) if self.world.done() => self.start(State::Idle, self.idle_clip),
            _ => {}
        }
        if self.state == State::Jump && self.world.frame < FLIP_WINDOW && self.air_back() {
            self.start(State::AirAct, "FLIP_B01");
            self.fall_from = self.pos[1];
        }
        if self.state == State::AirAct {
            if self.world.done() { self.start(State::Fall, "FALL01"); } else { self.advance(); }
            return;
        }
        if self.state == State::Jump {
            self.advance();
            if self.world.frame >= TAKEOFF_FRAME {
                if self.world.frame == TAKEOFF_FRAME {
                    self.vy = JUMP_V0;
                }
                self.vy += gravity(self.vy) * super::DT;
                self.pos[1] += self.vy * super::DT * UNIT_M;
            }
            return;
        }
        if self.state == State::Fall {
            if self.world.done() { self.start(State::Fall, "FALL01"); }
            return self.advance();
        }
        if self.state == State::WallRun {
            self.pos[1] += super::data::WALL_RUN_RISE / ACTIONS_KABE_U_FRAMES;
            if self.world.done() { self.wall_kick("KABE_E01"); } else { self.advance(); }
            return;
        }
        if self.state == State::WallKick {
            if self.world.done() { self.start(State::Fall, "FALL01"); } else { self.advance(); }
            self.pos[0] += self.air_vel[0];
            self.pos[2] += self.air_vel[1];
            return;
        }
        if self.state == State::Land {
            if self.world.done() {
                self.start(State::Idle, self.idle_clip);
            }
            return self.advance();
        }
        if self.state == State::Dodge && self.world.done() {
            self.start(State::Idle, self.idle_clip);
        }
        if matches!(self.state, State::Hit | State::Special) && self.world.done() {
            if self.carrying && self.state == State::Special {
                self.start(State::Carry, "STOP_K05");
            } else {
                self.start(State::Idle, self.idle_clip);
            }
        }
        if matches!(self.state, State::Guard | State::GuardEnd | State::Dodge | State::Hit | State::Special) {
            return self.advance();
        }
        if self.state == State::Carry {
            let clip = if forward { "RUN_K05" } else { "STOP_K05" };
            if self.world.action.name != clip {
                self.start(State::Carry, clip);
            } else if self.world.done() {
                self.world = World::new(clip);
            }
            return self.advance();
        }
        if self.state == State::Hug {
            return self.step_hug(forward);
        }
        if self.state == State::Attack {
            if self.world.done() {
                self.start(State::Idle, self.idle_clip);
            }
            return self.advance();
        }
        if let (true, true, true, Some(n)) = (self.crouch_held, forward, self.wall_tall, self.wall_normal) {
            let (s, c) = self.target_heading.sin_cos();
            if -(s * n[0] + c * n[2]) > 0.7 {
                self.heading = n[0].atan2(n[2]);
                self.hug_edge = 0.0;
                self.hug_dir = 0.0;
                self.start(State::Hug, "STOP_H05");
                return self.advance();
            }
        }
        if self.crouch_held && !self.guard_held {
            let clip = if forward { "WALK_F05" } else { "STOP_S05" };
            if self.state != State::Crouch || self.world.action.name != clip {
                self.start(State::Crouch, clip);
            } else if self.world.done() {
                self.world = World::new(clip);
            }
            return self.advance();
        }
        if self.state == State::Crouch {
            self.start(State::Idle, self.idle_clip);
        }
        let loco = if self.walk { State::Walk } else { State::Run };
        let next = match (self.state, forward) {
            (State::Run | State::Walk, false) => Some(State::Halt),
            (State::Idle | State::Halt, true) => Some(loco),
            (State::Idle, false) if self.world.action.name != self.idle_clip => Some(State::Idle),
            (State::Run, true) if !self.walk && self.world.action.name != self.run_clip => Some(State::Run),
            (State::Walk, true) if self.walk && self.world.action.name != self.walk_clip => Some(State::Walk),
            (State::Run | State::Walk, true) if self.state != loco => Some(loco),
            (State::Halt, false) if self.world.done() => Some(State::Idle),
            _ => None,
        };
        let walk_clip = self.walk_clip;
        let run_clip = self.run_clip;
        let idle_clip = self.idle_clip;
        let clip = |s| match s {
            State::Run => run_clip,
            State::Walk => walk_clip,
            State::Halt => "HALT01",
            _ => idle_clip,
        };
        if let Some(s) = next {
            self.start(s, clip(s));
        } else if self.world.done() && self.state != State::Halt {
            self.world = World::new(clip(self.state));
        }
        self.advance();
    }

    /// Catch a ledge in reach while rising or falling (Jump after take-off, Fall): TWO06, body hung below the top.
    fn try_grab(&mut self, terrain: &super::terrain::Terrain) {
        use super::data::{HANG_MIN, HANG_OFFSET, LEDGE_GRAB_DIST, LEDGE_REACH};
        let airborne = matches!(self.state, State::Fall | State::WallRun) || self.state == State::Jump && self.world.frame >= TAKEOFF_FRAME;
        if !airborne || self.grab_lock || self.crouch_held { return; }
        let (s, c) = self.heading.sin_cos();
        let Some(top) = terrain.ledge(self.pos, [s, c], LEDGE_REACH) else { return };
        let dy = top[1] - self.pos[1];
        if dy < HANG_MIN || (top[0] - self.pos[0]).hypot(top[2] - self.pos[2]) > LEDGE_GRAB_DIST { return; }
        let rise = ACTIONS.iter().find(|a| a.name == "CLIMB06").map_or(2.3, |a| a.root.last().map_or(2.3, |k| k.1[1]));
        let hung = [top[0] - s * HANG_OFFSET, top[1] - rise, top[2] - c * HANG_OFFSET];
        if hung[1] < terrain.ground_height(hung) { return; } // too low to hang: CLIMB06 starts 2.3 m under the top
        self.hang_top = top[1];
        self.pos = hung;
        self.vy = 0.0;
        self.air_vel = [0.0; 2];
        self.start(State::Hang, "TWO06");
    }

    /// Hanging and climbing own the body (no gravity or collision). Stick sideways shimmies while the ledge continues.
    fn step_ledge(&mut self, terrain: &super::terrain::Terrain, forward: bool) {
        if self.state == State::Vault {
            self.advance();
            // The clip's hips carry the rise (1.8 m, then down to the far side), so the body only follows the ground.
            if self.world.done() {
                self.pos[1] = terrain.ground_height([self.pos[0], self.vault_base + self.vault_h, self.pos[2]]);
                self.start(State::Idle, self.idle_clip);
            }
            return;
        }
        if self.state == State::Climb {
            self.advance();
            if self.world.done() {
                self.pos[1] = self.hang_top;
                self.start(State::Idle, self.idle_clip);
            }
            return;
        }
        if self.crouch_held {
            self.grab_lock = true;
            self.fall_from = self.pos[1];
            self.start(State::Fall, "FALL01");
            return;
        }
        let d = self.target_heading - self.heading;
        let side = if forward && d.sin().abs() > 0.5 { d.sin().signum() } else { 0.0 };
        let name = self.world.action.name;
        let clip = if side < 0.0 { "HANG_R06" } else if side > 0.0 { "HANG_L06" } else if name == "TWO06" && !self.world.done() { "TWO06" } else { "STOP_A06" };
        if name != clip || self.world.done() {
            self.start(State::Hang, clip);
        }
        let before = self.pos;
        self.advance();
        // The shimmy ends where the ledge does.
        let (s, c) = self.heading.sin_cos();
        let hand = [self.pos[0], self.hang_top - 1.0, self.pos[2]];
        if terrain.ledge(hand, [s, c], 1.0).is_none_or(|t| (t[1] - self.hang_top).abs() > 0.01) {
            self.pos = before;
        }
    }

    /// Set-05 lateral root motion; a wall end plays LOOK then holds the peek until the stick returns.
    fn step_hug(&mut self, forward: bool) {
        let name = self.world.action.name;
        let d = self.target_heading - self.heading;
        let (side, away) = if forward { (d.sin(), d.cos() > 0.7) } else { (0.0, false) };
        let side = if side.abs() > 0.5 { side.signum() } else { 0.0 };
        if !self.crouch_held || away {
            self.start(State::Idle, self.idle_clip);
            return self.advance();
        }
        if matches!(name, "BACK_RH05" | "BACK_LH05") && !self.world.done() {
            return self.advance();
        }
        if !self.wall_tall && self.hug_edge == 0.0 {
            self.hug_edge = self.hug_dir;
        }
        if self.wall_tall { self.hug_edge = 0.0; }
        let moving = side != 0.0 && side != self.hug_edge;
        if moving && matches!(name, "LOOK_RH05" | "LOOK_LH05" | "STOPLR05" | "STOPLL05") {
            self.start(State::Hug, if self.hug_dir < 0.0 { "BACK_RH05" } else { "BACK_LH05" });
            return self.advance();
        }
        if moving { self.hug_dir = side; }
        let right = self.hug_dir < 0.0; // RH translates along local -X in the source tracks.
        let clip = if moving {
            if right { "HUG_RH05" } else { "HUG_LH05" }
        } else if self.hug_edge != 0.0 {
            if matches!(name, "STOPLR05" | "STOPLL05") || matches!(name, "LOOK_RH05" | "LOOK_LH05") && self.world.done() {
                if right { "STOPLR05" } else { "STOPLL05" }
            } else if right { "LOOK_RH05" } else { "LOOK_LH05" }
        } else if self.hug_dir == 0.0 { "STOP_H05" }
        else if right { "STOP_RH05" } else { "STOP_LH05" };
        if name != clip || self.world.done() {
            self.start(State::Hug, clip);
        }
        self.advance();
    }

    /// True while the current attack clip's hit window is open (event types 1/2).
    /// Where the camera looks: the body, lifted with the hips while vaulting (CLIMB_B01 hips rise ~1.8 m over frames 25-60 and fall over 72-100: ESTIMATE from the pose export).
    pub fn focus_pos(&self) -> [f32; 3] {
        let mut p = self.pos;
        if self.state == State::Vault {
            let ramp = |a: f32, b: f32| ((self.world.frame as f32 - a) / (b - a)).clamp(0.0, 1.0);
            p[1] += 1.8 * (ramp(25.0, 60.0) - ramp(72.0, 100.0));
        }
        p
    }

    pub fn attack_active(&self) -> bool {
        self.state == State::Attack && self.world.action.hits.iter().any(|h| (h.0..h.1).contains(&self.world.frame))
    }

    /// Damage argument of the current attack clip's first type-35 event, if any.
    pub fn attack_arg(&self) -> Option<u32> {
        self.world.action.events.iter().find(|e| e.1 == 35).map(|e| e.2)
    }

    pub fn world_action(&self) -> &'static str {
        self.world.action.name
    }

    /// Length in frames of the current clip.
    pub fn world_len(&self) -> u32 {
        self.world.action.frames
    }

    /// Frame of the current clip (for posing the rig).
    pub fn world_frame(&self) -> u32 {
        self.world.frame
    }

    /// Start an evasive step from idle/run/halt. `dir`: "FRONT01", "BACK01", "RIGHT01" or "LEFT01".
    pub fn dodge(&mut self, dir: &str) {
        self.request(State::Dodge, dir);
    }

    /// Take a hit: interrupts any state and plays the given reaction (`HI_BS01`, `HI_US01`, ...) to its end.
    pub fn hit(&mut self, reaction: &str) {
        self.carrying = false; // whatever he held is dropped (Fight notices)
        self.vy = 0.0;
        self.start(State::Hit, reaction);
    }

    /// Also used for RUN_RE01/RUN_LE01 and STOP_K01 until wall-state selection is implemented.
    /// Start a one-shot ninja action from idle/run/walk/halt; it plays to its end, then returns to idle (ESTIMATE: no cancels).
    pub fn special(&mut self, name: &str) {
        self.request(State::Special, name);
    }

    /// Follow whether the blade should be out: drawing is instant (the swing itself draws it), sheathing waits for
    /// standing idle and plays SHEATH01.
    pub fn update_drawn(&mut self, want: bool) {
        if want {
            self.drawn = true;
        } else if self.drawn && self.state == State::Idle && self.request(State::Special, "SHEATH01") {
            self.drawn = false;
        }
        if self.state == State::Idle || self.state == State::Special {
            self.idle_clip = if self.drawn { "STOP02" } else { "STOP01" };
        }
    }

    /// Play a paired clip now, whatever the player is doing (grab kills, executions).
    pub fn start_special(&mut self, name: &str) {
        self.vy = 0.0;
        self.start(State::Special, name);
    }

    /// Drop straight down (the from-above kill's approach): FALL01 with no carried momentum.
    pub fn dive(&mut self) {
        self.air_vel = [0.0; 2];
        self.fall_from = self.pos[1];
        self.vy = 0.0;
        self.start(State::Fall, "FALL01");
    }

    /// Reach for a load: PICK01 from free locomotion, then `State::Carry` once it ends.
    pub fn begin_pick(&mut self) -> bool {
        self.carrying = self.request(State::Special, "PICK01");
        self.carrying
    }

    /// Put down or throw what is carried: play `clip` (PUT_K05 / NAGE_K05), then back to idle.
    pub fn unload(&mut self, clip: &str) -> bool {
        if self.state != State::Carry {
            return false;
        }
        self.carrying = false;
        self.start(State::Special, clip);
        true
    }

    /// Start a dodge now from the guard stance too (the roll into the enemy is R1 + Forward + Cross, R1 is the guard).
    pub fn start_dodge(&mut self, name: &str) -> bool {
        if matches!(self.state, State::Guard | State::GuardEnd) {
            self.start(State::Dodge, name);
            return true;
        }
        self.request(State::Dodge, name)
    }

    /// The stick points away from the heading the jump or fall started with (the quick back flick).
    fn air_back(&self) -> bool {
        let d = (self.target_heading - self.air_heading + std::f32::consts::PI).rem_euclid(std::f32::consts::TAU) - std::f32::consts::PI;
        d.abs() > super::data::AIR_BACK_ANGLE
    }

    /// Jump. Running into a tall wall (wall_tall, facing it) starts the wall run instead; a second press kicks off.
    pub fn jump(&mut self) {
        if self.state == State::Hang {
            return self.start(State::Climb, "CLIMB06");
        }
        if self.state == State::WallRun {
            return self.wall_kick("TRI_J01");
        }
        if let Some(dy) = self.vault_top.filter(|_| matches!(self.state, State::Idle | State::Walk | State::Run | State::Crouch)) {
            self.vault_base = self.pos[1];
            self.vault_h = dy;
            return self.start(State::Vault, "CLIMB_B01");
        }
        let facing = self.wall_normal.is_some_and(|n| -(self.heading.sin() * n[0] + self.heading.cos() * n[2]) > 0.7);
        if self.state == State::Run && self.wall_tall && facing {
            self.start(State::WallRun, "KABE_U01");
        } else {
            if self.request(State::Jump, "JUMP01") {
                self.air_heading = self.heading;
            }
        }
    }

    /// Leave the wall backwards (away from its normal) with the given clip.
    fn wall_kick(&mut self, clip: &str) {
        if let Some(n) = self.wall_normal {
            self.heading = n[0].atan2(n[2]);
            self.target_heading = self.heading;
        }
        let (s, c) = self.heading.sin_cos();
        self.air_vel = [s * super::data::KICK_SPEED, c * super::data::KICK_SPEED];
        self.vy = 0.0;
        self.fall_from = self.pos[1];
        self.start(State::WallKick, clip);
    }

    /// The one gate for player-chosen actions: free locomotion may start anything; any other state only what the
    /// current clip's open cancel window (event types 3/4) names. Returns whether the action started.
    pub fn request(&mut self, state: State, name: &str) -> bool {
        let f = self.world.frame;
        let id = ACTIONS.iter().find(|x| x.name == name).map(|x| x.id);
        let free = matches!(self.state, State::Idle | State::Run | State::Walk | State::Halt | State::Crouch);
        let ok = free || self.world.action.cancels.iter().any(|c| Some(c.0) == id && c.1 <= f && f < c.2);
        if ok {
            self.start(state, name);
        }
        ok
    }

    fn start(&mut self, state: State, name: &str) {
        self.state = state;
        self.world = World::new(name);
    }

    fn advance(&mut self) {
        let before = self.world.pos;
        self.world.step();
        let d: [f32; 3] = std::array::from_fn(|i| self.world.pos[i] - before[i]);
        let (s, c) = self.heading.sin_cos();
        self.pos[0] += d[0] * c + d[2] * s;
        self.pos[1] += d[1];
        self.pos[2] += d[2] * c - d[0] * s;
    }

    /// ESTIMATE: the first attack is AT_A01; while an attack's cancel window is open, a press goes to the first
    /// permitted target (the choice between targets in the real game depends on input direction/state, not decoded).
    fn press_attack(&mut self) {
        if self.state == State::Fall || self.state == State::Jump && self.world.frame >= TAKEOFF_FRAME {
            let clip = if self.air_back() { "KICK_JB01" } else { "KICK_J01" };
            self.fall_from = if self.state == State::Fall { self.fall_from } else { self.pos[1] };
            return self.start(State::AirAct, clip);
        }
        // ESTIMATE: from a wall hug the `_H` variants play, picked like the open-ground attacks (Forward = power
        // slash, R1 = uppercut, Back = 27 kick, else AT_E01_H); the wall-facing hug is left.
        if self.state == State::Hug {
            let clip = match self.attack_mod {
                AttackMod::Forward => "AT_CA01_H",
                AttackMod::Back => "KICK27_H",
                _ => "AT_E01_H",
            };
            return self.start(State::Attack, clip);
        }
        // R1 is held while the uppercut is pressed, so the guard stance is already up: Square leaves it.
        if self.state == State::Guard && self.attack_mod == AttackMod::Lock {
            return self.start(State::Attack, "AT_E01");
        }
        // ESTIMATE: "R1 + Square x2" (PAL) follows the uppercut with the downward strike; AT_E01 has no cancel list, so
        // the second press after its hit window (frame 19) plays KICK_J01 directly.
        if self.state == State::Attack && self.attack_mod == AttackMod::Lock
            && self.world.action.name == "AT_E01" && self.world.frame >= 19 {
            return self.start(State::Attack, "KICK_J01");
        }
        let f = self.world.frame;
        // ESTIMATE: pairing of rows to motions by cancel graph and clip names (AT_CA = forward power slash, SOCCER =
        // kick, AT_E = uppercut, AT_DASH05 = dash leap); not frame-matched against the attack video.
        let want: Option<&str> = match self.attack_mod {
            AttackMod::Forward => Some("AT_CA01"),
            AttackMod::Back => Some("SOCCER01"),
            AttackMod::Lock if self.state != State::Attack => Some("AT_E01"),
            AttackMod::Dash if self.state != State::Attack => Some("AT_DASH05"),
            _ => None,
        };
        let by_name = |n: &str| ACTIONS.iter().find(|x| x.name == n);
        let target = if self.state == State::Attack {
            let a = self.world.action;
            let open = a.cancels.iter().filter(|c| c.1 <= f && f < c.2);
            // ESTIMATE: inside a chain Forward prefers the jumping strike (AT_DA) before the power slash, R1 the
            // 27-set kicks (FUTO27, KICK27) that AT_C01's cancel list offers.
            let prefs: &[&str] = match self.attack_mod {
                AttackMod::Forward => &["AT_DA01", "AT_CA01"],
                AttackMod::Lock => &["FUTO27", "KICK27"],
                _ => want.as_slice(),
            };
            let preferred = prefs.iter().filter_map(|n| by_name(n)).find_map(|w| open.clone().find(|c| c.0 == w.id));
            preferred.or_else(|| open.clone().next()).and_then(|c| ACTIONS.iter().find(|x| x.id == c.0))
        } else {
            want.and_then(|w| if w == "AT_CA01" { None } else { by_name(w) }).or_else(|| by_name("AT_A01"))
        };
        if let Some(t) = target {
            self.request(State::Attack, t.name);
        }
    }
}

/// Hit damage from a type-35 event (formats.md, "Damage use of event type 35"):
/// `trunc(trunc(u16(arg0) * power) * mult)`; `power` is the attacker's float stat at `[+0x45c]+0x34`
/// (value not found: caller supplies it, ESTIMATE) and `mult` is 0.5/1.0/2.0 from the sign of the float at `+0x5c`.
pub fn hit_damage(arg0: u32, power: f32, bias: f32) -> i32 {
    let mult = if bias < 0.0 { 0.5 } else if bias <= 0.0 { 1.0 } else { 2.0 };
    (((arg0 & 0xffff) as f32 * power) as i32 as f32 * mult) as i32
}
