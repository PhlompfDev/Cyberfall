mod common;

use common::*;
use glam::Vec3;
use pilot_core::{Event, Input, Mode, PilotState, Tuning};
use world_parry::BoxWorld;

/// Floor plus a wall whose face is the plane y = -2, facing +Y.
/// Facing +X, the wall is on the player's right.
fn wall_world() -> BoxWorld {
    let mut world = flat_world();
    world.add_box(Vec3::new(0.0, -2.5, 10.0), Vec3::new(100.0, 0.5, 10.0));
    world
}

/// Airborne next to the wall, drifting into it, so it touches on the first tick.
fn into_wall(t: &Tuning, along: f32, height: f32) -> PilotState {
    let y = -2.0 + t.capsule_radius + 0.01;
    airborne(Vec3::new(0.0, y, height), Vec3::new(along, -1.0, 0.0), t)
}

/// Already wallrunning along +X, high enough that a full-length run does not
/// reach the floor.
fn running(t: &Tuning, along: f32) -> PilotState {
    let y = -2.0 + t.capsule_radius + t.collision_skin;
    let mut s = airborne(Vec3::new(0.0, y, 12.0), Vec3::new(along, 0.0, 0.0), t);
    s.mode = Mode::Wallrunning;
    s.wall_normal = Vec3::Y;
    s
}

fn one(world: &BoxWorld, s: PilotState, t: &Tuning, input: Input) -> (PilotState, Vec<Event>) {
    run(world, s, t, 1, |_| input)
}

#[test]
fn touching_a_wall_at_speed_starts_a_wallrun() {
    let t = tuning();
    let (s, events) = one(&wall_world(), into_wall(&t, t.sprint_speed, 3.0), &t, Input::default());
    assert_eq!(s.mode, Mode::Wallrunning);
    assert_eq!(events, vec![Event::WallrunStarted]);
    assert!((s.wall_normal - Vec3::Y).length() < 1e-3);
    assert!(s.velocity.y.abs() < 1e-3, "motion into the wall is removed");

    assert!((s.velocity.x - t.sprint_speed).abs() < 1e-3, "speed along the wall is kept");
    assert_eq!(s.jumps_used, 1, "the double jump is available again");
    assert_eq!(s.mode_time, 0.0);
}

#[test]
fn a_fast_fall_is_lifted_on_entry() {
    let t = tuning();
    let mut start = into_wall(&t, t.sprint_speed, 8.0);
    start.velocity.z = -6.0;
    let (s, _) = one(&wall_world(), start, &t, Input::default());
    assert_eq!(s.mode, Mode::Wallrunning);
    let falling = -6.0 - t.gravity * DT;
    assert!((s.velocity.z - (falling + t.wallrun_entry_up_speed)).abs() < 1e-4, "vz {}", s.velocity.z);
}

#[test]
fn a_wallrun_needs_minimum_speed() {
    let t = tuning();
    let world = wall_world();
    let (fast, _) = one(&world, into_wall(&t, t.wallrun_min_speed + 0.02, 3.0), &t, Input::default());
    assert_eq!(fast.mode, Mode::Wallrunning);

    let (slow, events) = one(&world, into_wall(&t, t.wallrun_min_speed - 0.02, 3.0), &t, Input::default());
    assert_eq!(slow.mode, Mode::Airborne);
    assert!(events.is_empty());
    assert!(slow.velocity.y.abs() < 1e-3, "it did reach the wall");
}

#[test]
fn a_wallrun_needs_clearance_above_the_ground() {
    let t = tuning();
    let world = wall_world();
    let high = t.wallrun_min_ground_clearance + 0.1;
    let (s, _) = one(&world, into_wall(&t, t.sprint_speed, high), &t, Input::default());
    assert_eq!(s.mode, Mode::Wallrunning);

    let low = t.wallrun_min_ground_clearance - 0.1;
    let (s, _) = one(&world, into_wall(&t, t.sprint_speed, low), &t, Input::default());
    assert_eq!(s.mode, Mode::Airborne);
    assert!(s.velocity.y.abs() < 1e-3, "it did reach the wall");
}

#[test]
fn the_same_wall_cannot_be_reused_straight_away() {
    let t = tuning();
    let world = wall_world();
    let mut just_left = into_wall(&t, t.sprint_speed, 3.0);
    just_left.last_wall_normal = Vec3::Y;
    just_left.time_since_wall = t.wallrun_same_wall_delay * 0.5;
    just_left.grounded_since_wall = false;

    let (s, events) = one(&world, just_left, &t, Input::default());
    assert_eq!(s.mode, Mode::Airborne, "same wall, too soon");
    assert!(events.is_empty());
    assert!(s.velocity.y.abs() < 1e-3, "it did reach the wall");

    let mut waited = just_left;
    waited.time_since_wall = t.wallrun_same_wall_delay + DT;
    assert_eq!(one(&world, waited, &t, Input::default()).0.mode, Mode::Wallrunning);

    let mut touched_ground = just_left;
    touched_ground.grounded_since_wall = true;
    assert_eq!(one(&world, touched_ground, &t, Input::default()).0.mode, Mode::Wallrunning);

    let mut other_wall = just_left;
    other_wall.last_wall_normal = -Vec3::Y;
    assert_eq!(one(&world, other_wall, &t, Input::default()).0.mode, Mode::Wallrunning);
}

#[test]
fn speed_along_the_wall_climbs_toward_wallrun_speed() {
    let t = tuning();
    let world = wall_world();
    let (s, _) = one(&world, running(&t, t.sprint_speed), &t, Input::default());
    let expected = t.wallrun_speed - (t.wallrun_speed - t.sprint_speed) * (-t.wallrun_accel_rate * DT).exp();
    assert!((s.velocity.x - expected).abs() < 1e-4, "speed {}", s.velocity.x);

    let ticks = ((t.wallrun_time_limit / DT) as usize).saturating_sub(2);
    let (s, _) = run(&world, running(&t, t.sprint_speed), &t, ticks, |_| Input::default());
    assert_eq!(s.mode, Mode::Wallrunning);
    assert!(s.velocity.x <= t.wallrun_speed + 1e-4);
    assert!(s.velocity.x > t.sprint_speed);
}

#[test]
fn a_faster_runner_is_not_slowed_by_the_wall() {
    let t = tuning();
    let fast = t.wallrun_speed + 2.0;
    let (s, _) = one(&wall_world(), running(&t, fast), &t, Input::default());
    assert_eq!(s.mode, Mode::Wallrunning);
    assert!((s.velocity.x - fast).abs() < 1e-4);
}

#[test]
fn the_downward_pull_grows_over_the_run() {
    let t = tuning();
    let world = wall_world();
    let (early, _) = one(&world, running(&t, t.sprint_speed), &t, Input::default());
    let early_expected = t.wallrun_vertical_damping * t.wallrun_lift * DT;
    assert!((early.velocity.z - early_expected).abs() < 1e-4, "a small lift at the start: {}", early.velocity.z);

    let mut late = running(&t, t.sprint_speed);
    late.mode_time = t.wallrun_time_limit * 0.9;
    let (late, _) = one(&world, late, &t, Input::default());
    let target = t.wallrun_lift - t.wallrun_sink_ramp * t.wallrun_time_limit * 0.9;
    let late_expected = t.wallrun_vertical_damping * target * DT;
    assert!((late.velocity.z - late_expected).abs() < 1e-4, "vz {}", late.velocity.z);
    assert!(late.velocity.z < early.velocity.z, "later means heavier");
    assert_eq!(late.mode, Mode::Wallrunning);
}

#[test]
fn the_time_limit_ends_the_run() {
    let t = tuning();
    let world = wall_world();
    let mut nearly = running(&t, t.sprint_speed);
    nearly.mode_time = t.wallrun_time_limit - DT;
    assert_eq!(one(&world, nearly, &t, Input::default()).0.mode, Mode::Wallrunning);

    let mut expired = running(&t, t.sprint_speed);
    expired.mode_time = t.wallrun_time_limit;
    let (s, events) = one(&world, expired, &t, Input::default());
    assert_eq!(s.mode, Mode::Airborne);
    assert_eq!(events, vec![Event::WallrunEnded]);
    assert_eq!(s.last_wall_normal, Vec3::Y);
    assert_eq!(s.time_since_wall, 0.0);
    assert!(!s.grounded_since_wall);
    assert!(s.velocity.z < 0.0, "full gravity applies on the way out");
}

#[test]
fn a_wall_kick_launches_away_and_up() {
    let t = tuning();
    let world = wall_world();
    let jump = Input { jump: Button::PRESSED, ..Input::default() };
    let (s, events) = one(&world, running(&t, t.sprint_speed), &t, jump);
    assert_eq!(s.mode, Mode::Airborne);
    assert_eq!(events, vec![Event::WallKicked, Event::WallrunEnded]);
    assert!((s.velocity.y - t.wall_kick_out_speed).abs() < 1e-3);
    assert_eq!(s.velocity.z, t.wall_kick_up_speed);
    // Facing along the wall, the kick adds its look-direction speed to the run.
    assert!((s.velocity.x - (t.sprint_speed + t.wall_kick_look_speed)).abs() < 1e-3, "speed along the wall is kept and boosted");
    assert_eq!(s.jumps_used, 1);

    let (s, events) = one(&world, s, &t, jump);
    assert_eq!(events, vec![Event::DoubleJumped], "the double jump follows a wall-kick");
    assert_eq!(s.jumps_used, 2);
}

#[test]
fn steering_away_from_the_wall_detaches() {
    let t = tuning();
    let world = wall_world();
    let away = Input { move_right: -1.0, ..Input::default() };
    let (s, events) = one(&world, running(&t, t.sprint_speed), &t, away);
    assert_eq!(s.mode, Mode::Airborne);
    assert_eq!(events, vec![Event::WallrunEnded]);

    let toward = Input { move_right: 1.0, ..Input::default() };
    let (s, events) = one(&world, running(&t, t.sprint_speed), &t, toward);
    assert_eq!(s.mode, Mode::Wallrunning);
    assert!(events.is_empty());
}

#[test]
fn pressing_crouch_drops_off_the_wall() {
    let t = tuning();
    let crouch = Input { crouch: Button::PRESSED, ..Input::default() };
    let (s, events) = one(&wall_world(), running(&t, t.sprint_speed), &t, crouch);
    assert_eq!(s.mode, Mode::Airborne);
    assert_eq!(events, vec![Event::WallrunEnded]);
}

#[test]
fn stalling_ends_the_run() {
    let t = tuning();
    let world = wall_world();
    let (s, events) = one(&world, running(&t, t.wallrun_stall_speed - 0.1), &t, Input::default());
    assert_eq!(s.mode, Mode::Airborne);
    assert_eq!(events, vec![Event::WallrunEnded]);

    let (s, _) = one(&world, running(&t, t.wallrun_stall_speed + 0.1), &t, Input::default());
    assert_eq!(s.mode, Mode::Wallrunning);
}

#[test]
fn running_past_the_end_of_the_wall_ends_the_run() {
    let t = tuning();
    let mut world = flat_world();
    // The wall only spans x from -5 to 5.
    world.add_box(Vec3::new(0.0, -2.5, 10.0), Vec3::new(5.0, 0.5, 10.0));
    let mut start = running(&t, t.sprint_speed);
    start.position.x = 4.0;
    let (s, events, ticks) =
        run_until(&world, start, &t, 60, |_| Input::default(), |s| s.mode != Mode::Wallrunning);
    assert!(ticks < 60 && ticks > 1, "ticks {ticks}");
    assert_eq!(s.mode, Mode::Airborne);
    assert_eq!(events, vec![Event::WallrunEnded]);
    assert!(s.position.x > 5.0);
}

#[test]
fn reaching_the_ground_ends_the_run_with_a_landing() {
    let t = tuning();
    let mut start = running(&t, t.sprint_speed);
    start.position.z = 0.02;
    start.velocity.z = -2.0;
    let (s, events) = one(&wall_world(), start, &t, Input::default());
    assert_eq!(s.mode, Mode::Grounded);
    assert!(matches!(events.as_slice(), [Event::WallrunEnded, Event::Landed { .. }]), "{events:?}");
    assert_eq!(s.jumps_used, 0);
    assert!(s.grounded_since_wall);
}

#[test]
fn a_wall_close_beside_v_is_grabbed_without_touching_it() {
    let t = tuning();
    // Moving parallel to the wall, a gap smaller than the stick distance away.
    let gap = t.wallrun_grab_distance * 0.5;
    let y = -2.0 + t.capsule_radius + gap;
    let start = airborne(Vec3::new(0.0, y, 3.0), Vec3::new(t.sprint_speed, 0.0, -2.0), &t);
    let (s, events) = one(&wall_world(), start, &t, Input::default());
    assert_eq!(s.mode, Mode::Wallrunning);
    assert_eq!(events, vec![Event::WallrunStarted]);

    // Twice the stick distance away is too far.
    let far = airborne(Vec3::new(0.0, -2.0 + t.capsule_radius + t.wallrun_stick_distance * 2.0, 3.0), Vec3::new(t.sprint_speed, 0.0, -2.0), &t);
    let (s, events) = one(&wall_world(), far, &t, Input::default());
    assert_eq!(s.mode, Mode::Airborne);
    assert!(events.is_empty());
}

#[test]
fn between_two_close_walls_the_nearer_one_is_grabbed() {
    let t = tuning();
    let mut world = wall_world();
    // A second wall facing -Y; V's side is a little further from it than from the first.
    let near_gap = t.wallrun_grab_distance * 0.3;
    let far_gap = t.wallrun_grab_distance * 0.8;
    let y = -2.0 + t.capsule_radius + near_gap;
    let far_face = y + t.capsule_radius + far_gap;
    world.add_box(Vec3::new(0.0, far_face + 0.5, 10.0), Vec3::new(100.0, 0.5, 10.0));
    let start = airborne(Vec3::new(0.0, y, 3.0), Vec3::new(t.sprint_speed, 0.0, -2.0), &t);
    let (s, _) = one(&world, start, &t, Input::default());
    assert_eq!(s.mode, Mode::Wallrunning);
    assert!((s.wall_normal - Vec3::Y).length() < 1e-3, "grabbed the wall {:?}", s.wall_normal);
}
