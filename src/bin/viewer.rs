//! Capsule rig driven by the sim. W run (Shift walk), A/D turn, Space attack, G guard,.
//! The sim stays engine-free: this only feeds `Player::step` at 60 Hz and draws `pos`/`heading`.

use bevy::prelude::*;
use bevy::render::view::screenshot::{save_to_disk, Screenshot};
use bevy::input::mouse::AccumulatedMouseMotion;
use shinobido_combat_rewrite::sim::camera::Camera as Orbit;
use shinobido_combat_rewrite::sim::data::{CAM_MOUSE_SENS, CAM_STICK_SPEED};
use shinobido_combat_rewrite::sim::player::Player;

#[derive(Resource, Default)]
struct Sim(Player);

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
        .add_systems(Startup, setup)
        .add_systems(FixedUpdate, tick.run_if(|s: Option<Res<Shot>>| s.is_none()))
        .add_systems(Update, (orbit, draw, draw_grid).chain())
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

fn tick(keys: Res<ButtonInput<KeyCode>>, mut sim: ResMut<Sim>) {
    let p = &mut sim.0;
    let turn = keys.pressed(KeyCode::KeyA) as i32 - keys.pressed(KeyCode::KeyD) as i32;
    p.target_heading = p.heading + turn as f32 * 0.5;
    p.walk = keys.pressed(KeyCode::ShiftLeft);
    p.guard_held = keys.pressed(KeyCode::KeyG);
    // ponytail: just_pressed is per render frame, so a tap can be missed or doubled on a fixed tick; fine for a viewer.
    p.step(keys.pressed(KeyCode::KeyW), keys.just_pressed(KeyCode::Space));
}

/// Mouse (while right button held) and right stick orbit the camera around the player.
fn orbit(
    mut cam: ResMut<Cam>,
    sim: Res<Sim>,
    mouse: Res<ButtonInput<MouseButton>>,
    motion: Res<AccumulatedMouseMotion>,
    pads: Query<&Gamepad>,
    time: Res<Time>,
    mut eye: Single<&mut Transform, With<Camera3d>>,
) {
    let mut d = Vec2::ZERO;
    if mouse.pressed(MouseButton::Right) {
        d += motion.delta * CAM_MOUSE_SENS;
    }
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
