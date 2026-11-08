//! The host-independent wall controller: the rules the game calls every frame.

mod common;

use common::*;
use glam::Vec3;
use pilot_core::wall::{touched_ground, wall_step, WallEnd, WallEvent, WallInput, WallReading, WallState};
use pilot_core::Tuning;

const UNIT: f32 = 0.0254;

/// A wall whose face looks along +Y, touching V.
fn wall_y() -> Option<WallReading> {
    Some(WallReading { normal: Vec3::Y, distance: 0.0 })
}

fn running(along: f32) -> (WallState, Vec3) {
    let state = WallState { running: true, normal: Vec3::Y, ..WallState::default() };
    (state, Vec3::new(along, 0.0, 0.0))
}

fn along_wall(v: Vec3) -> f32 {
    Vec3::new(v.x, 0.0, 0.0).length()
}

fn idle() -> WallInput {
    WallInput::default()
}

fn step(state: &WallState, v: Vec3, reading: Option<WallReading>, input: &WallInput, t: &Tuning) -> pilot_core::wall::WallStep {
    wall_step(state, v, reading, true, input, t, DT)
}

#[test]
fn contact_keeps_only_the_speed_along_the_wall() {
    let t = tuning();
    let angle = 61.5f32.to_radians();
    let speed = 260.0 * UNIT;
    let into_wall = Vec3::new(speed * angle.cos(), -speed * angle.sin(), 0.0);
    let out = step(&WallState::default(), into_wall, wall_y(), &idle(), &t);
    assert!(out.state.running);
    assert_eq!(out.events, vec![WallEvent::Started]);
    assert!((along_wall(out.velocity) - speed * angle.cos()).abs() < 1e-3, "kept {}", along_wall(out.velocity));
    assert!(out.velocity.y.abs() < 1e-5, "nothing left going into the wall");
}

#[test]
fn a_run_never_slows_a_faster_runner() {
    let t = tuning();
    let (state, v) = running(t.wallrun_speed + 2.0);
    let out = step(&state, v, wall_y(), &idle(), &t);
    assert!(out.state.running);
    assert!((along_wall(out.velocity) - (t.wallrun_speed + 2.0)).abs() < 1e-4);
}

#[test]
fn a_kick_keeps_the_along_wall_speed_and_adds_the_push() {
    let t = tuning();
    let (state, v) = running(8.0);
    let jump = WallInput { jump_pressed: true, ..idle() };
    let out = step(&state, v, wall_y(), &jump, &t);
    assert!(!out.state.running);
    assert_eq!(out.ended, Some(WallEnd::Kicked));
    assert!(out.events.contains(&WallEvent::Kicked));
    assert!(along_wall(out.velocity) >= 8.0 - 1e-4, "along-wall {}", along_wall(out.velocity));
    assert!((out.velocity.y - t.wall_kick_out_speed).abs() < 1e-4, "outward {}", out.velocity.y);
}

#[test]
fn a_new_run_refreshes_air_moves_and_the_same_wall_does_not() {
    let t = tuning();
    let fast = Vec3::new(8.0, -1.0, 0.0);
    let started = step(&WallState::default(), fast, wall_y(), &idle(), &t);
    assert!(started.state.running && started.refresh_air_moves);

    let left = step(&started.state, Vec3::new(8.0, 0.0, 0.0), None, &idle(), &t);
    assert!(!left.state.running);
    assert_eq!(left.state.time_since_wall, 0.0);

    let again = step(&left.state, fast, wall_y(), &idle(), &t);
    assert!(!again.state.running, "the same wall cannot be grabbed straight away");
    assert!(!again.refresh_air_moves);

    let landed = touched_ground(&left.state);
    let after_landing = step(&landed, fast, wall_y(), &idle(), &t);
    assert!(after_landing.state.running && after_landing.refresh_air_moves);
}

#[test]
fn losing_the_wall_ends_the_run_with_speed_kept() {
    let t = tuning();
    let (state, v) = running(9.0);
    let out = step(&state, v, None, &idle(), &t);
    assert!(!out.state.running);
    assert_eq!(out.events, vec![WallEvent::Ended]);
    assert_eq!(out.ended, Some(WallEnd::Lost));
    assert!((along_wall(out.velocity) - 9.0).abs() < 1e-4);
}

#[test]
fn a_hitch_is_several_small_steps() {
    let t = tuning();
    let (state, v) = running(7.0);
    let one = wall_step(&state, v, wall_y(), true, &idle(), &t, 0.1);
    let mut s = state;
    let mut velocity = v;
    for _ in 0..5 {
        let out = wall_step(&s, velocity, wall_y(), true, &idle(), &t, 0.02);
        s = out.state;
        velocity = out.velocity;
    }
    assert!((one.velocity - velocity).length() < 0.02, "one {:?} five {:?}", one.velocity, velocity);
    assert!((one.state.elapsed - 0.1).abs() < 1e-5);
    assert!(along_wall(one.velocity) <= t.wallrun_speed.max(7.0) + 1e-4, "no spike");
}

#[test]
fn no_reading_and_not_running_does_nothing() {
    let t = tuning();
    let v = Vec3::new(3.0, 2.0, -1.0);
    let out = step(&WallState::default(), v, None, &idle(), &t);
    assert_eq!(out.velocity, v);
    assert!(out.events.is_empty());
    assert!(!out.state.running);
    assert!(!out.refresh_air_moves);
}

#[test]
fn a_run_does_not_start_close_to_the_ground() {
    let t = tuning();
    let out = wall_step(&WallState::default(), Vec3::new(8.0, -1.0, 0.0), wall_y(), false, &idle(), &t, DT);
    assert!(!out.state.running);
    assert!(out.events.is_empty());
}

#[test]
fn entry_lifts_vertical_speed_by_the_boost_but_not_past_it() {
    let t = tuning();
    let touch = |vz: f32| step(&WallState::default(), Vec3::new(8.0, -1.0, vz), wall_y(), &idle(), &t).velocity.z;
    assert!((touch(1.0) - t.wallrun_entry_up_speed).abs() < 1e-4, "rising V is capped at the boost");
    assert!((touch(-7.0) - (-7.0 + t.wallrun_entry_up_speed)).abs() < 1e-4, "falling V gets the boost added");
}

#[test]
fn along_wall_speed_closes_on_the_ceiling_exponentially() {
    let t = tuning();
    let (state, v) = running(5.0);
    let out = step(&state, v, wall_y(), &idle(), &t);
    let expected = t.wallrun_speed - (t.wallrun_speed - 5.0) * (-t.wallrun_accel_rate * DT).exp();
    assert!((along_wall(out.velocity) - expected).abs() < 1e-4, "got {} expected {expected}", along_wall(out.velocity));
}

#[test]
fn vertical_speed_is_damped_and_sinks_more_the_longer_the_run() {
    let t = tuning();
    let mut early = running(8.0).0;
    early.elapsed = 0.0;
    let mut late = early;
    late.elapsed = 1.0;
    let rising = Vec3::new(8.0, 0.0, 3.0);
    let e = step(&early, rising, wall_y(), &idle(), &t).velocity.z;
    let l = step(&late, rising, wall_y(), &idle(), &t).velocity.z;
    // Vertical speed is pulled toward a target that starts at a small lift and
    // falls the longer the run lasts.
    let toward = |target: f32| 3.0 + t.wallrun_vertical_damping * (target - 3.0) * DT;
    let expected_early = toward(t.wallrun_lift);
    let expected_late = toward(t.wallrun_lift - t.wallrun_sink_ramp * 1.0);
    assert!((e - expected_early).abs() < 1e-4, "early {e} expected {expected_early}");
    assert!((l - expected_late).abs() < 1e-4, "late {l} expected {expected_late}");
}

#[test]
fn running_out_of_time_pushes_v_away_from_the_wall() {
    let t = tuning();
    let (mut state, v) = running(8.0);
    state.elapsed = t.wallrun_time_limit;
    let out = step(&state, v, wall_y(), &idle(), &t);
    assert_eq!(out.ended, Some(WallEnd::TimedOut));
    assert!((out.velocity.y - t.wall_fall_off_push).abs() < 1e-4, "outward {}", out.velocity.y);
    assert!((along_wall(out.velocity) - 8.0).abs() < 1e-4);
}

#[test]
fn the_kick_adds_look_speed_along_the_wall_only() {
    let t = tuning();
    let (state, v) = running(8.0);
    // Looking 30 degrees off the wall, toward it.
    let jump = WallInput { jump_pressed: true, yaw: -30f32.to_radians(), ..idle() };
    let out = step(&state, v, wall_y(), &jump, &t);
    let along_gain = t.wall_kick_look_speed * 30f32.to_radians().cos();
    assert!((out.velocity.x - (8.0 + along_gain)).abs() < 1e-4, "along {}", out.velocity.x);
    assert!((out.velocity.y - t.wall_kick_out_speed).abs() < 1e-4, "outward is exactly the kick: {}", out.velocity.y);
    assert!((out.velocity.z - t.wall_kick_up_speed).abs() < 1e-4);
}

#[test]
fn a_run_only_starts_on_a_wall_within_grab_distance() {
    let t = tuning();
    let near = Some(WallReading { normal: Vec3::Y, distance: t.wallrun_grab_distance * 0.5 });
    let between = Some(WallReading { normal: Vec3::Y, distance: (t.wallrun_grab_distance + t.wallrun_stick_distance) * 0.5 });
    let fast = Vec3::new(8.0, 0.0, 0.0);
    assert!(step(&WallState::default(), fast, near, &idle(), &t).state.running);
    assert!(!step(&WallState::default(), fast, between, &idle(), &t).state.running, "too far to grab");
    let (state, v) = running(8.0);
    assert!(step(&state, v, between, &idle(), &t).state.running, "but close enough to stay on");
}

#[test]
fn no_run_starts_moving_away_from_the_wall() {
    let t = tuning();
    // Walking off a roof past the facade below.
    let away = Vec3::new(4.0, 3.0, -2.0);
    assert!(!step(&WallState::default(), away, wall_y(), &idle(), &t).state.running);
}

#[test]
fn a_slight_drift_away_still_starts() {
    let t = tuning();
    let drift = Vec3::new(8.0, t.wallrun_max_away_speed * 0.5, 0.0);
    assert!(step(&WallState::default(), drift, wall_y(), &idle(), &t).state.running);
}

#[test]
fn no_run_starts_while_steering_away_from_the_wall() {
    let t = tuning();
    // Facing along the wall, holding toward open air.
    let steer_away = WallInput { move_right: -1.0, ..WallInput::default() };
    let along = Vec3::new(8.0, 0.0, 0.0);
    assert!(!step(&WallState::default(), along, wall_y(), &steer_away, &t).state.running);
}
