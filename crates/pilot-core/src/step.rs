use crate::airborne;
use crate::camera;
use crate::grounded;
use crate::mantle;
use crate::slide;
use crate::state::PilotState;
use crate::transitions::next_mode;
use crate::tuning::Tuning;
use crate::wallhang;
use crate::wallrun;
use crate::types::{Event, Input, Mode};
use crate::world::World;

/// Advance the pilot by one fixed timestep. Deterministic and side-effect free.
pub fn step(
    state: &PilotState,
    input: &Input,
    tuning: &Tuning,
    world: &dyn World,
    dt: f32,
) -> (PilotState, Vec<Event>) {
    let mut s = *state;
    let mut events = Vec::new();
    let t = tuning;

    s.slide_boost_cooldown = (s.slide_boost_cooldown - dt).max(0.0);
    s.time_since_wall += dt;
    if input.move_forward <= 0.0 {
        s.sprinting = false;
    } else if input.sprint.held {
        s.sprinting = true;
    }

    let trigger = match s.mode {
        Mode::Grounded => grounded::step(&mut s, input, t, world, dt, &mut events),
        Mode::Airborne => airborne::step(&mut s, input, t, world, dt, &mut events),
        Mode::Sliding => slide::step(&mut s, input, t, world, dt, &mut events),
        Mode::Wallrunning => wallrun::step(&mut s, input, t, world, dt, &mut events),
        Mode::WallHanging => wallhang::step(&mut s, input, t, world, dt, &mut events),
        Mode::Mantling => mantle::step(&mut s, input, t, world, dt, &mut events),
    };

    s.mode_time += dt;
    if let Some(next) = trigger.and_then(|tr| next_mode(s.mode, tr)) {
        s.mode = next;
        s.mode_time = 0.0;
    }
    camera::update(&mut s, input, t, dt);
    (s, events)
}
