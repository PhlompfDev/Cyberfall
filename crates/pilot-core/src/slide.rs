use crate::body::{air_move, capsule, centre};
use crate::collide::{ground_at, is_walkable};
use crate::grounded::ground_move;
use crate::kinematics::{air_accelerate, horizontal};
use crate::state::PilotState;
use crate::transitions::Trigger;
use crate::tuning::Tuning;
use crate::types::{Event, Input, UP};
use crate::world::World;

/// Start a slide: crouch, and add the boost if it is off cooldown.
pub fn begin(s: &mut PilotState, t: &Tuning, ev: &mut Vec<Event>) {
    s.crouched = true;
    if s.slide_boost_cooldown <= 0.0 {
        let flat = horizontal(s.velocity);
        let speed = flat.length();
        let boosted = (speed + t.slide_boost_speed).min(t.slide_boost_max_speed).max(speed);
        s.velocity += flat.normalize_or_zero() * (boosted - speed);
    }
    // Restarts on every slide so chained slides don't boost twice.
    s.slide_boost_cooldown = t.slide_boost_cooldown;
    ev.push(Event::SlideStarted);
}

pub fn step(
    s: &mut PilotState,
    input: &Input,
    t: &Tuning,
    world: &dyn World,
    dt: f32,
    ev: &mut Vec<Event>,
) -> Option<Trigger> {
    s.crouched = true;

    if input.jump.pressed {
        // Slide-hop: leave the ground with the slide's full speed.
        s.velocity.z = t.jump_speed;
        s.jumps_used = 1;
        ev.push(Event::Jumped);
        ev.push(Event::SlideEnded);
        air_move(s, t, world, dt);
        return Some(Trigger::Jump);
    }

    let flat = horizontal(s.velocity);
    let speed = flat.length();
    let stopping = !input.crouch.held || speed < t.slide_exit_speed;
    if speed < t.sprint_speed {
        s.sprinting = false;
    }

    let decel = t.slide_decel + t.slide_decay_rate * speed;
    let mut velocity = flat.normalize_or_zero() * (speed - decel * dt).max(0.0);
    let reach = t.step_height + t.ground_probe_distance;
    if let Some(g) = ground_at(world, capsule(s, t), centre(s, t), reach, t) {
        if is_walkable(g.normal, t) {
            // Gravity along the slope speeds a downhill slide and slows an uphill one.
            let pull = -UP * t.gravity;
            velocity += horizontal(pull - g.normal * pull.dot(g.normal)) * dt;
        }
    }
    velocity = air_accelerate(velocity, input.wish_dir(), t.air_wish_cap, t.slide_steer_accel, dt);
    s.velocity = velocity;

    if !ground_move(s, t, world, dt) {
        s.jumps_used = 1;
        ev.push(Event::SlideEnded);
        return Some(Trigger::LostGround);
    }
    if stopping {
        ev.push(Event::SlideEnded);
        return Some(Trigger::SlideStop);
    }
    None
}
