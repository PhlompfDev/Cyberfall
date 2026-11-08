//! Wallrunning mode: drives `wall::wall_step` against a `World`, plus landing,
//! mantling and wall-hang.

use glam::Vec3;

use crate::airborne;
use crate::body::{air_move, capsule, centre};
use crate::collide::{face_normal, is_walkable};
use crate::kinematics::horizontal;
use crate::mantle;
use crate::state::PilotState;
use crate::transitions::Trigger;
use crate::tuning::Tuning;
use crate::types::{Event, Input, UP};
use crate::wall::{self, WallEnd, WallInput, WallReading, WallState};
use crate::wallhang;
use crate::world::World;

fn wall_state(s: &PilotState, running: bool) -> WallState {
    WallState {
        running,
        normal: s.wall_normal,
        elapsed: s.mode_time,
        last_normal: s.last_wall_normal,
        time_since_wall: s.time_since_wall,
        grounded_since_wall: s.grounded_since_wall,
    }
}

fn wall_input(input: &Input) -> WallInput {
    WallInput {
        move_forward: input.move_forward,
        move_right: input.move_right,
        yaw: input.yaw,
        jump_pressed: input.jump.pressed,
        crouch_pressed: input.crouch.pressed,
    }
}

fn apply(s: &mut PilotState, w: &WallState) {
    s.wall_normal = w.normal;
    s.last_wall_normal = w.last_normal;
    s.time_since_wall = w.time_since_wall;
    s.grounded_since_wall = w.grounded_since_wall;
}

/// Nearest wall within stick distance to either side of the player's motion.
pub fn nearby_wall(s: &PilotState, t: &Tuning, world: &dyn World) -> Option<WallReading> {
    let heading = horizontal(s.velocity).normalize_or_zero();
    if heading == Vec3::ZERO {
        return None;
    }
    let side = UP.cross(heading);
    let body = centre(s, t);
    let radius = capsule(s, t).radius;
    let reach = radius + t.wallrun_stick_distance;
    [side, -side]
        .into_iter()
        .filter_map(|dir| world.raycast(body, body + dir * reach))
        .map(|hit| WallReading { normal: hit.normal, distance: (hit.fraction * reach - radius).max(0.0) })
        .min_by(|a, b| a.distance.total_cmp(&b.distance))
}

pub fn try_start(
    s: &mut PilotState,
    input: &Input,
    t: &Tuning,
    world: &dyn World,
    reading: WallReading,
    ev: &mut Vec<Event>,
) -> bool {
    let ground_clear =
        world.raycast(s.position, s.position - UP * t.wallrun_min_ground_clearance).is_none();
    let out = wall::wall_step(&wall_state(s, false), s.velocity, Some(reading), ground_clear, &wall_input(input), t, 0.0);
    if !out.state.running {
        return false;
    }
    s.velocity = out.velocity;
    apply(s, &out.state);
    s.jumps_used = 1;
    ev.push(Event::WallrunStarted);
    true
}

pub fn leave(s: &mut PilotState) {
    s.last_wall_normal = s.wall_normal;
    s.time_since_wall = 0.0;
    s.grounded_since_wall = false;
}

/// Jump off the current wall. Leaves the double jump available.
pub fn kick(s: &mut PilotState, t: &Tuning) {
    s.velocity = wall::kick_velocity(s.velocity, s.wall_normal, Vec3::ZERO, t);
    s.jumps_used = 1;
}

pub fn step(
    s: &mut PilotState,
    input: &Input,
    t: &Tuning,
    world: &dyn World,
    dt: f32,
    ev: &mut Vec<Event>,
) -> Option<Trigger> {
    if !input.jump.pressed && input.hang.held {
        let wall_normal = s.wall_normal;
        wallhang::begin(s, wall_normal);
        ev.push(Event::WallrunEnded);
        return Some(Trigger::HangStart);
    }

    let body = centre(s, t);
    let radius = capsule(s, t).radius;
    let reach = radius + t.wallrun_stick_distance;
    let reading = world.raycast(body, body - s.wall_normal * reach).map(|hit| WallReading {
        normal: hit.normal,
        distance: (hit.fraction * reach - radius).max(0.0),
    });
    let out = wall::wall_step(&wall_state(s, true), s.velocity, reading, true, &wall_input(input), t, dt);
    s.velocity = out.velocity;
    apply(s, &out.state);
    let exit = out.ended.map(|end| match end {
        WallEnd::Kicked => Trigger::WallKick,
        WallEnd::TimedOut => Trigger::WallTimeout,
        WallEnd::Detached => Trigger::WallDetach,
        WallEnd::Lost => Trigger::WallLost,
    });
    match exit {
        Some(Trigger::WallKick) => {
            s.jumps_used = 1;
            ev.push(Event::WallKicked);
        }
        // Leaving without a kick: gravity takes over on this very step.
        Some(_) => s.velocity.z -= t.gravity * dt,
        None => {}
    }

    let fall_speed = (-s.velocity.z).max(0.0);
    let hits = air_move(s, t, world, dt);

    if exit.is_none() {
        let body = centre(s, t);
        let on_floor = hits.iter().any(|h| {
            h.normal.z > t.wall_max_normal_z
                && is_walkable(face_normal(world, h.point, body, h.normal, t).0, t)
        });
        if on_floor {
            leave(s);
            ev.push(Event::WallrunEnded);
            return Some(airborne::land(s, input, t, fall_speed, ev));
        }
        if let Some(target) = mantle::find_ledge(s, input, t, world, s.wall_normal) {
            leave(s);
            ev.push(Event::WallrunEnded);
            mantle::begin(s, target, ev);
            return Some(Trigger::LedgeGrab);
        }
    }

    if let Some(trigger) = exit {
        ev.push(Event::WallrunEnded);
        return Some(trigger);
    }
    None
}
