mod common;

use common::*;
use glam::{Quat, Vec3};
use pilot_core::{Event, Input, Mode, PilotState, Tuning};
use world_parry::BoxWorld;

/// Standing on flat ground, already moving along +X, past the friction grace.
fn moving(t: &Tuning, speed: f32) -> PilotState {
    let mut s = standing(t);
    s.velocity = Vec3::new(speed, 0.0, 0.0);
    s.mode_time = 1.0;
    s
}

fn sliding(t: &Tuning, speed: f32) -> PilotState {
    let mut s = moving(t, speed);
    s.mode = Mode::Sliding;
    s.crouched = true;
    s.slide_boost_cooldown = t.slide_boost_cooldown;
    s
}

fn press_crouch() -> Input {
    Input { crouch: Button::PRESSED, ..sprint() }
}

fn hold_crouch() -> Input {
    Input { crouch: Button::HELD, ..Input::default() }
}

#[test]
fn crouching_at_slide_speed_starts_a_boosted_slide() {
    let t = tuning();
    let start = moving(&t, t.slide_entry_speed);
    let (s, events) = run(&flat_world(), start, &t, 1, |_| press_crouch());
    assert_eq!(s.mode, Mode::Sliding);
    assert_eq!(events, vec![Event::SlideStarted]);
    assert!(s.crouched);
    let boosted = (t.slide_entry_speed + t.slide_boost_speed).min(t.slide_boost_max_speed);
    assert!((hspeed(&s) - boosted).abs() < 1e-4);
    assert_eq!(s.slide_boost_cooldown, t.slide_boost_cooldown);
    assert!(s.position.x > 0.0);
}

#[test]
fn crouching_below_slide_speed_is_only_a_crouch() {
    let t = tuning();
    let start = moving(&t, t.slide_entry_speed - 0.05);
    let (s, events) = run(&flat_world(), start, &t, 1, |_| press_crouch());
    assert_eq!(s.mode, Mode::Grounded);
    assert!(events.is_empty());
    assert!(s.crouched);
    assert!(s.position.x > 0.0);
}

#[test]
fn the_boost_waits_for_its_cooldown() {
    let t = tuning();
    let mut cooling = moving(&t, t.slide_entry_speed);
    cooling.slide_boost_cooldown = 1.0;
    let (s, events) = run(&flat_world(), cooling, &t, 1, |_| press_crouch());
    assert_eq!(s.mode, Mode::Sliding);
    assert_eq!(events, vec![Event::SlideStarted]);
    assert!((hspeed(&s) - t.slide_entry_speed).abs() < 1e-4, "no boost while cooling down");

    let mut ready = moving(&t, t.slide_entry_speed);
    ready.slide_boost_cooldown = DT * 0.5;
    let (s, _) = run(&flat_world(), ready, &t, 1, |_| press_crouch());
    assert!(hspeed(&s) > t.slide_entry_speed + t.slide_boost_speed * 0.5, "boost is back");
}

#[test]
fn a_slide_loses_speed_at_the_slide_deceleration() {
    let t = tuning();
    let speed = t.slide_exit_speed + 4.0;
    let (s, events) = run(&flat_world(), sliding(&t, speed), &t, 1, |_| hold_crouch());
    assert_eq!(s.mode, Mode::Sliding);
    assert!(events.is_empty());
    let decel = t.slide_decel + t.slide_decay_rate * speed;
    assert!((hspeed(&s) - (speed - decel * DT)).abs() < 1e-4, "speed {}", hspeed(&s));
}

#[test]
fn a_slide_ends_once_it_drops_below_exit_speed() {
    let t = tuning();
    let world = flat_world();
    let at_exit = sliding(&t, t.slide_exit_speed);
    let (s, _) = run(&world, at_exit, &t, 1, |_| hold_crouch());
    assert_eq!(s.mode, Mode::Sliding, "exactly at exit speed the slide continues");

    let below = sliding(&t, t.slide_exit_speed - 0.01);
    let (s, events) = run(&world, below, &t, 1, |_| hold_crouch());
    assert_eq!(s.mode, Mode::Grounded);
    assert_eq!(events, vec![Event::SlideEnded]);
    assert!(s.crouched, "still crouched while the button is held");
}

#[test]
fn a_long_slide_ends_by_itself_with_one_end_event() {
    let t = tuning();
    let start = sliding(&t, t.slide_exit_speed + 4.0);
    let (s, events, ticks) =
        run_until(&flat_world(), start, &t, 1200, |_| hold_crouch(), |s| s.mode == Mode::Grounded);
    assert!(ticks < 1200, "the slide never ended");
    assert_eq!(events, vec![Event::SlideEnded]);
    assert!(hspeed(&s) < t.slide_exit_speed);
}

#[test]
fn releasing_crouch_ends_the_slide() {
    let t = tuning();
    let start = sliding(&t, t.slide_exit_speed + 4.0);
    let (s, events) = run(&flat_world(), start, &t, 1, |_| Input::default());
    assert_eq!(s.mode, Mode::Grounded);
    assert_eq!(events, vec![Event::SlideEnded]);
    assert!(s.position.x > 0.0);
}

#[test]
fn a_slide_hop_keeps_all_the_horizontal_speed() {
    let t = tuning();
    let speed = t.slide_exit_speed + 6.0;
    let hop = Input { jump: Button::PRESSED, crouch: Button::HELD, ..Input::default() };
    let (s, events) = run(&flat_world(), sliding(&t, speed), &t, 1, |_| hop);
    assert_eq!(s.mode, Mode::Airborne);
    assert_eq!(events, vec![Event::Jumped, Event::SlideEnded]);
    assert!((hspeed(&s) - speed).abs() < 1e-4, "speed {}", hspeed(&s));
    assert_eq!(s.velocity.z, t.jump_speed);
    assert_eq!(s.jumps_used, 1);
}

#[test]
fn steering_turns_a_slide() {
    let t = tuning();
    let steer = Input { move_right: 1.0, crouch: Button::HELD, ..Input::default() };
    let (s, _) = run(&flat_world(), sliding(&t, t.slide_exit_speed + 6.0), &t, 20, |_| steer);
    assert_eq!(s.mode, Mode::Sliding);
    assert!(s.velocity.y < 0.0, "right is -Y");
    assert!(s.velocity.x > 0.0);
}

#[test]
fn sliding_downhill_is_faster_than_sliding_on_the_flat() {
    let t = tuning();
    let speed = t.slide_exit_speed + 6.0;
    let (flat, _) = run(&flat_world(), sliding(&t, speed), &t, 30, |_| hold_crouch());

    // A 15 degree ramp descending toward +X; its surface is at z = 0.5176 at x = 0.
    let mut ramp = BoxWorld::new();
    ramp.add_rotated_box(Vec3::ZERO, Vec3::new(60.0, 60.0, 0.5), Quat::from_rotation_y(15f32.to_radians()));
    let mut start = sliding(&t, speed);
    start.position = Vec3::new(0.0, 0.0, 0.5176 + 0.03);
    let (downhill, _) = run(&ramp, start, &t, 30, |_| hold_crouch());

    assert_eq!(flat.mode, Mode::Sliding);
    assert_eq!(downhill.mode, Mode::Sliding);
    assert!(downhill.position.z < 0.0, "it followed the ramp down");
    assert!(hspeed(&downhill) > hspeed(&flat) + 0.5, "downhill {} flat {}", hspeed(&downhill), hspeed(&flat));
}

#[test]
fn sliding_off_a_ledge_becomes_airborne() {
    let t = tuning();
    let mut world = BoxWorld::new();
    world.add_box(Vec3::new(-1.5, 0.0, -0.5), Vec3::new(3.5, 5.0, 0.5));
    let start = sliding(&t, t.slide_exit_speed + 6.0);
    let (s, events, ticks) =
        run_until(&world, start, &t, 300, |_| hold_crouch(), |s| s.mode != Mode::Sliding);
    assert!(ticks < 300);
    assert_eq!(s.mode, Mode::Airborne);
    assert_eq!(events, vec![Event::SlideEnded]);
    assert_eq!(s.jumps_used, 1);
}

#[test]
fn landing_with_crouch_held_goes_straight_into_a_slide() {
    let t = tuning();
    let world = flat_world();
    let fast = t.slide_entry_speed + 1.0;
    let falling = |speed: f32| airborne(Vec3::new(0.0, 0.0, 0.05), Vec3::new(speed, 0.0, -2.0), &t);
    let landed = |s: &PilotState| s.mode != Mode::Airborne;

    let (s, events, _) = run_until(&world, falling(fast), &t, 60, |_| hold_crouch(), landed);
    assert_eq!(s.mode, Mode::Sliding);
    assert!(matches!(events.as_slice(), [Event::Landed { .. }, Event::SlideStarted]), "{events:?}");
    assert!((hspeed(&s) - (fast + t.slide_boost_speed).min(t.slide_boost_max_speed)).abs() < 1e-3);

    let (s, _, _) = run_until(&world, falling(fast), &t, 60, |_| Input::default(), landed);
    assert_eq!(s.mode, Mode::Grounded, "no crouch, no slide");

    let slow = t.slide_entry_speed - 1.0;
    let (s, _, _) = run_until(&world, falling(slow), &t, 60, |_| hold_crouch(), landed);
    assert_eq!(s.mode, Mode::Grounded, "too slow to slide");
    assert!(s.crouched);
}

#[test]
fn a_crouch_that_is_only_held_does_not_start_a_slide() {
    let t = tuning();
    let start = moving(&t, t.slide_entry_speed + 1.0);
    let (s, events) = run(&flat_world(), start, &t, 1, |_| hold_crouch());
    assert_eq!(s.mode, Mode::Grounded, "a slide needs a fresh crouch press on the ground");
    assert!(events.is_empty());
    assert!(s.crouched);
    assert!(s.position.x > 0.0);
}

#[test]
fn the_boost_stops_at_its_ceiling_and_never_slows_a_faster_slide() {
    let t = tuning();
    let world = flat_world();
    let near = t.slide_boost_max_speed - t.slide_boost_speed * 0.25;
    let (s, _) = run(&world, moving(&t, near), &t, 1, |_| press_crouch());
    assert_eq!(s.mode, Mode::Sliding);
    assert!((hspeed(&s) - t.slide_boost_max_speed).abs() < 1e-4, "capped: {}", hspeed(&s));

    let above = t.slide_boost_max_speed + 2.0;
    let (s, _) = run(&world, moving(&t, above), &t, 1, |_| press_crouch());
    assert_eq!(s.mode, Mode::Sliding);
    assert!((hspeed(&s) - above).abs() < 1e-4, "already faster, so untouched: {}", hspeed(&s));
}

#[test]
fn a_slide_without_a_boost_still_restarts_the_cooldown() {
    let t = tuning();
    let mut cooling = moving(&t, t.slide_entry_speed);
    cooling.slide_boost_cooldown = DT * 3.0;
    let (s, _) = run(&flat_world(), cooling, &t, 1, |_| press_crouch());
    assert_eq!(s.mode, Mode::Sliding);
    assert!((hspeed(&s) - t.slide_entry_speed).abs() < 1e-4, "no boost yet");
    assert_eq!(s.slide_boost_cooldown, t.slide_boost_cooldown, "the wait starts over");
}

#[test]
fn a_slide_that_drops_below_sprint_speed_ends_the_sprint() {
    let t = tuning();
    let world = flat_world();
    let mut fast = sliding(&t, t.sprint_speed + 2.0);
    fast.sprinting = true;
    let slide_on = Input { move_forward: 1.0, ..hold_crouch() };
    let (s, _) = run(&world, fast, &t, 1, |_| slide_on);
    assert!(s.sprinting, "still above sprint speed when the step began");

    let mut slow = sliding(&t, t.sprint_speed - 0.2);
    slow.sprinting = true;
    let (s, _) = run(&world, slow, &t, 1, |_| slide_on);
    assert_eq!(s.mode, Mode::Sliding);
    assert!(!s.sprinting);
}

// Slide math the game host calls directly, without a pilot-core world.

#[test]
fn a_boost_adds_speed_up_to_the_cap() {
    let t = tuning();
    assert!((pilot_core::slide::boosted_speed(5.0, &t) - 8.81).abs() < 1e-3);
    assert!((pilot_core::slide::boosted_speed(8.0, &t) - 10.16).abs() < 1e-3);
    assert_eq!(pilot_core::slide::boosted_speed(11.0, &t), 11.0, "a boost never slows");
}

#[test]
fn a_flat_slide_loses_speed_by_the_titanfall_curve() {
    let t = tuning();
    let v = pilot_core::slide::slide_velocity(Vec3::new(10.0, 0.0, 0.0), Vec3::ZERO, Some(Vec3::Z), &t, 0.1);
    let expected = 10.0 - (t.slide_decel + t.slide_decay_rate * 10.0) * 0.1;
    assert!((v.length() - expected).abs() < 1e-3);
    assert!((expected - (10.0 - (1.27 + 0.3567 * 10.0) * 0.1)).abs() < 1e-3);
}

#[test]
fn downhill_slides_faster_than_flat() {
    let t = tuning();
    let slope = Vec3::new(0.3, 0.0, 1.0).normalize();
    let down = pilot_core::slide::slide_velocity(Vec3::new(8.0, 0.0, 0.0), Vec3::ZERO, Some(slope), &t, 0.1);
    let flat = pilot_core::slide::slide_velocity(Vec3::new(8.0, 0.0, 0.0), Vec3::ZERO, Some(Vec3::Z), &t, 0.1);
    assert!(down.length() > flat.length());
    assert_eq!(down.z, 0.0, "horizontal only");
}
