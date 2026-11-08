mod common;

use common::*;
use glam::Vec3;
use pilot_core::{Input, Mode, PilotState, Tuning};
use world_parry::BoxWorld;

fn wall_world() -> BoxWorld {
    let mut world = flat_world();
    world.add_box(Vec3::new(0.0, -2.5, 10.0), Vec3::new(100.0, 0.5, 10.0));
    world
}

/// Wallrunning on the wall at y = -2 (normal +Y), moving along `direction` (+1 or -1 on X).
fn running(t: &Tuning, direction: f32) -> PilotState {
    let y = -2.0 + t.capsule_radius + t.collision_skin;
    let mut s = airborne(Vec3::new(0.0, y, 12.0), Vec3::new(direction * t.sprint_speed, 0.0, 0.0), t);
    s.mode = Mode::Wallrunning;
    s.wall_normal = Vec3::Y;
    s
}

#[test]
fn the_eye_lowers_at_the_configured_rate_when_crouching() {
    let t = tuning();
    let world = flat_world();
    let crouch = Input { crouch: Button::HELD, ..Input::default() };
    let (one, _) = run(&world, standing(&t), &t, 1, |_| crouch);
    assert!((one.camera.eye_height - (t.stand_eye_height - t.eye_height_rate * DT)).abs() < 1e-5);

    let (settled, _) = run(&world, standing(&t), &t, 120, |_| crouch);
    assert_eq!(settled.camera.eye_height, t.crouch_eye_height);

    let (risen, _) = run(&world, settled, &t, 120, |_| Input::default());
    assert_eq!(risen.camera.eye_height, t.stand_eye_height);
}

#[test]
fn a_slide_uses_its_own_eye_height() {
    let t = tuning();
    let mut s = standing(&t);
    s.mode = Mode::Sliding;
    s.crouched = true;
    s.velocity = Vec3::new(t.slide_exit_speed + 8.0, 0.0, 0.0);
    let crouch = Input { crouch: Button::HELD, ..Input::default() };
    let ticks = ((t.stand_eye_height - t.slide_eye_height) / (t.eye_height_rate * DT)).ceil() as usize + 2;
    let (s, _) = run(&flat_world(), s, &t, ticks, |_| crouch);
    assert_eq!(s.mode, Mode::Sliding);
    assert_eq!(s.camera.eye_height, t.slide_eye_height);
}

#[test]
fn a_wallrun_rolls_the_view_away_from_the_wall() {
    let t = tuning();
    let world = wall_world();
    let step_size = t.roll_rate_deg.to_radians() * DT;
    let full = t.wallrun_roll_deg.to_radians();

    // Facing +X, the wall is on the right: lean left, which is negative roll.
    let (right_wall, _) = run(&world, running(&t, 1.0), &t, 1, |_| Input::default());
    assert_eq!(right_wall.mode, Mode::Wallrunning);
    assert!((right_wall.camera.roll + step_size.min(full)).abs() < 1e-5, "roll {}", right_wall.camera.roll);

    // Facing -X, the same wall is on the left: lean right, positive roll.
    let facing_back = Input { yaw: std::f32::consts::PI, ..Input::default() };
    let (left_wall, _) = run(&world, running(&t, -1.0), &t, 1, |_| facing_back);
    assert_eq!(left_wall.mode, Mode::Wallrunning);
    assert!((left_wall.camera.roll - step_size.min(full)).abs() < 1e-5, "roll {}", left_wall.camera.roll);

    let ticks = (full / step_size).ceil() as usize + 2;
    let (leaned, _) = run(&world, running(&t, 1.0), &t, ticks, |_| Input::default());
    assert!((leaned.camera.roll + full).abs() < 1e-5, "it stops at the full lean");
}

#[test]
fn the_view_levels_out_after_the_wallrun() {
    let t = tuning();
    let world = wall_world();
    let mut leaning = airborne(Vec3::new(0.0, 5.0, 30.0), Vec3::ZERO, &t);
    leaning.camera.roll = -t.wallrun_roll_deg.to_radians();
    let (one, _) = run(&world, leaning, &t, 1, |_| Input::default());
    assert!(one.camera.roll > leaning.camera.roll, "it starts levelling");
    let (level, _) = run(&world, leaning, &t, 60, |_| Input::default());
    assert_eq!(level.camera.roll, 0.0);
    assert_eq!(level.mode, Mode::Airborne);
}
