//! Capsule rig driven by the sim. W run (Shift walk), A/D turn, Space attack, G guard,.
//! The sim stays engine-free: this only feeds `Player::step` at 60 Hz and draws `pos`/`heading`.

use bevy::prelude::*;
use shinobido_combat_rewrite::sim::player::Player;

#[derive(Resource, Default)]
struct Sim(Player);

#[derive(Component)]
struct Rig;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window { title: "Kagetora - combat sandbox".into(), ..default() }),
            ..default()
        }))
        .insert_resource(Time::<Fixed>::from_hz(60.0))
        .init_resource::<Sim>()
        .add_systems(Startup, setup)
        .add_systems(FixedUpdate, tick)
        .add_systems(Update, draw)
        .run();
}

fn setup(mut commands: Commands, mut meshes: ResMut<Assets<Mesh>>, mut mats: ResMut<Assets<StandardMaterial>>) {
    commands.spawn((Camera3d::default(), Transform::from_xyz(0.0, 4.0, -7.0).looking_at(Vec3::new(0.0, 1.0, 0.0), Vec3::Y)));
    commands.spawn((DirectionalLight { shadow_maps_enabled: true, ..default() }, Transform::from_xyz(3.0, 8.0, -4.0).looking_at(Vec3::ZERO, Vec3::Y)));
    commands.spawn((Mesh3d(meshes.add(Plane3d::default().mesh().size(60.0, 60.0))), MeshMaterial3d(mats.add(Color::srgb(0.3, 0.35, 0.3)))));
    commands.spawn((Rig, Mesh3d(meshes.add(Capsule3d::new(0.3, 1.1))), MeshMaterial3d(mats.add(Color::srgb(0.2, 0.2, 0.35))), Transform::default()));
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

fn draw(sim: Res<Sim>, mut rig: Single<&mut Transform, With<Rig>>) {
    let p = &sim.0;
    rig.translation = Vec3::new(p.pos[0], p.pos[1] + 0.85, p.pos[2]);
    rig.rotation = Quat::from_rotation_y(p.heading);
}
