use crate::body::{air_move, centre, update_crouch};
use crate::collide::{face_normal, is_wall, is_walkable};
use crate::kinematics::{air_accelerate, horizontal};
use crate::mantle;
use crate::slide;
use crate::state::PilotState;
use crate::transitions::Trigger;
use crate::tuning::Tuning;
use crate::types::{Event, Input};
use crate::wallhang;
use crate::wall::WallReading;
use crate::wallrun;
use crate::world::World;

pub fn step(
    s: &mut PilotState,
    input: &Input,
    t: &Tuning,
    world: &dyn World,
    dt: f32,
    ev: &mut Vec<Event>,
) -> Option<Trigger> {
    update_crouch(s, input, t, world);

    s.velocity.z -= t.gravity * dt;
    if input.jump.pressed && s.jumps_used < 2 {
        double_jump(s, input, t);
        ev.push(Event::DoubleJumped);
    }
    s.velocity = air_accelerate(s.velocity, input.wish_dir(), t.air_wish_cap, t.air_accel, dt);

    let fall_speed = -s.velocity.z;
    let hits = air_move(s, t, world, dt);
    let body = centre(s, t);

    let on_floor = hits.iter().any(|h| {
        h.normal.z > t.wall_max_normal_z
            && is_walkable(face_normal(world, h.point, body, h.normal, t).0, t)
    });
    if fall_speed >= 0.0 && on_floor {
        return Some(land(s, input, t, fall_speed, ev));
    }
    for hit in hits.iter().filter(|h| is_wall(h.normal, t)) {
        let wall_normal = horizontal(hit.normal).normalize_or_zero();
        if let Some(target) = mantle::find_ledge(s, input, t, world, wall_normal) {
            mantle::begin(s, target, ev);
            return Some(Trigger::LedgeGrab);
        }
        if input.hang.held {
            wallhang::begin(s, wall_normal);
            return Some(Trigger::HangStart);
        }
        let touch = WallReading { normal: wall_normal, distance: 0.0 };
        if wallrun::try_start(s, input, t, world, touch, ev) {
            return Some(Trigger::WallContact);
        }
    }
    if let Some(near) = wallrun::nearby_wall(s, t, world) {
        if wallrun::try_start(s, input, t, world, near, ev) {
            return Some(Trigger::WallContact);
        }
    }
    None
}

fn double_jump(s: &mut PilotState, input: &Input, t: &Tuning) {
    s.velocity.z = t.double_jump_speed;
    let flat = horizontal(s.velocity);
    let limit = flat.length().max(t.sprint_speed);
    let boosted = flat + input.wish_dir() * t.double_jump_boost;
    let limited = if boosted.length() > limit { boosted.normalize() * limit } else { boosted };
    s.velocity.x = limited.x;
    s.velocity.y = limited.y;
    s.jumps_used = 2;
}

/// Shared landing bookkeeping. Holding crouch at slide speed lands in a slide.
pub fn land(
    s: &mut PilotState,
    input: &Input,
    t: &Tuning,
    fall_speed: f32,
    ev: &mut Vec<Event>,
) -> Trigger {
    s.velocity.z = 0.0;
    s.jumps_used = 0;
    s.grounded_since_wall = true;
    ev.push(Event::Landed { impact_speed: fall_speed });
    if input.crouch.held && horizontal(s.velocity).length() >= t.slide_entry_speed {
        slide::begin(s, t, ev);
        return Trigger::LandSliding;
    }
    Trigger::Land
}
