//! Feeds a recording's inputs to pilot-core.

use pilot_core::{step, Button, Input, Mode, PilotState, Tuning, World};

use crate::recording::{CoarseMode, Recording, Tick};

/// One simulated step, beside the recorded row it lines up with.
#[derive(Clone, Copy, Debug)]
pub struct SimTick {
    pub time: f32,
    pub state: PilotState,
    pub recorded: Tick,
}

pub fn coarse(mode: Mode) -> CoarseMode {
    match mode {
        Mode::Grounded | Mode::Sliding => CoarseMode::Ground,
        Mode::Airborne | Mode::Mantling => CoarseMode::Air,
        Mode::Wallrunning => CoarseMode::Wallrun,
        Mode::WallHanging => CoarseMode::Hang,
    }
}

fn merge(earlier: Button, later: Button) -> Button {
    Button { held: later.held, pressed: earlier.pressed || later.pressed }
}

fn held_only(button: Button) -> Button {
    Button { held: button.held, pressed: false }
}

/// Step at a fixed `dt` with the inputs recorded during each step.
pub fn simulate(recording: &Recording, world: &dyn World, t: &Tuning, dt: f32) -> Vec<SimTick> {
    let Some(first) = recording.ticks.first() else {
        return Vec::new();
    };
    let mut state = PilotState::spawn(first.position, t);
    state.velocity = first.velocity;
    state.mode = match first.mode {
        CoarseMode::Ground => Mode::Grounded,
        _ => Mode::Airborne,
    };
    // Recordings start at rest or mid-motion, never just after a landing.
    state.mode_time = t.landing_friction_grace;

    let mut out = Vec::new();
    let mut time = first.time;
    let mut next_row = 1;
    let mut current = *first;
    let end = recording.ticks[recording.ticks.len() - 1].time;
    while time + dt <= end {
        // A press belongs to the step it happened in, not to later ones.
        let mut input = Input {
            jump: held_only(current.input.jump),
            crouch: held_only(current.input.crouch),
            sprint: held_only(current.input.sprint),
            ..current.input
        };
        while next_row < recording.ticks.len() && recording.ticks[next_row].time <= time + dt {
            current = recording.ticks[next_row];
            input = Input {
                jump: merge(input.jump, current.input.jump),
                crouch: merge(input.crouch, current.input.crouch),
                sprint: merge(input.sprint, current.input.sprint),
                ..current.input
            };
            next_row += 1;
        }
        state = step(&state, &input, t, world, dt).0;
        time += dt;
        out.push(SimTick { time, state, recorded: current });
    }
    out
}
