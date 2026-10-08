//! Capsule rig driven by the sim. WASD move (Alt walk), LMB attack, Space jump, Q item, RMB guard, Ctrl+Space dodge; pad per docs/controls.md.
//! The sim stays engine-free: this only feeds `Player::step` at 60 Hz and draws `pos`/`heading`.

use bevy::prelude::*;
use bevy::render::view::screenshot::{save_to_disk, Screenshot};
use bevy::input::mouse::AccumulatedMouseMotion;
use shinobido_combat_rewrite::sim::camera::Camera as Orbit;
use shinobido_combat_rewrite::sim::data::{CAM_MOUSE_SENS, CAM_STICK_SPEED};
use shinobido_combat_rewrite::sim::input::Input;
use shinobido_combat_rewrite::sim::player::Player;
use shinobido_combat_rewrite::sim::terrain::Terrain;

#[derive(Resource, Default)]
struct Sim(Player);

#[derive(Resource, Default)]
struct Pad(Input);

#[derive(Resource, Default)]
struct Cam(Orbit);

#[derive(Component)]
struct Rig;

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

fn main() {
    let shot = parse_shot();
    let mut app = App::new();
    if let Some(shot) = shot {
        if let Some(dir) = std::path::Path::new(&shot.path).parent() {
            let _ = std::fs::create_dir_all(dir);
        }
        let _ = std::fs::remove_file(&shot.path);
        let mut sim = Sim::default();
        for _ in 0..shot.steps {
            sim.0.step(true, false);
        }
        app.insert_resource(sim).insert_resource(Cam(Orbit { yaw: shot.yaw.to_radians(), ..default() })).insert_resource(shot).add_systems(Update, shoot);
    } else {
        app.init_resource::<Sim>().init_resource::<Cam>();
    }
    app
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window { title: "Kagetora - combat sandbox".into(), ..default() }),
            ..default()
        }))
        .insert_resource(Time::<Fixed>::from_hz(60.0))
        .insert_resource(ClearColor(Color::srgb(0.07, 0.08, 0.1)))
        .insert_resource(GlobalAmbientLight { brightness: 350.0, ..default() })
        .init_resource::<Pad>()
        .add_systems(Startup, setup)
        .add_systems(FixedUpdate, tick.run_if(|s: Option<Res<Shot>>| s.is_none()))
        .add_systems(Update, (read_pad, orbit, draw, draw_grid).chain())
        .run();
}

fn setup(mut commands: Commands, mut meshes: ResMut<Assets<Mesh>>, mut mats: ResMut<Assets<StandardMaterial>>) {
    commands.spawn((Camera3d::default(), Transform::default()));
    commands.spawn((DirectionalLight { illuminance: 9000.0, shadow_maps_enabled: true, ..default() }, Transform::default().looking_to(Vec3::new(-0.5, -1.0, 0.35), Vec3::Y)));
    commands.spawn((Mesh3d(meshes.add(Plane3d::default().mesh().size(400.0, 400.0))), MeshMaterial3d(mats.add(StandardMaterial { base_color: Color::srgb(0.2, 0.23, 0.2), perceptual_roughness: 0.95, ..default() }))));
    commands.spawn((Rig, Mesh3d(meshes.add(Capsule3d::new(0.3, 1.1))), MeshMaterial3d(mats.add(Color::srgb(0.2, 0.2, 0.35))), Transform::default()));
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
fn read_pad(keys: Res<ButtonInput<KeyCode>>, mouse: Res<ButtonInput<MouseButton>>, pads: Query<&Gamepad>, mut pad: ResMut<Pad>) {
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
    }
    i.stick = [stick.x, stick.y];
}

fn tick(mut pad: ResMut<Pad>, cam: Res<Cam>, mut sim: ResMut<Sim>) {
    pad.0.apply(&mut sim.0, &Terrain::default(), cam.0.yaw);
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
    let (e, f) = cam.0.eye(sim.0.pos);
    **eye = Transform::from_translation(Vec3::from(e)).looking_at(Vec3::from(f), Vec3::Y);
}

fn draw(sim: Res<Sim>, mut rig: Single<&mut Transform, With<Rig>>) {
    let p = &sim.0;
    rig.translation = Vec3::new(p.pos[0], p.pos[1] + 0.85, p.pos[2]);
    rig.rotation = Quat::from_rotation_y(p.heading);
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
