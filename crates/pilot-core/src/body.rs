use glam::Vec3;

use crate::collide::move_and_slide;
use crate::state::PilotState;
use crate::tuning::Tuning;
use crate::types::{Input, UP};
use crate::world::{Capsule, Hit, World};

pub fn capsule(s: &PilotState, t: &Tuning) -> Capsule {
    if s.crouched {
        t.crouch_capsule()
    } else {
        t.stand_capsule()
    }
}

pub fn centre(s: &PilotState, t: &Tuning) -> Vec3 {
    s.position + UP * capsule(s, t).centre_offset()
}

pub fn set_centre(s: &mut PilotState, t: &Tuning, centre: Vec3) {
    s.position = centre - UP * capsule(s, t).centre_offset();
}

/// True when a crouched player has room to stand.
pub fn can_stand(s: &PilotState, t: &Tuning, world: &dyn World) -> bool {
    let crouched = t.crouch_capsule();
    let from = s.position + UP * crouched.centre_offset();
    world.sweep(crouched, from, from + UP * (t.stand_height - t.crouch_height)).is_none()
}

/// Crouch while the button is held; stand again when there is room.
pub fn update_crouch(s: &mut PilotState, input: &Input, t: &Tuning, world: &dyn World) {
    if input.crouch.held {
        s.crouched = true;
    } else if s.crouched && can_stand(s, t, world) {
        s.crouched = false;
    }
}

/// Move by the current velocity with no ground handling. Returns what was hit.
pub fn air_move(s: &mut PilotState, t: &Tuning, world: &dyn World, dt: f32) -> Vec<Hit> {
    let moved = move_and_slide(world, capsule(s, t), centre(s, t), s.velocity, dt, t);
    set_centre(s, t, moved.centre);
    s.velocity = moved.velocity;
    moved.hits
}
