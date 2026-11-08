mod common;

use std::collections::HashSet;

use common::*;
use glam::Vec3;
use pilot_core::{step, Event, Input, Mode, PilotState};
use world_parry::BoxWorld;

/// Floor, a long wall, a mantle block and a low step.
fn course() -> BoxWorld {
    let mut world = flat_world();
    world.add_box(Vec3::new(0.0, -6.5, 10.0), Vec3::new(100.0, 0.5, 10.0));
    world.add_box(Vec3::new(20.0, 0.0, 0.75), Vec3::new(3.0, 4.0, 0.75));
    world.add_box(Vec3::new(8.0, 0.0, 0.15), Vec3::new(1.0, 4.0, 0.15));
    world
}

/// A fixed, busy input sequence that wanders through most of the modes.
fn script(tick: usize) -> Input {
    let phase = tick / 40;
    Input {
        move_forward: if phase % 5 == 4 { -1.0 } else { 1.0 },
        move_right: [0.0, 1.0, 0.0, -1.0][phase % 4],
        // Keeps turning, so the path loops around the course instead of leaving the floor.
        yaw: tick as f32 * 0.02,
        pitch: 0.0,
        jump: Button { held: tick % 37 < 3, pressed: tick.is_multiple_of(37) },
        crouch: Button { held: tick % 53 < 20, pressed: tick.is_multiple_of(53) },
        sprint: Button::HELD,
        hang: Button { held: tick % 211 < 15, pressed: tick.is_multiple_of(211) },
    }
}

fn playthrough() -> (Vec<PilotState>, Vec<Event>) {
    let t = tuning();
    let world = course();
    let mut state = standing(&t);
    let mut states = Vec::new();
    let mut events = Vec::new();
    for tick in 0..3000 {
        let (next, mut new_events) = step(&state, &script(tick), &t, &world, DT);
        state = next;
        states.push(state);
        events.append(&mut new_events);
    }
    (states, events)
}

#[test]
fn the_same_inputs_give_identical_results() {
    let (first_states, first_events) = playthrough();
    let (second_states, second_events) = playthrough();
    assert_eq!(first_states, second_states);
    assert_eq!(first_events, second_events);
    assert_eq!(format!("{:?}", first_states.last()), format!("{:?}", second_states.last()));
}

#[test]
fn the_state_stays_finite_and_above_the_floor() {
    let (states, events) = playthrough();
    for (tick, s) in states.iter().enumerate() {
        let numbers = [
            s.position.x, s.position.y, s.position.z,
            s.velocity.x, s.velocity.y, s.velocity.z,
            s.camera.eye_height, s.camera.roll, s.mode_time,
        ];
        assert!(numbers.iter().all(|n| n.is_finite()), "tick {tick}: {s:?}");
        // The feet never dip below the floor plane, not even by the collision skin.
        assert!(s.position.z > 0.0, "tick {tick}: sank into the floor at {:?}", s.position);
        assert!(s.velocity.length() < 100.0, "tick {tick}: runaway velocity {:?}", s.velocity);
    }
    let modes: HashSet<Mode> = states.iter().map(|s| s.mode).collect();
    assert!(modes.contains(&Mode::Grounded) && modes.contains(&Mode::Airborne));
    assert!(modes.len() >= 3, "the script should exercise several modes, saw {modes:?}");
    assert!(events.len() > 20, "the script should produce events");
}

fn sprint_then_jump(dt: f32) -> Vec3 {
    let t = tuning();
    let world = flat_world();
    let mut state = standing(&t);
    let steps = (3.0 / dt).round() as usize;
    let jump_step = (2.0 / dt).round() as usize;
    for i in 0..steps {
        let jump = if i == jump_step { Button::PRESSED } else { Button::UP };
        state = step(&state, &Input { jump, ..sprint() }, &t, &world, dt).0;
    }
    state.position
}

#[test]
fn halving_the_timestep_barely_changes_the_outcome() {
    let coarse = sprint_then_jump(1.0 / 60.0);
    let fine = sprint_then_jump(1.0 / 120.0);
    assert!(coarse.x > 10.0, "it should have travelled, x {}", coarse.x);
    assert!((coarse - fine).length() < 0.02 * coarse.x, "coarse {coarse:?} fine {fine:?}");
}

#[test]
#[ignore = "diagnostic: prints what the scripted playthrough visits"]
fn describe_the_playthrough() {
    let (states, events) = playthrough();
    let mut counts = std::collections::BTreeMap::new();
    for s in &states {
        *counts.entry(format!("{:?}", s.mode)).or_insert(0usize) += 1;
    }
    let reach = states.iter().map(|s| s.position.length()).fold(0.0_f32, f32::max);
    println!("modes {counts:?}\nevents {}\nfarthest from origin {reach:.1} m", events.len());
}
