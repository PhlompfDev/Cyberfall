use crate::body::{air_move, capsule, centre, set_centre, update_crouch};
use crate::collide::{free_distance, ground_at, is_walkable, move_and_slide};
use crate::kinematics::{apply_friction, horizontal, push};
use crate::slide;
use crate::state::PilotState;
use crate::transitions::Trigger;
use crate::tuning::Tuning;
use crate::types::{Event, Input, UP};
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

    if input.jump.pressed {
        s.velocity.z = t.jump_speed;
        s.jumps_used = 1;
        ev.push(Event::Jumped);
        air_move(s, t, world, dt);
        return Some(Trigger::Jump);
    }

    if input.crouch.pressed && horizontal(s.velocity).length() >= t.slide_entry_speed {
        slide::begin(s, t, ev);
        if !ground_move(s, t, world, dt) {
            s.jumps_used = 1;
            ev.push(Event::SlideEnded);
            return Some(Trigger::LostGround);
        }
        return Some(Trigger::SlideStart);
    }

    let wish_speed = if s.crouched {
        t.crouch_speed
    } else if s.sprinting {
        t.sprint_speed
    } else {
        t.walk_speed
    };
    if s.mode_time >= t.landing_friction_grace {
        s.velocity = apply_friction(s.velocity, t.ground_friction, dt);
    }
    let limit = horizontal(s.velocity).length().max(wish_speed);
    s.velocity = push(s.velocity, input.wish_dir(), t.ground_accel, limit, dt);

    if !ground_move(s, t, world, dt) {
        s.jumps_used = 1;
        return Some(Trigger::LostGround);
    }
    None
}

/// Move along the ground, climbing steps. False when there's no ground left underneath.
pub fn ground_move(s: &mut PilotState, t: &Tuning, world: &dyn World, dt: f32) -> bool {
    s.velocity.z = 0.0;
    let cap = capsule(s, t);
    let start = centre(s, t);
    let feet = s.position.z;

    let direct = move_and_slide(world, cap, start, s.velocity, dt, t);
    let mut end = direct.centre;
    let mut velocity = direct.velocity;

    if !direct.hits.is_empty() {
        // Blocked: try the same move from one step higher, then drop back down.
        let lift = free_distance(world, cap, start, UP, t.step_height, t);
        let raised = move_and_slide(world, cap, start + UP * lift, s.velocity, dt, t);
        if let Some(g) = ground_at(world, cap, raised.centre, lift + t.ground_probe_distance, t) {
            let landed = raised.centre - UP * (g.distance - t.collision_skin).max(0.0);
            let rise = g.point.z - feet;
            let direct_progress = horizontal(direct.centre - start).length();
            let stepped_progress = horizontal(landed - start).length();
            if is_walkable(g.normal, t)
                && rise <= t.step_height + t.collision_skin
                && stepped_progress > direct_progress + t.collision_skin
            {
                end = landed;
                velocity = raised.velocity;
            }
        }
    }

    velocity.z = 0.0;
    s.velocity = velocity;

    let grounded = match ground_at(world, cap, end, t.step_height + t.ground_probe_distance, t) {
        Some(g) if is_walkable(g.normal, t) => {
            end -= UP * (g.distance - t.collision_skin).max(0.0);
            true
        }
        _ => false,
    };
    set_centre(s, t, end);
    grounded
}
