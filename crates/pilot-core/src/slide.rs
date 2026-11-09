use crate::body::{air_move, capsule, centre};
use crate::collide::{ground_at, is_walkable};
use crate::grounded::ground_move;
use crate::kinematics::{air_accelerate, horizontal};
use crate::state::PilotState;
use crate::transitions::Trigger;
use crate::tuning::Tuning;
use crate::types::{Event, Input, UP};
use glam::Vec3;
use crate::world::World;

/// Speed after the slide-start boost: never slower, capped at `slide_boost_max_speed`.
pub fn boosted_speed(speed: f32, t: &Tuning) -> f32 {
    (speed + t.slide_boost_speed).min(t.slide_boost_max_speed).max(speed)
}

/// One frame of slide motion on horizontal velocity: Titanfall's speed decay,
/// gravity along a walkable slope, and light steering. Returns horizontal velocity.
pub fn slide_velocity(flat_velocity: Vec3, wish: Vec3, ground_normal: Option<Vec3>, t: &Tuning, dt: f32) -> Vec3 {
    let flat = horizontal(flat_velocity);
    let speed = flat.length();
    let decel = t.slide_decel + t.slide_decay_rate * speed;
    let mut velocity = flat.normalize_or_zero() * (speed - decel * dt).max(0.0);
    if let Some(normal) = ground_normal.filter(|n| is_walkable(*n, t)) {
        // Gravity along the slope speeds a downhill slide and slows an uphill one.
        let pull = -UP * t.gravity;
        velocity += horizontal(pull - normal * pull.dot(normal)) * dt;
    }
    horizontal(air_accelerate(velocity, wish, t.air_wish_cap, t.slide_steer_accel, dt))
}

/// Start a slide: crouch, and add the boost if it is off cooldown.
pub fn begin(s: &mut PilotState, t: &Tuning, ev: &mut Vec<Event>) {
    s.crouched = true;
    if s.slide_boost_cooldown <= 0.0 {
        let flat = horizontal(s.velocity);
        let speed = flat.length();
        s.velocity += flat.normalize_or_zero() * (boosted_speed(speed, t) - speed);
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

    let reach = t.step_height + t.ground_probe_distance;
    let ground = ground_at(world, capsule(s, t), centre(s, t), reach, t).map(|g| g.normal);
    s.velocity = slide_velocity(flat, input.wish_dir(), ground, t, dt);

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
