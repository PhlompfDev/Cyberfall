//! Wallrun rules, independent of any host. Call `wall_step` each airborne frame
//! and apply the velocity it returns.

use glam::Vec3;

use crate::collide::is_wall;
use crate::kinematics::{clip, horizontal};
use crate::tuning::Tuning;
use crate::types::{Input, UP};

/// What the host found beside the player.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct WallReading {
    /// Surface normal of the wall, pointing away from it.
    pub normal: Vec3,
    /// Gap between the player's side and the wall, metres.
    pub distance: f32,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct WallInput {
    pub move_forward: f32,
    pub move_right: f32,
    /// Radians about +Z; 0 faces +X.
    pub yaw: f32,
    pub jump_pressed: bool,
    pub crouch_pressed: bool,
}

impl WallInput {
    fn as_input(&self) -> Input {
        Input { move_forward: self.move_forward, move_right: self.move_right, yaw: self.yaw, ..Input::default() }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct WallState {
    pub running: bool,
    /// The wall being run on.
    pub normal: Vec3,
    /// Seconds on the current wall.
    pub elapsed: f32,
    /// The wall most recently left, for the same-wall rule.
    pub last_normal: Vec3,
    pub time_since_wall: f32,
    pub grounded_since_wall: bool,
}

impl Default for WallState {
    /// Not on a wall, and free to grab any wall.
    fn default() -> WallState {
        WallState {
            running: false,
            normal: Vec3::ZERO,
            elapsed: 0.0,
            last_normal: Vec3::ZERO,
            time_since_wall: 0.0,
            grounded_since_wall: true,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WallEvent {
    Started,
    Ended,
    Kicked,
}

/// Why a wallrun ended.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WallEnd {
    Kicked,
    TimedOut,
    Detached,
    Lost,
}

#[derive(Clone, Debug, PartialEq)]
pub struct WallStep {
    pub velocity: Vec3,
    pub state: WallState,
    pub events: Vec<WallEvent>,
    pub ended: Option<WallEnd>,
    /// Camera roll the view should lean toward, radians, positive clockwise.
    pub roll: f32,
    /// The host should give back the air dash and the double jump.
    pub refresh_air_moves: bool,
}

/// The host saw the player land: any wall may be grabbed again.
pub fn touched_ground(state: &WallState) -> WallState {
    WallState { running: false, grounded_since_wall: true, ..*state }
}

/// Whether a run may start on a wall with `normal`.
pub fn start_allowed(state: &WallState, velocity: Vec3, normal: Vec3, ground_clear: bool, t: &Tuning) -> bool {
    if horizontal(clip(velocity, normal)).length() < t.wallrun_min_speed
        || horizontal(velocity).dot(normal) > t.wallrun_max_away_speed
    {
        return false;
    }
    !same_wall(state, normal, t) && ground_clear
}

/// The wall just left can't be run again until landing or `wallrun_same_wall_delay`.
pub fn same_wall(state: &WallState, normal: Vec3, t: &Tuning) -> bool {
    !state.grounded_since_wall
        && state.time_since_wall < t.wallrun_same_wall_delay
        && state.last_normal.dot(normal) >= t.wallrun_same_wall_dot
}

/// Velocity on touching a wall: into-wall speed dropped, plus a capped upward lift.
pub fn entry_velocity(velocity: Vec3, normal: Vec3, t: &Tuning) -> Vec3 {
    let mut v = clip(velocity, normal);
    v.z = (v.z + t.wallrun_entry_up_speed).min(t.wallrun_entry_up_speed);
    v
}

/// Velocity after a wall kick.
pub fn kick_velocity(velocity: Vec3, normal: Vec3, look: Vec3, t: &Tuning) -> Vec3 {
    let along_look = clip(horizontal(look), normal);
    let mut v = clip(velocity, normal) + along_look * t.wall_kick_look_speed + normal * t.wall_kick_out_speed;
    v.z = t.wall_kick_up_speed;
    v
}

/// Velocity after one step of running along the wall.
pub fn run_velocity(velocity: Vec3, normal: Vec3, elapsed: f32, dt: f32, t: &Tuning) -> Vec3 {
    let side = UP.cross(normal);
    let tangent = if velocity.dot(side) >= 0.0 { side } else { -side };
    let mut along = velocity.dot(tangent);
    if along < t.wallrun_speed {
        along = t.wallrun_speed - (t.wallrun_speed - along) * (-t.wallrun_accel_rate * dt).exp();
    }
    // Small lift at first, sinking the longer the run lasts.
    let target = t.wallrun_lift - t.wallrun_sink_ramp * elapsed;
    let vertical = velocity.z + t.wallrun_vertical_damping * (target - velocity.z) * dt;
    tangent * along + UP * vertical
}

/// Camera lean away from the wall.
pub fn roll_target(input: &WallInput, normal: Vec3, t: &Tuning) -> f32 {
    let wall_side = input.as_input().right().dot(-normal).clamp(-1.0, 1.0);
    -wall_side * t.wallrun_roll_deg.to_radians()
}

pub fn leave(state: &WallState) -> WallState {
    WallState {
        running: false,
        last_normal: state.normal,
        time_since_wall: 0.0,
        grounded_since_wall: false,
        ..*state
    }
}

/// One frame of wall logic. Long frames are split into steps of at most `max_wall_dt`.
pub fn wall_step(
    state: &WallState,
    velocity: Vec3,
    reading: Option<WallReading>,
    ground_clear: bool,
    input: &WallInput,
    t: &Tuning,
    dt: f32,
) -> WallStep {
    let pieces = (dt / t.max_wall_dt).ceil().max(1.0) as usize;
    let piece = dt / pieces as f32;
    let mut out = single(state, velocity, reading, ground_clear, input, t, piece);
    for _ in 1..pieces {
        if !out.state.running {
            break;
        }
        let next = single(&out.state, out.velocity, reading, ground_clear, input, t, piece);
        let mut events = out.events;
        events.extend(next.events.iter().copied());
        out = WallStep { events, refresh_air_moves: out.refresh_air_moves || next.refresh_air_moves, ..next };
    }
    out
}

fn single(
    state: &WallState,
    velocity: Vec3,
    reading: Option<WallReading>,
    ground_clear: bool,
    input: &WallInput,
    t: &Tuning,
    dt: f32,
) -> WallStep {
    let quiet = |state: WallState, velocity: Vec3| WallStep {
        velocity,
        state,
        events: Vec::new(),
        ended: None,
        roll: 0.0,
        refresh_air_moves: false,
    };
    let reach = if state.running { t.wallrun_stick_distance } else { t.wallrun_grab_distance };
    let touching = reading.filter(|r| is_wall(r.normal, t) && r.distance <= reach);

    if !state.running {
        let Some(wall) = touching else { return quiet(*state, velocity) };
        let normal = horizontal(wall.normal).normalize_or_zero();
        let steering_away = input.as_input().wish_dir().dot(normal) > t.wallrun_detach_dot;
        if steering_away || !start_allowed(state, velocity, normal, ground_clear, t) {
            return quiet(*state, velocity);
        }
        let started = WallState { running: true, normal, elapsed: 0.0, ..*state };
        return WallStep {
            velocity: entry_velocity(velocity, normal, t),
            state: started,
            events: vec![WallEvent::Started],
            ended: None,
            roll: roll_target(input, normal, t),
            refresh_air_moves: true,
        };
    }

    let mut s = *state;
    let ended = if input.jump_pressed {
        Some(WallEnd::Kicked)
    } else if s.elapsed >= t.wallrun_time_limit {
        Some(WallEnd::TimedOut)
    } else if input.crouch_pressed
        || input.as_input().wish_dir().dot(s.normal) > t.wallrun_detach_dot
        || horizontal(velocity).length() < t.wallrun_stall_speed
    {
        Some(WallEnd::Detached)
    } else if let Some(wall) = touching {
        // Follow the wall's normal if it curves.
        s.normal = horizontal(wall.normal).normalize_or(s.normal);
        None
    } else {
        Some(WallEnd::Lost)
    };

    match ended {
        None => {
            let v = run_velocity(velocity, s.normal, s.elapsed, dt, t);
            s.elapsed += dt;
            WallStep {
                velocity: v,
                state: s,
                events: Vec::new(),
                ended: None,
                roll: roll_target(input, s.normal, t),
                refresh_air_moves: false,
            }
        }
        Some(WallEnd::Kicked) => WallStep {
            velocity: kick_velocity(velocity, s.normal, input.as_input().forward(), t),
            state: leave(&s),
            events: vec![WallEvent::Kicked, WallEvent::Ended],
            ended,
            roll: 0.0,
            refresh_air_moves: false,
        },
        Some(_) => WallStep {
            velocity: velocity + s.normal * t.wall_fall_off_push,
            state: leave(&s),
            events: vec![WallEvent::Ended],
            ended,
            roll: 0.0,
            refresh_air_moves: false,
        },
    }
}
