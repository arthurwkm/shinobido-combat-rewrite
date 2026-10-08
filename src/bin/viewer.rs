//! Capsule rig driven by the sim. WASD move (Alt walk), LMB attack, Space jump, Q item, RMB guard, Ctrl+Space dodge; pad per docs/controls.md.
//! The sim stays engine-free: this only feeds `Player::step` at 60 Hz and draws `pos`/`heading`.

use bevy::prelude::*;
use bevy::render::view::screenshot::{save_to_disk, Screenshot};
use bevy::input::mouse::AccumulatedMouseMotion;
use shinobido_combat_rewrite::sim::camera::Camera as Orbit;
use shinobido_combat_rewrite::sim::data::{CAM_MOUSE_SENS, CAM_STICK_SPEED};
use shinobido_combat_rewrite::sim::hud::{self, Debug};
use shinobido_combat_rewrite::sim::input::Input;
use shinobido_combat_rewrite::sim::combat::Fight;
use shinobido_combat_rewrite::sim::enemy::Enemy;
use shinobido_combat_rewrite::sim::execution;
use shinobido_combat_rewrite::sim::stealth::Stealth;
use shinobido_combat_rewrite::sim::player::Player;
use shinobido_combat_rewrite::sim::pose::{Clips, Skeleton, NODES};
use shinobido_combat_rewrite::sim::terrain::{Terrain, STAIRS_SLOPE};

#[derive(Resource)]
struct Sim(Fight);

impl Default for Sim {
    fn default() -> Self {
        // Soldier 6 m ahead, facing the player (position ESTIMATE).
        let mut enemy = Enemy::new([0.0, 0.0, 6.0]);
        enemy.heading = std::f32::consts::PI;
        Sim(Fight::new(Player::default(), enemy))
    }
}

#[derive(Resource)]
struct World(Terrain);

#[derive(Resource, Default)]
struct Pad(Input);

/// Execute pressed since the last tick (E / Triangle).
#[derive(Resource, Default)]
struct Exec(bool);

#[derive(Resource, Default)]
struct Dbg(Debug);

#[derive(Resource, Default)]
struct Aware(Stealth);

/// Game HUD pieces (Shinobido clips): health top-left, eye icon and stealth meter top-centre, ki compass top-right, execution cue.
#[derive(Component)]
enum Game {
    Health,
    Eye,
    Meter,
    Needle,
}

/// Debug HUD pieces (reference hud.rs layout): bars and log top-left, readout top-right, target bar bottom.
#[derive(Component)]
enum Hud {
    PlayerBar,
    EnemyBar,
    Log,
    Readout,
}


#[derive(Resource, Default)]
struct Cam(Orbit);

#[derive(Component)]
struct Rig;

/// One cuboid per skeleton node, stretched from its parent joint to its own (the game skeleton drawn as primitives); `true` = the soldier's.
#[derive(Component)]
struct Bone(usize, bool);

/// Floating diamond over the soldier while an execution is available.
#[derive(Component)]
struct Cue;

/// Game skeleton and sampled clips from `extracted/anim` (tools/pose.py); `None` leaves the capsule.
#[derive(Resource)]
struct Anim(Option<(Skeleton, Clips)>);

fn load_anim() -> Anim {
    let read = |n| std::fs::read(format!("extracted/anim/{n}.bin")).ok();
    Anim(read("skeleton").and_then(|s| Skeleton::parse(&s)).zip(read("clips").and_then(|c| Clips::parse(&c))))
}

/// `--shot <png> [--yaw <deg>] [--steps <n>]`: render a fixed scene (n sim ticks running, camera
/// swung by yaw degrees around the player), save the PNG and exit.
#[derive(Resource)]
struct Shot {
    path: String,
    yaw: f32,
    steps: u32,
    frame: u32,
}

fn parse_shot() -> Option<Shot> {
    let a: Vec<String> = std::env::args().collect();
    let val = |k: &str| a.iter().position(|x| x == k).and_then(|i| a.get(i + 1));
    Some(Shot {
        path: val("--shot")?.clone(),
        yaw: val("--yaw").and_then(|v| v.parse().ok()).unwrap_or(0.0),
        steps: val("--steps").and_then(|v| v.parse().ok()).unwrap_or(0),
        frame: 0,
    })
}

/// `--record <dir> --script <file> [--every <n>] [--ticks <n>]`: one sim tick per rendered frame, pad driven by the
/// script (`<tick> stick <x> <y> | lock <0|1> | crouch <0|1> | attack | jump | item | exec | yaw <deg>`, `#` comments),
/// a frame saved every n ticks as `<dir>/<tick>.png`, exits after `ticks` (default: last script tick + 30).
#[derive(Resource)]
struct Record {
    dir: String,
    script: Vec<(u32, String)>,
    every: u32,
    ticks: u32,
    tick: u32,
}

fn parse_record() -> Option<Record> {
    let a: Vec<String> = std::env::args().collect();
    let val = |k: &str| a.iter().position(|x| x == k).and_then(|i| a.get(i + 1));
    let dir = val("--record")?.clone();
    let text = std::fs::read_to_string(val("--script")?).expect("script unreadable");
    let script: Vec<(u32, String)> = text
        .lines()
        .filter_map(|l| {
            let l = l.split('#').next()?.trim();
            let (t, cmd) = l.split_once(' ')?;
            Some((t.parse().ok()?, cmd.trim().to_string()))
        })
        .collect();
    let last = script.iter().map(|s| s.0).max().unwrap_or(0);
    Some(Record {
        dir,
        script,
        every: val("--every").and_then(|v| v.parse().ok()).unwrap_or(6),
        ticks: val("--ticks").and_then(|v| v.parse().ok()).unwrap_or(last + 30),
        tick: 0,
    })
}

fn record_step(mut rec: ResMut<Record>, mut pad: ResMut<Pad>, mut exec: ResMut<Exec>, mut cam: ResMut<Cam>, mut commands: Commands, mut exit: MessageWriter<AppExit>) {
    let t = rec.tick;
    for (_, cmd) in rec.script.iter().filter(|s| s.0 == t) {
        let mut w = cmd.split_whitespace();
        let v: Vec<f32> = w.clone().skip(1).filter_map(|x| x.parse().ok()).collect();
        let i = &mut pad.0;
        match (w.next(), v.as_slice()) {
            (Some("stick"), [x, y]) => i.stick = [*x, *y],
            (Some("lock"), [b]) => i.lock = *b != 0.0,
            (Some("crouch"), [b]) => i.crouch = *b != 0.0,
            (Some("yaw"), [d]) => cam.0.yaw = d.to_radians(),
            (Some("attack"), _) => i.attack = true,
            (Some("jump"), _) => i.jump = true,
            (Some("item"), _) => i.item = true,
            (Some("exec"), _) => exec.0 = true,
            _ => eprintln!("script: bad line `{cmd}`"),
        }
    }
    if t > 0 && t.is_multiple_of(rec.every) {
        commands.spawn(Screenshot::primary_window()).observe(save_to_disk(format!("{}/{t:05}.png", rec.dir)));
    }
    if t >= rec.ticks + 30 {
        exit.write(AppExit::Success); // slack for the last async screenshots
    }
    rec.tick += 1;
}

fn main() {
    let shot = parse_shot();
    let rec = parse_record();
    let mut app = App::new();
    if let Some(shot) = shot {
        if let Some(dir) = std::path::Path::new(&shot.path).parent() {
            let _ = std::fs::create_dir_all(dir);
        }
        let _ = std::fs::remove_file(&shot.path);
        let mut sim = Sim::default();
        for _ in 0..shot.steps {
            sim.0.player.step(true, false);
        }
        app.insert_resource(sim).insert_resource(Cam(Orbit { yaw: shot.yaw.to_radians(), ..default() })).insert_resource(shot).add_systems(Update, shoot);
    } else {
        app.init_resource::<Sim>().init_resource::<Cam>();
        if let Some(rec) = rec {
            let _ = std::fs::remove_dir_all(&rec.dir);
            let _ = std::fs::create_dir_all(&rec.dir);
            app.insert_resource(rec);
        }
    }
    app
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window { title: "Kagetora - combat sandbox".into(), ..default() }),
            ..default()
        }))
        .insert_resource(Time::<Fixed>::from_hz(60.0))
        .insert_resource(ClearColor(Color::srgb(0.07, 0.08, 0.1)))
        .insert_resource(GlobalAmbientLight { brightness: 350.0, ..default() })
        .insert_resource(World(Terrain::sandbox()))
        .init_resource::<Pad>()
        .init_resource::<Exec>()
        .init_resource::<Dbg>()
        .init_resource::<Aware>()
        .insert_resource(load_anim())
        .add_systems(Startup, (setup, setup_hud, setup_game_hud))
        .add_systems(FixedUpdate, tick.run_if(|s: Option<Res<Shot>>, r: Option<Res<Record>>| s.is_none() && r.is_none()))
        .add_systems(Update, (
            (record_step, tick).chain().run_if(resource_exists::<Record>),
            read_pad.run_if(|r: Option<Res<Record>>| r.is_none()),
            orbit, draw, pose_bones, cue, draw_grid, update_hud, update_game_hud).chain())
        .run();
}

fn setup(mut commands: Commands, mut meshes: ResMut<Assets<Mesh>>, mut mats: ResMut<Assets<StandardMaterial>>, world: Res<World>, anim: Res<Anim>) {
    commands.spawn((Camera3d::default(), Transform::default()));
    commands.spawn((DirectionalLight { illuminance: 9000.0, shadow_maps_enabled: true, ..default() }, Transform::default().looking_to(Vec3::new(-0.5, -1.0, 0.35), Vec3::Y)));
    commands.spawn((Mesh3d(meshes.add(Plane3d::default().mesh().size(400.0, 400.0))), MeshMaterial3d(mats.add(StandardMaterial { base_color: Color::srgb(0.2, 0.23, 0.2), perceptual_roughness: 0.95, ..default() }))));
    spawn_terrain(&mut commands, &mut meshes, &mut mats, &world.0);
    let body = mats.add(Color::srgb(0.2, 0.2, 0.35));
    if let Some((skel, _)) = &anim.0 {
        let unit = meshes.add(Cuboid::new(0.08, 0.08, 1.0));
        let foe = mats.add(Color::srgb(0.6, 0.15, 0.12)); // A_BUSHI1 is the red soldier
        for n in (0..NODES).filter(|&n| skel.parent[n].is_some_and(|p| p != 23)) {
            commands.spawn((Bone(n, false), Mesh3d(unit.clone()), MeshMaterial3d(body.clone()), Transform::default()));
            commands.spawn((Bone(n, true), Mesh3d(unit.clone()), MeshMaterial3d(foe.clone()), Transform::default()));
        }
    } else {
        commands.spawn((Rig, Mesh3d(meshes.add(Capsule3d::new(0.3, 1.1))), MeshMaterial3d(body), Transform::default()));
    }
    let cue = mats.add(StandardMaterial { base_color: Color::srgb(1.0, 0.85, 0.1), emissive: LinearRgba::rgb(4.0, 3.0, 0.2), ..default() });
    commands.spawn((Cue, Mesh3d(meshes.add(Cuboid::new(0.25, 0.25, 0.25))), MeshMaterial3d(cue), Transform::default(), Visibility::Hidden));
}

/// Plain grey blocks; ramps are slabs, or steps (top at each step's low edge) when steeper than `STAIRS_SLOPE`.
fn spawn_terrain(commands: &mut Commands, meshes: &mut Assets<Mesh>, mats: &mut Assets<StandardMaterial>, terrain: &Terrain) {
    let grey = mats.add(StandardMaterial { base_color: Color::srgb(0.45, 0.45, 0.47), perceptual_roughness: 0.9, ..default() });
    let mut block = |commands: &mut Commands, min: Vec3, max: Vec3, rot: Quat| {
        let size = max - min;
        commands.spawn((Mesh3d(meshes.add(Cuboid::new(size.x, size.y, size.z))), MeshMaterial3d(grey.clone()), Transform::from_translation((min + max) / 2.0).with_rotation(rot)));
    };
    for c in &terrain.colliders {
        let (min, max) = (Vec3::from(c.min), Vec3::from(c.max));
        if !c.ramp {
            block(commands, min, max, Quat::IDENTITY);
            continue;
        }
        let (rise, run) = (max.y - min.y, max.z - min.z);
        if rise / run > STAIRS_SLOPE {
            let n = (rise / 0.25).round().max(1.0) as u32;
            for i in 0..n {
                let z0 = min.z + run * i as f32 / n as f32;
                let top = min.y + rise * i as f32 / n as f32;
                if top > 0.0 {
                    block(commands, Vec3::new(min.x, 0.0, z0), Vec3::new(max.x, top, z0 + run / n as f32), Quat::IDENTITY);
                }
            }
        } else {
            let (len, tilt) = (rise.hypot(run), (rise / run).atan());
            let centre = Vec3::new((min.x + max.x) / 2.0, min.y + rise / 2.0 - 0.05, (min.z + max.z) / 2.0);
            let half = Vec3::new((max.x - min.x) / 2.0, 0.05, len / 2.0);
            block(commands, centre - half, centre + half, Quat::from_rotation_x(-tilt));
        }
    }
}

fn shoot(mut commands: Commands, mut shot: ResMut<Shot>, mut exit: MessageWriter<AppExit>) {
    shot.frame += 1;
    if shot.frame == 10 {
        commands.spawn(Screenshot::primary_window()).observe(save_to_disk(shot.path.clone()));
    } else if shot.frame > 10 && std::path::Path::new(&shot.path).exists() {
        exit.write(AppExit::Success);
    } else if shot.frame > 600 {
        exit.write(AppExit::error());
    }
}

/// Samples keyboard/mouse and gamepad every render frame: held state overwritten, edges OR-ed until a tick takes them.
fn read_pad(keys: Res<ButtonInput<KeyCode>>, mouse: Res<ButtonInput<MouseButton>>, pads: Query<&Gamepad>, mut pad: ResMut<Pad>, mut exec: ResMut<Exec>) {
    let k = |c| keys.pressed(c) as i32 as f32;
    let mut stick = Vec2::new(k(KeyCode::KeyD) - k(KeyCode::KeyA), k(KeyCode::KeyW) - k(KeyCode::KeyS)).clamp_length_max(1.0);
    if keys.pressed(KeyCode::AltLeft) {
        stick *= 0.4; // keyboard walk (docs/controls.md); below STICK_WALK
    }
    let i = &mut pad.0;
    i.attack |= mouse.just_pressed(MouseButton::Left);
    i.jump |= keys.just_pressed(KeyCode::Space);
    i.item |= keys.just_pressed(KeyCode::KeyQ);
    i.lock = mouse.pressed(MouseButton::Right);
    i.crouch = keys.pressed(KeyCode::ControlLeft);
    exec.0 |= keys.just_pressed(KeyCode::KeyE);
    for g in &pads {
        let s = Vec2::new(g.get(GamepadAxis::LeftStickX).unwrap_or(0.0), g.get(GamepadAxis::LeftStickY).unwrap_or(0.0));
        if s.length() > stick.length() {
            stick = s.clamp_length_max(1.0);
        }
        i.attack |= g.just_pressed(GamepadButton::West);
        i.jump |= g.just_pressed(GamepadButton::South);
        i.item |= g.just_pressed(GamepadButton::LeftTrigger2);
        i.lock |= g.pressed(GamepadButton::RightTrigger);
        i.crouch |= g.pressed(GamepadButton::LeftTrigger);
        exec.0 |= g.just_pressed(GamepadButton::North);
    }
    i.stick = [stick.x, stick.y];
}

fn tick(mut pad: ResMut<Pad>, mut exec: ResMut<Exec>, cam: Res<Cam>, mut sim: ResMut<Sim>, world: Res<World>, mut dbg: ResMut<Dbg>, mut aware: ResMut<Aware>) {
    if std::mem::take(&mut exec.0) {
        sim.0.execute();
    }
    pad.0.apply(&mut sim.0.player, &world.0, cam.0.yaw);
    sim.0.step_enemy();
    if sim.0.enemy.hp > 0 {
        let (pos, action) = (sim.0.player.pos, sim.0.player.world_action());
        aware.0.step(&mut sim.0.enemy, pos, action);
    }
    dbg.0.observe(&sim.0);
}

fn setup_hud(mut commands: Commands) {
    let font = |size: f32| TextFont { font_size: FontSize::Px(size), ..default() };
    let back = Color::srgba(0.0, 0.0, 0.0, 0.6);
    let bar = |commands: &mut Commands, node: Node, kind: Hud, colour: Color| {
        let outer = commands.spawn((node, BackgroundColor(back))).id();
        commands.spawn((Node { width: percent(100), height: percent(100), ..default() }, BackgroundColor(colour), kind, ChildOf(outer)));
    };
    let abs = |left: Option<Val>, right: Option<Val>, top: Option<Val>, bottom: Option<Val>| Node {
        position_type: PositionType::Absolute,
        left: left.unwrap_or(Val::Auto), right: right.unwrap_or(Val::Auto), top: top.unwrap_or(Val::Auto), bottom: bottom.unwrap_or(Val::Auto),
        ..default()
    };
    let red = Color::srgb(0.62, 0.12, 0.1);
    bar(&mut commands, Node { width: px(240), height: px(11), ..abs(Some(px(28)), None, Some(px(34)), None) }, Hud::PlayerBar, red);
    commands.spawn((Text::new(""), font(15.0), TextColor(Color::srgb(0.95, 0.85, 0.5)), Hud::Log, abs(Some(px(28)), None, Some(px(54)), None)));
    commands.spawn((Text::new(""), font(15.0), Hud::Readout, abs(None, Some(px(20)), Some(px(20)), None)));
    commands.spawn((Text::new("Soldier"), font(18.0), abs(Some(percent(25)), None, None, Some(px(66)))));
    bar(&mut commands, Node { width: percent(50), height: px(9), ..abs(Some(percent(25)), None, None, Some(px(48))) }, Hud::EnemyBar, red);
}

fn setup_game_hud(mut commands: Commands) {
    let back = Color::srgba(0.0, 0.0, 0.0, 0.6);
    let abs = |left: Option<Val>, right: Option<Val>, top: Val, w: f32, h: f32| Node {
        position_type: PositionType::Absolute, left: left.unwrap_or(Val::Auto), right: right.unwrap_or(Val::Auto), top, width: px(w), height: px(h), ..default()
    };
    // Health: red bar, top-left.
    let outer = commands.spawn((abs(Some(px(28)), None, px(8), 300.0, 14.0), BackgroundColor(back))).id();
    commands.spawn((Node { width: percent(100), height: percent(100), ..default() }, BackgroundColor(Color::srgb(0.75, 0.1, 0.08)), Game::Health, ChildOf(outer)));
    // Eye icon with the stealth meter under it, top-centre.
    commands.spawn((abs(Some(percent(50)), None, px(8), 44.0, 22.0), UiTransform::from_xy(px(-22), px(0)), BackgroundColor(Color::WHITE), Game::Eye));
    let meter = commands.spawn((abs(Some(percent(50)), None, px(34), 60.0, 6.0), UiTransform::from_xy(px(-30), px(0)), BackgroundColor(back))).id();
    commands.spawn((Node { width: percent(0), height: percent(100), ..default() }, BackgroundColor(Color::srgb(0.9, 0.9, 0.8)), Game::Meter, ChildOf(meter)));
    // Ki compass: ring with an orange needle, top-right under the readout.
    let ring = commands.spawn((Node { border: UiRect::all(px(3)), border_radius: BorderRadius::MAX, ..abs(None, Some(px(24)), px(170), 64.0, 64.0) }, BorderColor::all(Color::srgb(0.8, 0.8, 0.75)), BackgroundColor(back))).id();
    let needle = commands.spawn((Node { width: percent(100), height: percent(100), justify_content: JustifyContent::Center, ..default() }, Game::Needle, ChildOf(ring))).id();
    commands.spawn((Node { width: px(5), height: px(26), ..default() }, BackgroundColor(Color::srgb(1.0, 0.55, 0.1)), ChildOf(needle)));
}

type GameParts = (&'static Game, Option<&'static mut Node>, Option<&'static mut BackgroundColor>, Option<&'static mut UiTransform>);

fn update_game_hud(
    sim: Res<Sim>,
    aware: Res<Aware>,
    cam: Res<Cam>,
    mut q: Query<GameParts>,
) {
    let f = &sim.0;
    let bearing = hud::compass(f, cam.0.yaw);
    for (kind, node, bg, tr) in &mut q {
        match (kind, node, bg, tr) {
            (Game::Health, Some(mut n), ..) => n.width = percent(100.0 * hud::player_bar(f)),
            (Game::Meter, Some(mut n), ..) => n.width = percent(100.0 * aware.0.meter),
            (Game::Eye, _, Some(mut b), ..) => {
                let c = hud::eye_colour(aware.0.level());
                b.0 = Color::srgb(c[0], c[1], c[2]);
            }
            (Game::Needle, Some(mut n), _, Some(mut t)) => {
                n.display = if bearing.is_some() { Display::Flex } else { Display::None };
                t.rotation = Rot2::radians(bearing.unwrap_or(0.0));
            }
            _ => {}
        }
    }
}

fn update_hud(sim: Res<Sim>, pad: Res<Pad>, dbg: Res<Dbg>, mut q: Query<(&Hud, Option<&mut Text>, Option<&mut Node>)>) {
    for (kind, text, node) in &mut q {
        match (kind, text, node) {
            (Hud::PlayerBar, _, Some(mut n)) => n.width = percent(100.0 * hud::player_bar(&sim.0)),
            (Hud::EnemyBar, _, Some(mut n)) => n.width = percent(100.0 * hud::enemy_bar(&sim.0)),
            (Hud::Log, Some(mut t), _) => t.0 = dbg.0.log.join("\n"),
            (Hud::Readout, Some(mut t), _) => t.0 = dbg.0.readout(&sim.0, &pad.0),
            _ => {}
        }
    }
}

/// Mouse motion and right stick orbit the camera around the player.
fn orbit(
    mut cam: ResMut<Cam>,
    sim: Res<Sim>,
    motion: Res<AccumulatedMouseMotion>,
    pads: Query<&Gamepad>,
    time: Res<Time>,
    mut eye: Single<&mut Transform, With<Camera3d>>,
) {
    let mut d = motion.delta * CAM_MOUSE_SENS;
    for pad in &pads {
        d += Vec2::new(pad.get(GamepadAxis::RightStickX).unwrap_or(0.0), -pad.get(GamepadAxis::RightStickY).unwrap_or(0.0))
            * CAM_STICK_SPEED
            * time.delta_secs();
    }
    cam.0.orbit([-d.x, d.y]); // looking down +z, screen-right is -x
    let (e, f) = cam.0.eye(sim.0.player.pos);
    **eye = Transform::from_translation(Vec3::from(e)).looking_at(Vec3::from(f), Vec3::Y);
}

fn draw(sim: Res<Sim>, mut rig: Option<Single<&mut Transform, With<Rig>>>) {
    let p = &sim.0.player;
    if let Some(rig) = rig.as_mut() {
        rig.translation = Vec3::new(p.pos[0], p.pos[1] + 0.85, p.pos[2]);
        rig.rotation = Quat::from_rotation_y(p.heading);
    }
}

/// Poses both sets of bone cuboids with the sim's current clips and frames (clips that were not exported keep the last pose).
fn pose_bones(sim: Res<Sim>, anim: Res<Anim>, mut bones: Query<(&Bone, &mut Transform)>) {
    let Some((skel, clips)) = &anim.0 else { return };
    let (p, e) = (&sim.0.player, &sim.0.enemy);
    let rows = [clips.row(p.world_action(), p.world_frame()).map(|r| (r, p.pos, p.heading)), clips.row(e.action(), e.frame()).map(|r| (r, e.pos, e.heading))];
    let joints = rows.map(|r| r.map(|(row, pos, heading)| skel.joints(row).map(|j| Vec3::from(pos) + Quat::from_rotation_y(heading) * Vec3::from(j))));
    for (Bone(n, foe), mut t) in &mut bones {
        let Some(joints) = &joints[*foe as usize] else { continue };
        let (a, b) = (joints[skel.parent[*n].unwrap_or(*n)], joints[*n]);
        let len = a.distance(b).max(1e-3);
        *t = Transform::from_translation((a + b) / 2.0).looking_to(if len > 1e-3 { b - a } else { Vec3::Z }, Vec3::Y).with_scale(Vec3::new(1.0, 1.0, len));
    }
}

/// The execution cue: a spinning diamond above the soldier's head while `execution::available` offers a kill.
fn cue(sim: Res<Sim>, time: Res<Time>, mut c: Single<(&mut Transform, &mut Visibility), With<Cue>>) {
    let e = &sim.0.enemy;
    let on = execution::available(&sim.0.player, e).is_some();
    *c.1 = if on { Visibility::Visible } else { Visibility::Hidden };
    c.0.translation = Vec3::new(e.pos[0], e.pos[1] + 2.1, e.pos[2]);
    c.0.rotation = Quat::from_rotation_y(time.elapsed_secs() * 3.0) * Quat::from_rotation_x(0.6);
}

/// With no textures, a grid is what makes speed and distance readable (as in the reference).
fn draw_grid(mut gizmos: Gizmos) {
    gizmos.grid(
        Isometry3d::new(Vec3::Y * 0.01, Quat::from_rotation_x(std::f32::consts::FRAC_PI_2)),
        UVec2::splat(100),
        Vec2::splat(2.0),
        Color::srgba(0.6, 0.65, 0.6, 0.18),
    );
}
