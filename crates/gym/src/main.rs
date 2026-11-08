//! Gym: play pilot-core on a test course.

mod course;

use std::path::PathBuf;
use std::time::SystemTime;

use bevy::input::mouse::AccumulatedMouseMotion;
use bevy::prelude::*;
use bevy::window::{CursorGrabMode, PrimaryWindow};
use pilot_core::{step, Button as PilotButton, Input as PilotInput, PilotState, Tuning};
use world_parry::BoxWorld;

const MOUSE_SENSITIVITY: f32 = 0.002;
const PITCH_LIMIT: f32 = 1.5;
const FIELD_OF_VIEW_DEG: f32 = 95.0;

/// pilot-core is Z-up; Bevy is Y-up. (x, y, z) becomes (x, z, -y).
fn to_bevy(v: Vec3) -> Vec3 {
    Vec3::new(v.x, v.z, -v.y)
}

fn rotation_to_bevy(q: Quat) -> Quat {
    Quat::from_xyzw(q.x, q.z, -q.y, q.w)
}

/// Full Bevy-space size of a box given its pilot-space half extents.
fn box_size_to_bevy(half_extents: Vec3) -> Vec3 {
    Vec3::new(half_extents.x, half_extents.z, half_extents.y) * 2.0
}

/// Unit view direction in pilot space.
fn look_dir(yaw: f32, pitch: f32) -> Vec3 {
    Vec3::new(pitch.cos() * yaw.cos(), pitch.cos() * yaw.sin(), pitch.sin())
}

#[derive(Resource)]
struct Sim {
    state: PilotState,
    /// Feet position before the latest step, for smooth rendering between steps.
    previous_position: Vec3,
    tuning: Tuning,
    world: BoxWorld,
    tuning_path: PathBuf,
    tuning_stamp: Option<SystemTime>,
}

/// View angles, plus button presses latched until the next fixed step.
#[derive(Resource, Default)]
struct Controls {
    yaw: f32,
    pitch: f32,
    jump: bool,
    crouch: bool,
    hang: bool,
}

#[derive(Component)]
struct Eye;

#[derive(Component)]
struct Hud;

fn main() {
    let tuning_path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../tuning/pilot.toml");
    let tuning = pilot_tuning::load(&tuning_path).expect("tuning/pilot.toml must load");
    let state = PilotState::spawn(course::SPAWN, &tuning);

    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window { title: "Cyberfall gym".into(), ..default() }),
            ..default()
        }))
        .insert_resource(Time::<Fixed>::from_hz(60.0))
        .insert_resource(Sim {
            state,
            previous_position: state.position,
            tuning,
            world: course::build(),
            tuning_path,
            tuning_stamp: None,
        })
        .init_resource::<Controls>()
        .add_systems(Startup, setup)
        .add_systems(FixedUpdate, simulate)
        .add_systems(
            Update,
            (look, latch_presses, quit_or_respawn, reload_tuning, sync_camera, update_hud).chain(),
        )
        .run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    sim: Res<Sim>,
    mut window: Single<&mut Window, With<PrimaryWindow>>,
) {
    window.cursor_options.grab_mode = CursorGrabMode::Locked;
    window.cursor_options.visible = false;

    for (index, solid) in sim.world.boxes().enumerate() {
        let shade = 0.35 + 0.1 * (index % 5) as f32;
        commands.spawn((
            Mesh3d(meshes.add(Cuboid::from_size(box_size_to_bevy(solid.half_extents)))),
            MeshMaterial3d(materials.add(Color::srgb(shade, shade, shade + 0.08))),
            Transform {
                translation: to_bevy(solid.centre),
                rotation: rotation_to_bevy(solid.rotation),
                ..default()
            },
        ));
    }

    commands.spawn((
        DirectionalLight { illuminance: 10_000.0, shadows_enabled: true, ..default() },
        Transform::from_xyz(30.0, 60.0, 20.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));
    commands.insert_resource(AmbientLight { brightness: 400.0, ..default() });

    commands.spawn((
        Camera3d::default(),
        Projection::from(PerspectiveProjection {
            fov: FIELD_OF_VIEW_DEG.to_radians(),
            ..default()
        }),
        Transform::default(),
        Eye,
    ));

    commands.spawn((
        Text::new(""),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(12.0),
            left: Val::Px(12.0),
            ..default()
        },
        Hud,
    ));
}

fn look(motion: Res<AccumulatedMouseMotion>, mut controls: ResMut<Controls>) {
    // Moving the mouse right turns right, which lowers yaw.
    controls.yaw -= motion.delta.x * MOUSE_SENSITIVITY;
    controls.pitch =
        (controls.pitch - motion.delta.y * MOUSE_SENSITIVITY).clamp(-PITCH_LIMIT, PITCH_LIMIT);
}

fn crouch_keys(keys: &ButtonInput<KeyCode>, check: fn(&ButtonInput<KeyCode>, KeyCode) -> bool) -> bool {
    check(keys, KeyCode::ControlLeft) || check(keys, KeyCode::KeyC)
}

/// Frames can outnumber fixed steps, so a press is remembered until `simulate` uses it.
fn latch_presses(
    keys: Res<ButtonInput<KeyCode>>,
    mouse: Res<ButtonInput<MouseButton>>,
    mut controls: ResMut<Controls>,
) {
    controls.jump |= keys.just_pressed(KeyCode::Space);
    controls.crouch |= crouch_keys(&keys, ButtonInput::just_pressed);
    controls.hang |= mouse.just_pressed(MouseButton::Right);
}

fn simulate(
    keys: Res<ButtonInput<KeyCode>>,
    mouse: Res<ButtonInput<MouseButton>>,
    time: Res<Time>,
    mut controls: ResMut<Controls>,
    mut sim: ResMut<Sim>,
) {
    let axis = |positive: KeyCode, negative: KeyCode| {
        f32::from(i8::from(keys.pressed(positive)) - i8::from(keys.pressed(negative)))
    };
    let button = |held: bool, pressed: bool| PilotButton { held: held || pressed, pressed };

    let input = PilotInput {
        move_forward: axis(KeyCode::KeyW, KeyCode::KeyS),
        move_right: axis(KeyCode::KeyD, KeyCode::KeyA),
        yaw: controls.yaw,
        pitch: controls.pitch,
        jump: button(keys.pressed(KeyCode::Space), controls.jump),
        crouch: button(crouch_keys(&keys, ButtonInput::pressed), controls.crouch),
        sprint: button(keys.pressed(KeyCode::ShiftLeft), false),
        hang: button(mouse.pressed(MouseButton::Right), controls.hang),
    };
    controls.jump = false;
    controls.crouch = false;
    controls.hang = false;

    let sim = &mut *sim;
    sim.previous_position = sim.state.position;
    sim.state = step(&sim.state, &input, &sim.tuning, &sim.world, time.delta_secs()).0;
}

fn quit_or_respawn(
    keys: Res<ButtonInput<KeyCode>>,
    mut sim: ResMut<Sim>,
    mut exit: EventWriter<AppExit>,
) {
    if keys.just_pressed(KeyCode::Escape) {
        exit.write(AppExit::Success);
    }
    if keys.just_pressed(KeyCode::KeyR) {
        sim.state = PilotState::spawn(course::SPAWN, &sim.tuning);
        sim.previous_position = sim.state.position;
    }
}

/// Pick up edits to pilot.toml while the gym is running.
fn reload_tuning(mut sim: ResMut<Sim>, time: Res<Time>, mut since_check: Local<f32>) {
    *since_check += time.delta_secs();
    if *since_check < 0.5 {
        return;
    }
    *since_check = 0.0;
    let stamp = std::fs::metadata(&sim.tuning_path).and_then(|m| m.modified()).ok();
    if stamp == sim.tuning_stamp {
        return;
    }
    let first_check = sim.tuning_stamp.is_none();
    sim.tuning_stamp = stamp;
    if first_check {
        return;
    }
    match pilot_tuning::load(&sim.tuning_path) {
        Ok(tuning) => {
            sim.tuning = tuning;
            info!("tuning reloaded");
        }
        Err(error) => warn!("tuning not reloaded: {error}"),
    }
}

fn sync_camera(
    sim: Res<Sim>,
    controls: Res<Controls>,
    fixed: Res<Time<Fixed>>,
    mut eye: Single<&mut Transform, With<Eye>>,
) {
    let state = &sim.state;
    let feet = sim.previous_position.lerp(state.position, fixed.overstep_fraction());
    let eye_position = feet + Vec3::Z * state.camera.eye_height;
    let mut transform = Transform::from_translation(to_bevy(eye_position))
        .looking_to(to_bevy(look_dir(controls.yaw, controls.pitch)), Vec3::Y);
    // pilot-core: positive roll is clockwise as the player sees it.
    transform.rotate_local_z(-state.camera.roll);
    **eye = transform;
}

fn update_hud(sim: Res<Sim>, mut text: Single<&mut Text, With<Hud>>) {
    let state = &sim.state;
    let speed = Vec3::new(state.velocity.x, state.velocity.y, 0.0).length();
    text.0 = format!(
        "{speed:5.1} m/s   {:5.1} km/h\n{:?}  {:.2} s\njumps used {}   slide boost in {:.1} s\n\n\
         WASD move   Shift sprint   Space jump   Ctrl or C crouch/slide\n\
         Right mouse wall-hang   R respawn   Esc quit",
        speed * 3.6,
        state.mode,
        state.mode_time,
        state.jumps_used,
        state.slide_boost_cooldown,
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    fn close(a: Vec3, b: Vec3) -> bool {
        (a - b).length() < 1e-5
    }

    #[test]
    fn pilot_axes_map_onto_bevy_axes() {
        assert!(close(to_bevy(Vec3::X), Vec3::X));
        assert!(close(to_bevy(Vec3::Z), Vec3::Y), "pilot up is Bevy up");
        assert!(close(to_bevy(Vec3::Y), -Vec3::Z));
    }

    #[test]
    fn rotations_convert_consistently_with_points() {
        let q = Quat::from_axis_angle(Vec3::new(0.3, -0.5, 0.8).normalize(), 0.9);
        let v = Vec3::new(1.0, 2.0, 3.0);
        assert!(close(to_bevy(q * v), rotation_to_bevy(q) * to_bevy(v)));
    }

    #[test]
    fn box_sizes_swap_y_and_z_and_double() {
        assert!(close(box_size_to_bevy(Vec3::new(1.0, 2.0, 3.0)), Vec3::new(2.0, 6.0, 4.0)));
    }

    #[test]
    fn look_direction_follows_yaw_and_pitch() {
        assert!(close(look_dir(0.0, 0.0), Vec3::X));
        assert!(close(look_dir(std::f32::consts::FRAC_PI_2, 0.0), Vec3::Y));
        assert!(look_dir(0.0, 0.5).z > 0.0, "positive pitch looks up");
    }
}
