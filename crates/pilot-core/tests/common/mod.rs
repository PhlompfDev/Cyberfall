#![allow(dead_code)]

use std::path::PathBuf;

use glam::Vec3;
pub use pilot_core::Button;
use pilot_core::{step, Event, Input, Mode, PilotState, Tuning};
use world_parry::BoxWorld;

pub const DT: f32 = 1.0 / 60.0;

/// The repo's real tuning. Tests read expected values from it.
pub fn tuning() -> Tuning {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../tuning/pilot.toml");
    pilot_tuning::load(&path).unwrap()
}

/// A floor whose top face is the plane z = 0.
pub fn flat_world() -> BoxWorld {
    let mut world = BoxWorld::new();
    world.add_box(Vec3::new(0.0, 0.0, -0.5), Vec3::new(200.0, 200.0, 0.5));
    world
}

/// Standing on z = 0 ground, a collision skin above it.
pub fn standing(t: &Tuning) -> PilotState {
    PilotState::spawn(Vec3::new(0.0, 0.0, t.collision_skin), t)
}

pub fn airborne(position: Vec3, velocity: Vec3, t: &Tuning) -> PilotState {
    let mut state = PilotState::spawn(position, t);
    state.mode = Mode::Airborne;
    state.velocity = velocity;
    state.jumps_used = 1;
    state
}

pub fn forward() -> Input {
    Input { move_forward: 1.0, ..Input::default() }
}

pub fn sprint() -> Input {
    Input { move_forward: 1.0, sprint: Button::HELD, ..Input::default() }
}

pub fn hspeed(state: &PilotState) -> f32 {
    Vec3::new(state.velocity.x, state.velocity.y, 0.0).length()
}

/// Run `ticks` steps. `input` receives the tick index.
pub fn run(
    world: &BoxWorld,
    state: PilotState,
    t: &Tuning,
    ticks: usize,
    mut input: impl FnMut(usize) -> Input,
) -> (PilotState, Vec<Event>) {
    let mut state = state;
    let mut events = Vec::new();
    for tick in 0..ticks {
        let (next, mut new_events) = step(&state, &input(tick), t, world, DT);
        state = next;
        events.append(&mut new_events);
    }
    (state, events)
}

/// Run until `done` is true or `max_ticks` pass. Returns the tick count used.
pub fn run_until(
    world: &BoxWorld,
    state: PilotState,
    t: &Tuning,
    max_ticks: usize,
    mut input: impl FnMut(usize) -> Input,
    done: impl Fn(&PilotState) -> bool,
) -> (PilotState, Vec<Event>, usize) {
    let mut state = state;
    let mut events = Vec::new();
    for tick in 0..max_ticks {
        if done(&state) {
            return (state, events, tick);
        }
        let (next, mut new_events) = step(&state, &input(tick), t, world, DT);
        state = next;
        events.append(&mut new_events);
    }
    (state, events, max_ticks)
}
