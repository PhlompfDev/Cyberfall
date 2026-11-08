mod common;

use common::*;
use glam::Vec3;
use pilot_core::{Event, Input, Mode, PilotState, Tuning};
use world_parry::BoxWorld;

fn wall_world() -> BoxWorld {
    let mut world = flat_world();
    world.add_box(Vec3::new(0.0, -2.5, 10.0), Vec3::new(100.0, 0.5, 10.0));
    world
}

fn running(t: &Tuning) -> PilotState {
    let y = -2.0 + t.capsule_radius + t.collision_skin;
    let mut s = airborne(Vec3::new(0.0, y, 3.0), Vec3::new(t.sprint_speed, 0.0, 0.0), t);
    s.mode = Mode::Wallrunning;
    s.wall_normal = Vec3::Y;
    s
}

fn hold_hang() -> Input {
    Input { hang: Button::HELD, ..Input::default() }
}

#[test]
fn holding_hang_during_a_wallrun_freezes_on_the_wall() {
    let t = tuning();
    let world = wall_world();
    let (s, events) = run(&world, running(&t), &t, 1, |_| hold_hang());
    assert_eq!(s.mode, Mode::WallHanging);
    assert_eq!(events, vec![Event::WallrunEnded]);
    assert_eq!(s.velocity, Vec3::ZERO);

    let (later, events) = run(&world, s, &t, 60, |_| hold_hang());
    assert_eq!(later.mode, Mode::WallHanging);
    assert_eq!(later.position, s.position, "a hang does not drift");
    assert!(events.is_empty());
    assert!(later.mode_time > 0.9, "the steps ran");
}

#[test]
fn hang_can_start_straight_from_the_air() {
    let t = tuning();
    let y = -2.0 + t.capsule_radius + 0.01;
    let start = airborne(Vec3::new(0.0, y, 3.0), Vec3::new(0.5, -1.0, 0.0), &t);
    let (s, _) = run(&wall_world(), start, &t, 1, |_| hold_hang());
    assert_eq!(s.mode, Mode::WallHanging);
    assert!((s.wall_normal - Vec3::Y).length() < 1e-3);
    assert_eq!(s.velocity, Vec3::ZERO);
}

#[test]
fn releasing_hang_drops_and_records_the_wall() {
    let t = tuning();
    let world = wall_world();
    let (hanging, _) = run(&world, running(&t), &t, 1, |_| hold_hang());
    let (s, events) = run(&world, hanging, &t, 1, |_| Input::default());
    assert_eq!(s.mode, Mode::Airborne);
    assert!(events.is_empty());
    assert_eq!(s.last_wall_normal, Vec3::Y);
    assert!(!s.grounded_since_wall);
}

#[test]
fn jumping_from_a_hang_is_a_wall_kick() {
    let t = tuning();
    let world = wall_world();
    let (hanging, _) = run(&world, running(&t), &t, 1, |_| hold_hang());
    let kick = Input { jump: Button::PRESSED, hang: Button::HELD, ..Input::default() };
    let (s, events) = run(&world, hanging, &t, 1, |_| kick);
    assert_eq!(s.mode, Mode::Airborne);
    assert_eq!(events, vec![Event::WallKicked]);
    assert!((s.velocity.y - t.wall_kick_out_speed).abs() < 1e-3);
    assert_eq!(s.velocity.z, t.wall_kick_up_speed);
    assert_eq!(s.jumps_used, 1);
    assert!(s.position.y > hanging.position.y, "the kick tick already moves away");
}

/// Floor plus a block whose near face is x = 2 and whose top is at `top`.
fn ledge_world(top: f32) -> BoxWorld {
    let mut world = flat_world();
    world.add_box(Vec3::new(4.0, 0.0, top * 0.5), Vec3::new(2.0, 10.0, top * 0.5));
    world
}

const FEET: f32 = 0.3;

/// Airborne against the block's face, moving into it, feet at `FEET`.
fn at_ledge(t: &Tuning) -> PilotState {
    let x = 2.0 - t.capsule_radius - 0.01;
    airborne(Vec3::new(x, 0.0, FEET), Vec3::new(3.0, 0.0, 1.0), t)
}

fn mid_reach(t: &Tuning) -> f32 {
    FEET + (t.mantle_min_height + t.mantle_max_height) * 0.5
}

#[test]
fn pushing_into_a_reachable_ledge_starts_a_mantle() {
    let t = tuning();
    let (s, events) = run(&ledge_world(mid_reach(&t)), at_ledge(&t), &t, 1, |_| forward());
    assert_eq!(s.mode, Mode::Mantling);
    assert_eq!(events, vec![Event::MantleStarted]);
    assert_eq!(s.velocity, Vec3::ZERO);
    assert!((s.mantle_to.z - (mid_reach(&t) + t.collision_skin)).abs() < 1e-3);
}

#[test]
fn a_mantle_rises_then_moves_onto_the_ledge_and_ends_standing() {
    let t = tuning();
    let top = mid_reach(&t);
    let world = ledge_world(top);
    let (mut s, _) = run(&world, at_ledge(&t), &t, 1, |_| forward());
    let from = s.mantle_from;
    let to = s.mantle_to;

    let mut ticks = 0_usize;
    let mut events = Vec::new();
    while s.mode == Mode::Mantling {
        let (next, mut new_events) = run(&world, s, &t, 1, |_| forward());
        s = next;
        events.append(&mut new_events);
        ticks += 1;
        assert!(ticks < 600, "the mantle never finished");
        if s.mode == Mode::Mantling {
            let still_rising = (s.position.x - from.x).abs() < 1e-4;
            let at_height = (s.position.z - to.z).abs() < 1e-4;
            assert!(still_rising || at_height, "it must go up before it goes forward");
        }
    }

    let expected_ticks = (t.mantle_time / DT).ceil() as usize;
    assert!(ticks.abs_diff(expected_ticks) <= 1, "took {ticks} ticks, expected {expected_ticks}");
    assert_eq!(s.mode, Mode::Grounded);
    assert_eq!(events, vec![Event::MantleEnded]);
    assert!((s.position - to).length() < 1e-4);
    assert!((s.position.x - (2.0 + t.mantle_forward)).abs() < 0.03, "x {}", s.position.x);
    assert!((s.velocity - Vec3::X * t.mantle_exit_speed).length() < 1e-3);
    assert_eq!(s.jumps_used, 0);

    let (rested, _) = run(&world, s, &t, 30, |_| Input::default());
    assert_eq!(rested.mode, Mode::Grounded, "the ledge is real ground");
    assert!((rested.position.z - top).abs() < 0.02);
}

#[test]
fn ledges_outside_the_reach_window_are_not_mantled() {
    let t = tuning();
    let try_top = |top: f32| run(&ledge_world(top), at_ledge(&t), &t, 1, |_| forward()).0;

    assert_eq!(try_top(FEET + t.mantle_max_height - 0.05).mode, Mode::Mantling);
    let too_high = try_top(FEET + t.mantle_max_height + 0.05);
    assert_ne!(too_high.mode, Mode::Mantling);
    assert!(too_high.velocity.x.abs() < 1e-3, "it did reach the wall");

    assert_eq!(try_top(FEET + t.mantle_min_height + 0.05).mode, Mode::Mantling);
    let too_low = try_top(FEET + t.mantle_min_height - 0.05);
    assert_ne!(too_low.mode, Mode::Mantling);
    assert!(too_low.velocity.x.abs() < 1e-3, "it did reach the wall");
}

#[test]
fn a_mantle_needs_forward_input_toward_the_wall() {
    let t = tuning();
    let world = ledge_world(mid_reach(&t));

    let (idle, _) = run(&world, at_ledge(&t), &t, 1, |_| Input::default());
    assert_ne!(idle.mode, Mode::Mantling);
    assert!(idle.velocity.x.abs() < 1e-3, "it did reach the wall");

    let facing_away = Input { yaw: std::f32::consts::PI, ..forward() };
    let (away, _) = run(&world, at_ledge(&t), &t, 1, |_| facing_away);
    assert_ne!(away.mode, Mode::Mantling);
    assert!(away.velocity.x.abs() < 1e-3, "it did reach the wall");
}

#[test]
fn a_ledge_without_standing_room_is_not_mantled() {
    let t = tuning();
    let top = mid_reach(&t);
    let with_ceiling = |gap: f32| {
        let mut world = ledge_world(top);
        world.add_box(Vec3::new(4.0, 0.0, top + gap + 0.5), Vec3::new(2.0, 10.0, 0.5));
        world
    };

    let roomy = with_ceiling(t.stand_height + 0.3);
    assert_eq!(run(&roomy, at_ledge(&t), &t, 1, |_| forward()).0.mode, Mode::Mantling);

    let cramped = with_ceiling(t.stand_height - 0.3);
    let (s, _) = run(&cramped, at_ledge(&t), &t, 1, |_| forward());
    assert_ne!(s.mode, Mode::Mantling);
    assert!(s.velocity.x.abs() < 1e-3, "it did reach the wall");
}

#[test]
fn a_wallrunner_facing_the_wall_mantles_over_its_top() {
    let t = tuning();
    let mut world = flat_world();
    // The same wall as `wall_world`, but only 4 m tall.
    world.add_box(Vec3::new(0.0, -2.5, 2.0), Vec3::new(100.0, 0.5, 2.0));
    let mut start = running(&t);
    start.position.z = 4.0 - (t.mantle_min_height + t.mantle_max_height) * 0.5;
    let face_wall = Input { yaw: -std::f32::consts::FRAC_PI_2, ..forward() };
    let (s, events) = run(&world, start, &t, 1, |_| face_wall);
    assert_eq!(s.mode, Mode::Mantling);
    assert_eq!(events, vec![Event::WallrunEnded, Event::MantleStarted]);
    assert!((s.mantle_to.z - (4.0 + t.collision_skin)).abs() < 1e-3);
    assert!(s.mantle_to.y < -2.0, "the target is on top of the wall");
}

#[test]
fn a_reachable_ledge_wins_over_a_wallrun() {
    let t = tuning();
    // Fast and diagonal: quick enough along the wall to start a wallrun,
    // but the player is facing the ledge and pushing into it.
    let mut start = at_ledge(&t);
    start.position.z = t.wallrun_min_ground_clearance + 0.3;
    start.velocity = Vec3::new(3.0, t.sprint_speed, 1.0);
    let top = start.position.z + (t.mantle_min_height + t.mantle_max_height) * 0.5;
    let world = ledge_world(top);

    let (s, events) = run(&world, start, &t, 1, |_| forward());
    assert_eq!(s.mode, Mode::Mantling);
    assert_eq!(events, vec![Event::MantleStarted]);

    // The same approach without pushing forward is a wallrun, so both were possible.
    let (s, events) = run(&world, start, &t, 1, |_| Input::default());
    assert_eq!(s.mode, Mode::Wallrunning);
    assert_eq!(events, vec![Event::WallrunStarted]);
}
