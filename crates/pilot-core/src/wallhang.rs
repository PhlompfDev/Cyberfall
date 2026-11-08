use glam::Vec3;

use crate::body::air_move;
use crate::state::PilotState;
use crate::transitions::Trigger;
use crate::tuning::Tuning;
use crate::types::{Event, Input};
use crate::wallrun;
use crate::world::World;

pub fn begin(s: &mut PilotState, wall_normal: Vec3) {
    s.wall_normal = wall_normal;
    s.velocity = Vec3::ZERO;
}

pub fn step(
    s: &mut PilotState,
    input: &Input,
    t: &Tuning,
    world: &dyn World,
    dt: f32,
    ev: &mut Vec<Event>,
) -> Option<Trigger> {
    s.velocity = Vec3::ZERO;
    if input.jump.pressed {
        wallrun::kick(s, t);
        wallrun::leave(s);
        ev.push(Event::WallKicked);
        air_move(s, t, world, dt);
        return Some(Trigger::WallKick);
    }
    if !input.hang.held {
        wallrun::leave(s);
        return Some(Trigger::HangRelease);
    }
    None
}
