mod common;

use common::*;
use glam::Vec3;
use pilot_core::{Event, Input, Mode};
use world_parry::BoxWorld;

#[test]
fn walking_settles_at_walk_speed() {
    let t = tuning();
    let (s, _) = run(&flat_world(), standing(&t), &t, 180, |_| forward());
    assert_eq!(s.mode, Mode::Grounded);
    assert!((hspeed(&s) - t.walk_speed).abs() < 1e-3, "speed {}", hspeed(&s));
    assert!(s.position.x > 1.0);
    assert!(s.position.y.abs() < 1e-3);
    assert!((s.position.z - t.collision_skin).abs() < 1e-3);
}

#[test]
fn sprint_needs_the_button_and_forward_input() {
    let t = tuning();
    let world = flat_world();
    let (s, _) = run(&world, standing(&t), &t, 180, |_| sprint());
    assert!((hspeed(&s) - t.sprint_speed).abs() < 1e-3, "speed {}", hspeed(&s));

    let strafe = Input { move_right: 1.0, sprint: Button::HELD, ..Input::default() };
    let (s, _) = run(&world, standing(&t), &t, 180, |_| strafe);
    assert!((hspeed(&s) - t.walk_speed).abs() < 1e-3, "sideways sprint is a walk");
    assert!(s.position.y < -1.0, "right is -Y at yaw 0");
}

#[test]
fn releasing_input_stops_the_player() {
    let t = tuning();
    let world = flat_world();
    let (moving, _) = run(&world, standing(&t), &t, 60, |_| sprint());
    assert!(hspeed(&moving) > 1.0);
    let (stopped, _) = run(&world, moving, &t, 180, |_| Input::default());
    assert_eq!(hspeed(&stopped), 0.0);
}

#[test]
fn friction_waits_for_the_landing_grace_period() {
    let t = tuning();
    let world = flat_world();
    let mut inside = standing(&t);
    inside.velocity = Vec3::new(5.0, 0.0, 0.0);
    inside.mode_time = t.landing_friction_grace - DT * 0.5;
    let (after, _) = run(&world, inside, &t, 1, |_| Input::default());
    assert_eq!(hspeed(&after), 5.0, "no friction inside the grace period");

    let mut at_limit = inside;
    at_limit.mode_time = t.landing_friction_grace;
    let (after, _) = run(&world, at_limit, &t, 1, |_| Input::default());
    assert!(hspeed(&after) < 5.0, "friction applies once the grace period is over");
    assert!(after.position.x > 0.0);
}

#[test]
fn crouching_slows_to_crouch_speed() {
    let t = tuning();
    let crouch = Input { crouch: Button::HELD, ..forward() };
    let (s, _) = run(&flat_world(), standing(&t), &t, 180, |_| crouch);
    assert!(s.crouched);
    assert!((hspeed(&s) - t.crouch_speed).abs() < 1e-3);
}

#[test]
fn the_player_stands_up_only_with_headroom() {
    let t = tuning();
    let mut crouched = standing(&t);
    crouched.crouched = true;

    let (open, _) = run(&flat_world(), crouched, &t, 5, |_| Input::default());
    assert!(!open.crouched, "stands up under open sky");

    let mut low = flat_world();
    let ceiling = t.crouch_height + 0.2;
    low.add_box(Vec3::new(0.0, 0.0, ceiling + 0.5), Vec3::new(50.0, 50.0, 0.5));
    let (blocked, _) = run(&low, crouched, &t, 5, |_| Input::default());
    assert!(blocked.crouched, "stays crouched under a low ceiling");
    assert_eq!(blocked.mode, Mode::Grounded);
}

#[test]
fn jumping_leaves_the_ground_with_jump_speed() {
    let t = tuning();
    let jump = Input { jump: Button::PRESSED, ..forward() };
    let (s, events) = run(&flat_world(), standing(&t), &t, 1, |_| jump);
    assert_eq!(s.mode, Mode::Airborne);
    assert_eq!(s.velocity.z, t.jump_speed);
    assert_eq!(s.jumps_used, 1);
    assert_eq!(s.mode_time, 0.0, "mode time resets on a transition");
    assert_eq!(events, vec![Event::Jumped]);
    assert!(s.position.z > t.collision_skin, "the jump tick already moves upward");
}

#[test]
fn holding_jump_without_a_fresh_press_does_nothing() {
    let t = tuning();
    let held = Input { jump: Button::HELD, ..forward() };
    let (s, events) = run(&flat_world(), standing(&t), &t, 5, |_| held);
    assert_eq!(s.mode, Mode::Grounded);
    assert!(events.is_empty());
    assert!(s.position.x > 0.0, "the steps still ran");
}

#[test]
fn walking_off_a_ledge_becomes_airborne_with_one_jump_spent() {
    let t = tuning();
    let mut world = BoxWorld::new();
    // Platform: x from -5 to 2, top at z = 0.
    world.add_box(Vec3::new(-1.5, 0.0, -0.5), Vec3::new(3.5, 5.0, 0.5));
    let (s, _, ticks) =
        run_until(&world, standing(&t), &t, 600, |_| forward(), |s| s.mode != Mode::Grounded);
    assert!(ticks < 600, "never left the platform");
    assert_eq!(s.mode, Mode::Airborne);
    assert_eq!(s.jumps_used, 1);
    assert!(s.position.x > 2.0);
}

/// Floor plus a block from x = 2 to x = 10 whose top is at `height`.
fn world_with_step(height: f32) -> BoxWorld {
    let mut world = flat_world();
    world.add_box(Vec3::new(6.0, 0.0, height * 0.5), Vec3::new(4.0, 5.0, height * 0.5));
    world
}

#[test]
fn a_step_just_under_step_height_is_climbed() {
    let t = tuning();
    let height = t.step_height - 0.05;
    // 120 ticks: far enough to be on the block, not so far as to walk off its end.
    let (s, _) = run(&world_with_step(height), standing(&t), &t, 120, |_| forward());
    assert_eq!(s.mode, Mode::Grounded);
    assert!(s.position.x > 3.0 && s.position.x < 10.0, "x {}", s.position.x);
    assert!((s.position.z - height).abs() < 0.02, "z {}", s.position.z);
    assert!((hspeed(&s) - t.walk_speed).abs() < 1e-2, "climbing keeps walk speed");
}

#[test]
fn a_step_just_over_step_height_blocks() {
    let t = tuning();
    let height = t.step_height + 0.05;
    let (s, _) = run(&world_with_step(height), standing(&t), &t, 240, |_| forward());
    assert_eq!(s.mode, Mode::Grounded);
    assert!(s.position.x < 2.0, "x {}", s.position.x);
    assert!(s.position.x > 1.0, "it should have reached the step");
    assert!(s.position.z < 0.05, "z {}", s.position.z);
}

#[test]
fn step_bookkeeping_advances_timers() {
    let t = tuning();
    let mut start = standing(&t);
    start.slide_boost_cooldown = DT * 1.5;
    let (one, _) = run(&flat_world(), start, &t, 1, |_| Input::default());
    assert!((one.mode_time - DT).abs() < 1e-7);
    assert!((one.time_since_wall - DT).abs() < 1e-7);
    assert!((one.slide_boost_cooldown - DT * 0.5).abs() < 1e-6);
    let (two, _) = run(&flat_world(), one, &t, 1, |_| Input::default());
    assert_eq!(two.slide_boost_cooldown, 0.0, "the cooldown stops at zero");
}

#[test]
fn walking_down_a_ramp_stays_on_its_surface() {
    let t = tuning();
    // A 15 degree ramp descending toward +X. Its surface passes through z = 0.5176 at x = 0.
    let angle = 15f32.to_radians();
    let mut world = BoxWorld::new();
    world.add_rotated_box(Vec3::ZERO, Vec3::new(60.0, 60.0, 0.5), glam::Quat::from_rotation_y(angle));
    let surface = |x: f32| (0.5 - angle.sin() * x) / angle.cos();
    // A capsule's lowest point sits slightly above a slope it rests on.
    let rest_height = t.capsule_radius * (1.0 / angle.cos() - 1.0) + t.collision_skin;

    let start = pilot_core::PilotState::spawn(Vec3::new(0.0, 0.0, surface(0.0) + 0.03), &t);
    let (s, _) = run(&world, start, &t, 60, |_| forward());
    assert_eq!(s.mode, Mode::Grounded);
    assert!(s.position.x > 3.0, "x {}", s.position.x);
    let expected = surface(s.position.x) + rest_height;
    assert!((s.position.z - expected).abs() < 0.01, "z {} expected {expected}", s.position.z);
    assert!((hspeed(&s) - t.walk_speed).abs() < 1e-2, "downhill keeps walk speed");
}

#[test]
fn a_sprint_lasts_until_forward_is_released() {
    let t = tuning();
    let world = flat_world();
    // Tap sprint for five ticks, then keep holding forward only.
    let (s, _) = run(&world, standing(&t), &t, 180, |tick| if tick < 5 { sprint() } else { forward() });
    assert!(s.sprinting);
    assert!((hspeed(&s) - t.sprint_speed).abs() < 1e-3, "still sprinting: {}", hspeed(&s));

    let (stopped, _) = run(&world, s, &t, 1, |_| Input::default());
    assert!(!stopped.sprinting, "releasing forward ends the sprint");

    let (walking, _) = run(&world, stopped, &t, 180, |_| forward());
    assert!(!walking.sprinting);
    assert!((hspeed(&walking) - t.walk_speed).abs() < 1e-3, "forward alone is a walk: {}", hspeed(&walking));
}

#[test]
fn turning_while_sprinting_never_exceeds_sprint_speed() {
    let t = tuning();
    let world = flat_world();
    let (mut s, _) = run(&world, standing(&t), &t, 120, |_| sprint());
    let start_direction = s.velocity.normalize();
    let mut fastest = 0.0_f32;
    // Sweep the view through a quarter turn over half a second.
    for tick in 0..60 {
        let turning = Input { yaw: tick as f32 * (std::f32::consts::FRAC_PI_2 / 30.0).min(0.06), ..sprint() };
        s = run(&world, s, &t, 1, |_| turning).0;
        fastest = fastest.max(hspeed(&s));
    }
    assert!(fastest <= t.sprint_speed + 1e-3, "peaked at {fastest}");
    assert!((hspeed(&s) - t.sprint_speed).abs() < 1e-2, "still at sprint speed: {}", hspeed(&s));
    assert!(s.velocity.normalize().dot(start_direction) < 0.5, "the run did turn");
}

#[test]
fn accelerating_gains_the_push_less_friction_each_step() {
    let t = tuning();
    let world = flat_world();
    let mut start = standing(&t);
    start.mode_time = 1.0;
    let (one, _) = run(&world, start, &t, 1, |_| forward());
    let (two, _) = run(&world, start, &t, 2, |_| forward());
    assert!((hspeed(&one) - t.ground_accel * DT).abs() < 1e-4, "first step {}", hspeed(&one));
    let gain = (t.ground_accel - t.ground_friction) * DT;
    assert!((hspeed(&two) - hspeed(&one) - gain).abs() < 1e-4, "second step gained {}", hspeed(&two) - hspeed(&one));
}

#[test]
fn speed_above_the_target_is_worn_down_by_friction_even_while_pushing() {
    let t = tuning();
    let mut fast = standing(&t);
    fast.velocity = Vec3::new(t.walk_speed + 3.0, 0.0, 0.0);
    fast.mode_time = 1.0;
    let (s, _) = run(&flat_world(), fast, &t, 1, |_| forward());
    let expected = t.walk_speed + 3.0 - t.ground_friction * DT;
    assert!((hspeed(&s) - expected).abs() < 1e-4, "speed {}", hspeed(&s));
}
