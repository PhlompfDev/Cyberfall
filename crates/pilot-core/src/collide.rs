use glam::Vec3;

use crate::kinematics::{clip, horizontal};
use crate::tuning::Tuning;
use crate::types::UP;
use crate::world::{Capsule, Ground, Hit, World};

pub struct MoveResult {
    pub centre: Vec3,
    pub velocity: Vec3,
    pub hits: Vec<Hit>,
}

/// Move the capsule by `velocity * dt`, sliding along whatever it hits.
pub fn move_and_slide(
    world: &dyn World,
    capsule: Capsule,
    centre: Vec3,
    velocity: Vec3,
    dt: f32,
    t: &Tuning,
) -> MoveResult {
    let original = velocity;
    let mut centre = centre;
    let mut velocity = velocity;
    let mut remaining = dt;
    let mut hits = Vec::new();
    for _ in 0..t.max_bumps {
        let delta = velocity * remaining;
        let distance = delta.length();
        if distance <= 0.0 {
            break;
        }
        let Some(hit) = world.sweep(capsule, centre, centre + delta) else {
            centre += delta;
            break;
        };
        let travel = (hit.fraction * distance - t.collision_skin).max(0.0);
        centre += delta / distance * travel;
        remaining *= 1.0 - hit.fraction;
        velocity = clip(velocity, hit.normal);
        hits.push(hit);
        if velocity.dot(original) < 0.0 {
            // Two surfaces turned the motion back on itself: stop.
            velocity = Vec3::ZERO;
            break;
        }
    }
    MoveResult { centre, velocity, hits }
}

/// How far the capsule can travel along unit `direction`, up to `max`.
pub fn free_distance(
    world: &dyn World,
    capsule: Capsule,
    from: Vec3,
    direction: Vec3,
    max: f32,
    t: &Tuning,
) -> f32 {
    match world.sweep(capsule, from, from + direction * max) {
        Some(hit) => (hit.fraction * max - t.collision_skin).max(0.0),
        None => max,
    }
}

/// The real face under a contact point (a capsule on a step edge reports a tilted normal).
pub fn face_normal(
    world: &dyn World,
    contact: Vec3,
    body_centre: Vec3,
    fallback: Vec3,
    t: &Tuning,
) -> (Vec3, Vec3) {
    let outward = horizontal(contact - body_centre).normalize_or_zero();
    let from = contact + outward * t.surface_probe_out + UP * t.surface_probe_up;
    let to = from - UP * (t.surface_probe_up * 2.0);
    match world.raycast(from, to) {
        Some(face) => (face.normal, face.point),
        None => (fallback, contact),
    }
}

/// Ground under the capsule within `max_drop`.
pub fn ground_at(
    world: &dyn World,
    capsule: Capsule,
    centre: Vec3,
    max_drop: f32,
    t: &Tuning,
) -> Option<Ground> {
    let contact = world.ground(capsule, centre, max_drop)?;
    let resting = centre - UP * contact.distance;
    let (normal, point) = face_normal(world, contact.point, resting, contact.normal, t);
    Some(Ground { normal, distance: contact.distance, point })
}

pub fn is_walkable(normal: Vec3, t: &Tuning) -> bool {
    normal.z >= t.max_slope_cos()
}

pub fn is_wall(normal: Vec3, t: &Tuning) -> bool {
    normal.z.abs() < t.wall_max_normal_z
}
