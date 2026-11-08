mod common;

use common::*;
use glam::Vec3;
use pilot_core::{Event, Input, Mode};
use world_parry::BoxWorld;

fn high(t: &pilot_core::Tuning, velocity: Vec3) -> pilot_core::PilotState {
    airborne(Vec3::new(0.0, 0.0, 50.0), velocity, t)
}

#[test]
fn gravity_pulls_down_each_tick() {
    let t = tuning();
    let (s, _) = run(&flat_world(), high(&t, Vec3::ZERO), &t, 1, |_| Input::default());
    assert!((s.velocity.z + t.gravity * DT).abs() < 1e-6);
    assert!(s.position.z < 50.0);
    assert_eq!(s.mode, Mode::Airborne);
}

#[test]
fn a_jump_peaks_near_the_ballistic_height() {
    let t = tuning();
    let world = flat_world();
    let jump = Input { jump: Button::PRESSED, ..Input::default() };
    let (mut s, _) = run(&world, standing(&t), &t, 1, |_| jump);
    let mut peak = s.position.z;
    for _ in 0..120 {
        s = run(&world, s, &t, 1, |_| Input::default()).0;
        peak = peak.max(s.position.z);
    }
    let expected = t.jump_speed * t.jump_speed / (2.0 * t.gravity);
    assert!((peak - expected).abs() <= t.jump_speed * DT + t.collision_skin, "peak {peak}, expected {expected}");
    assert_eq!(s.mode, Mode::Grounded, "it should have landed again");
}

#[test]
fn the_double_jump_works_once() {
    let t = tuning();
    let world = flat_world();
    let press = Input { jump: Button::PRESSED, ..Input::default() };
    let start = high(&t, Vec3::new(0.0, 0.0, -3.0));

    let (after_one, events) = run(&world, start, &t, 1, |_| press);
    assert_eq!(events, vec![Event::DoubleJumped]);
    assert_eq!(after_one.velocity.z, t.double_jump_speed);
    assert_eq!(after_one.jumps_used, 2);

    let (after_two, events) = run(&world, after_one, &t, 1, |_| press);
    assert!(events.is_empty(), "no jumps left");
    assert!(after_two.velocity.z < t.double_jump_speed, "gravity still applied");
    assert_eq!(after_two.mode, Mode::Airborne);
}

#[test]
fn the_double_jump_pushes_toward_the_held_direction() {
    let t = tuning();
    let press_right = Input { move_right: 1.0, jump: Button::PRESSED, ..Input::default() };
    let (s, _) = run(&flat_world(), high(&t, Vec3::ZERO), &t, 1, |_| press_right);
    assert!(s.velocity.y < 0.0, "right is -Y");
    let speed = hspeed(&s);
    assert!(speed >= t.double_jump_boost.min(t.sprint_speed) - 1e-3, "speed {speed}");
    assert!(speed <= t.double_jump_boost + t.air_wish_cap + 1e-3, "speed {speed}");
}

#[test]
fn the_double_jump_boost_never_exceeds_the_speed_already_held() {
    let t = tuning();
    let fast = t.sprint_speed + 3.0;
    let press = Input { jump: Button::PRESSED, ..forward() };
    let (s, events) = run(&flat_world(), high(&t, Vec3::new(fast, 0.0, 0.0)), &t, 1, |_| press);
    assert_eq!(events, vec![Event::DoubleJumped]);
    assert!(hspeed(&s) <= fast + 1e-3, "speed {}", hspeed(&s));
    assert!(hspeed(&s) >= fast - 1e-3, "a forward double jump keeps the speed");
}

#[test]
fn air_control_from_rest_is_limited_to_the_air_cap() {
    let t = tuning();
    let (s, _) = run(&flat_world(), high(&t, Vec3::ZERO), &t, 60, |_| forward());
    assert!(hspeed(&s) > 0.0);
    assert!(hspeed(&s) <= t.air_wish_cap + 1e-4, "speed {}", hspeed(&s));
}

#[test]
fn strafing_sideways_in_the_air_gains_speed() {
    let t = tuning();
    let strafe = Input { move_right: 1.0, ..Input::default() };
    let start = high(&t, Vec3::new(t.sprint_speed, 0.0, 0.0));
    let (s, _) = run(&flat_world(), start, &t, 30, |_| strafe);
    assert!(hspeed(&s) > t.sprint_speed, "speed {}", hspeed(&s));
    assert!(hspeed(&s) < t.sprint_speed + t.air_wish_cap);
    assert_eq!(s.velocity.x, t.sprint_speed, "the forward component is untouched");
}

#[test]
fn landing_reports_impact_and_restores_jumps() {
    let t = tuning();
    let drop = 2.0;
    let start = airborne(Vec3::new(0.0, 0.0, drop), Vec3::ZERO, &t);
    let (s, events, ticks) =
        run_until(&flat_world(), start, &t, 300, |_| Input::default(), |s| s.mode == Mode::Grounded);
    assert!(ticks < 300, "never landed");
    assert_eq!(s.jumps_used, 0);
    assert_eq!(s.velocity.z, 0.0);
    assert_eq!(s.mode_time, 0.0);
    let expected = (2.0 * t.gravity * drop).sqrt();
    match events.as_slice() {
        [Event::Landed { impact_speed }] => {
            assert!((impact_speed - expected).abs() < expected * 0.1, "impact {impact_speed}")
        }
        other => panic!("expected one Landed event, got {other:?}"),
    }
}

#[test]
fn a_ceiling_is_not_a_floor() {
    let t = tuning();
    let mut world = BoxWorld::new();
    world.add_box(Vec3::new(0.0, 0.0, 4.0), Vec3::new(5.0, 5.0, 0.5));
    let start = airborne(Vec3::new(0.0, 0.0, 1.0), Vec3::new(0.0, 0.0, 8.0), &t);
    let (s, events) = run(&world, start, &t, 20, |_| Input::default());
    assert_eq!(s.mode, Mode::Airborne);
    assert!(events.is_empty());
    assert!(s.velocity.z < 0.0, "it hit the ceiling and is falling");
}

#[test]
fn touching_a_slope_while_still_rising_is_not_a_landing() {
    let t = tuning();
    let mut world = BoxWorld::new();
    // A 30 degree ramp rising toward +X. Its surface is at z = 0.577 at x = 0.
    let tilt = glam::Quat::from_rotation_y(-30f32.to_radians());
    world.add_rotated_box(Vec3::ZERO, Vec3::new(30.0, 30.0, 0.5), tilt);
    let start = airborne(Vec3::new(0.0, 0.0, 0.7), Vec3::new(8.0, 0.0, 2.0), &t);
    let (s, events) = run(&world, start, &t, 6, |_| Input::default());
    assert_eq!(s.mode, Mode::Airborne, "still rising, so still airborne");
    assert!(events.is_empty());
    assert!(s.velocity.z > 2.0, "the ramp deflected the motion upward");
    assert!(s.position.z > 0.7);
}

#[test]
fn a_quick_jump_after_landing_keeps_the_speed_and_a_late_one_does_not() {
    let t = tuning();
    let world = flat_world();
    let fast = t.sprint_speed + 3.0;
    let start = airborne(Vec3::new(0.0, 0.0, 0.05), Vec3::new(fast, 0.0, -1.0), &t);
    let (landed, _, _) =
        run_until(&world, start, &t, 60, |_| forward(), |s| s.mode == Mode::Grounded);
    assert_eq!(landed.mode, Mode::Grounded);

    let jump = Input { jump: Button::PRESSED, ..forward() };
    let quick_wait = ((t.landing_friction_grace / DT) as usize).saturating_sub(2);
    let (waited, _) = run(&world, landed, &t, quick_wait, |_| forward());
    let (hopped, _) = run(&world, waited, &t, 1, |_| jump);
    assert_eq!(hopped.mode, Mode::Airborne);
    assert!(hspeed(&hopped) >= fast - 1e-3, "quick hop speed {}", hspeed(&hopped));

    let late_wait = (t.landing_friction_grace / DT) as usize + 10;
    let (waited, _) = run(&world, landed, &t, late_wait, |_| forward());
    let (hopped, _) = run(&world, waited, &t, 1, |_| jump);
    assert_eq!(hopped.mode, Mode::Airborne);
    assert!(hspeed(&hopped) < fast - 0.1, "late hop speed {}", hspeed(&hopped));
}

#[test]
fn walking_off_a_ledge_leaves_exactly_one_double_jump() {
    let t = tuning();
    let mut world = BoxWorld::new();
    world.add_box(Vec3::new(-1.5, 0.0, 49.5), Vec3::new(3.5, 5.0, 0.5));
    let start = pilot_core::PilotState::spawn(Vec3::new(0.0, 0.0, 50.0 + t.collision_skin), &t);
    let (falling, _, _) =
        run_until(&world, start, &t, 600, |_| forward(), |s| s.mode == Mode::Airborne);
    assert_eq!(falling.mode, Mode::Airborne);
    let press = Input { jump: Button::PRESSED, ..Input::default() };
    let (s, events) = run(&world, falling, &t, 1, |_| press);
    assert_eq!(events, vec![Event::DoubleJumped]);
    let (_, events) = run(&world, s, &t, 1, |_| press);
    assert!(events.is_empty());
}

#[test]
fn landing_on_a_slope_leaves_no_vertical_speed() {
    let t = tuning();
    let mut world = BoxWorld::new();
    world.add_rotated_box(Vec3::ZERO, Vec3::new(30.0, 30.0, 0.5), glam::Quat::from_rotation_y(20f32.to_radians()));
    let start = airborne(Vec3::new(0.0, 0.0, 2.0), Vec3::ZERO, &t);
    let (s, events, ticks) =
        run_until(&world, start, &t, 300, |_| Input::default(), |s| s.mode == Mode::Grounded);
    assert!(ticks < 300, "never landed");
    assert!(matches!(events.as_slice(), [Event::Landed { .. }]), "{events:?}");
    assert_eq!(s.velocity.z, 0.0, "sliding along the slope must not leave vertical speed behind");
}
