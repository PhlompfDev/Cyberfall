use glam::Vec3;

use crate::collide::is_walkable;
use crate::kinematics::horizontal;
use crate::state::PilotState;
use crate::transitions::Trigger;
use crate::tuning::Tuning;
use crate::types::{Event, Input, UP};
use crate::world::World;

/// Where the feet end up on top of a ledge, if it can be mantled.
pub fn find_ledge(
    s: &PilotState,
    input: &Input,
    t: &Tuning,
    world: &dyn World,
    wall_normal: Vec3,
) -> Option<Vec3> {
    if input.move_forward <= 0.0 || input.forward().dot(-wall_normal) < t.mantle_min_approach_dot {
        return None;
    }
    let over = s.position - wall_normal * (t.capsule_radius + t.mantle_forward);
    let top = world.raycast(over + UP * t.mantle_max_height, over + UP * t.mantle_min_height)?;
    if !is_walkable(top.normal, t) {
        return None;
    }
    let stand = t.stand_capsule();
    let centre = top.point + UP * (stand.centre_offset() + t.surface_probe_up);
    if world.sweep(stand, centre, centre + UP * t.collision_skin).is_some() {
        return None;
    }
    Some(top.point + UP * t.collision_skin)
}

pub fn begin(s: &mut PilotState, target: Vec3, ev: &mut Vec<Event>) {
    s.mantle_from = s.position;
    s.mantle_to = target;
    s.velocity = Vec3::ZERO;
    s.crouched = false;
    ev.push(Event::MantleStarted);
}

pub fn step(
    s: &mut PilotState,
    _input: &Input,
    t: &Tuning,
    _world: &dyn World,
    dt: f32,
    ev: &mut Vec<Event>,
) -> Option<Trigger> {
    let from = s.mantle_from;
    let to = s.mantle_to;
    let progress = ((s.mode_time + dt) / t.mantle_time).min(1.0);
    // First half: straight up. Second half: forward onto the ledge.
    let rise = (progress * 2.0).min(1.0);
    let advance = (progress * 2.0 - 1.0).max(0.0);
    s.position = Vec3::new(
        from.x + (to.x - from.x) * advance,
        from.y + (to.y - from.y) * advance,
        from.z + (to.z - from.z) * rise,
    );
    s.velocity = Vec3::ZERO;

    if progress >= 1.0 {
        s.position = to;
        s.velocity = horizontal(to - from).normalize_or_zero() * t.mantle_exit_speed;
        s.jumps_used = 0;
        s.grounded_since_wall = true;
        ev.push(Event::MantleEnded);
        return Some(Trigger::MantleDone);
    }
    None
}
