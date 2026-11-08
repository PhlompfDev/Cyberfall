//! Game-facing wrapper around the wall controller: set inputs, `step`, read results.
//! Cyberpunk-only adjustments (`host_*` in tuning/pilot.toml) are applied here.

use glam::Vec3;
use pilot_core::collide::is_wall;
use pilot_core::settings::{apply, WallSettings};
use pilot_core::wall::{leave, touched_ground, wall_step, WallEvent, WallEnd, WallInput, WallReading, WallState, WallStep};
use pilot_core::Tuning;

const TUNING: &str = include_str!("../../../tuning/pilot.toml");

pub const ENDED_NONE: i32 = 0;
pub const ENDED_KICKED: i32 = 1;
pub const ENDED_TIMED_OUT: i32 = 2;
pub const ENDED_DETACHED: i32 = 3;
pub const ENDED_LOST: i32 = 4;

pub struct Bridge {
    base: Tuning,
    tuning: Tuning,
    settings: WallSettings,
    state: WallState,
    velocity: Vec3,
    reading: Option<WallReading>,
    reading_heights: i32,
    input: WallInput,
    last: Option<WallStep>,
    camera_roll: f32,
    missing_for: f32,
    // Last frame's horizontal speed, before Cyberpunk's collision clips it.
    prev_speed: f32,
    // (distance, clearance) of a bump ahead, for the next step only.
    ahead: Option<(f32, f32)>,
    touch_refresh: bool,
}

impl Default for Bridge {
    fn default() -> Bridge {
        Bridge::new()
    }
}

impl Bridge {
    pub fn new() -> Bridge {
        let base = pilot_tuning::parse(TUNING).expect("the compiled-in tuning must parse");
        Bridge {
            base,
            tuning: for_cyberpunk(&base),
            settings: WallSettings::TITANFALL,
            state: WallState::default(),
            velocity: Vec3::ZERO,
            reading: None,
            reading_heights: 0,
            input: WallInput::default(),
            last: None,
            camera_roll: 0.0,
            missing_for: 0.0,
            prev_speed: 0.0,
            ahead: None,
            touch_refresh: false,
        }
    }

    pub fn base_tuning(&self) -> &Tuning {
        &self.base
    }

    pub fn tuning(&self) -> &Tuning {
        &self.tuning
    }

    pub fn state(&self) -> &WallState {
        &self.state
    }

    pub fn set_settings(&mut self, enabled: bool, needs_perk: bool, length: f32, speed: f32, kick: f32, tilt: f32) {
        self.settings = WallSettings { enabled, needs_perk, length, speed, kick, tilt };
        self.tuning = for_cyberpunk(&apply(&self.base, &self.settings));
    }

    pub fn enabled(&self) -> bool {
        self.settings.enabled
    }

    pub fn needs_perk(&self) -> bool {
        self.settings.needs_perk
    }

    pub fn set_velocity(&mut self, x: f32, y: f32, z: f32) {
        self.velocity = Vec3::new(x, y, z);
    }

    /// `heights`: how many of the probe heights saw this wall.
    pub fn set_reading(&mut self, has_wall: bool, nx: f32, ny: f32, nz: f32, distance: f32, heights: i32) {
        self.reading = has_wall.then_some(WallReading { normal: Vec3::new(nx, ny, nz), distance });
        self.reading_heights = heights;
    }

    pub fn set_input(&mut self, forward: f32, right: f32, facing_x: f32, facing_y: f32, jump: bool, crouch: bool) {
        self.input = WallInput {
            move_forward: forward,
            move_right: right,
            yaw: facing_y.atan2(facing_x),
            jump_pressed: jump,
            crouch_pressed: crouch,
        };
    }

    /// `clearance` below zero means a solid wall ahead.
    pub fn set_ahead(&mut self, blocked: bool, distance: f32, clearance: f32) {
        self.ahead = blocked.then_some((distance, clearance));
    }

    pub fn wall_max_normal_z(&self) -> f32 {
        self.tuning.wall_max_normal_z
    }

    pub fn bump_lookahead(&self) -> f32 {
        self.tuning.host_bump_lookahead
    }

    pub fn bump_max_clearance(&self) -> f32 {
        self.tuning.host_bump_max_clearance
    }

    pub fn input_yaw(&self) -> f32 {
        self.input.yaw
    }

    /// One frame. Returns whether V is wallrunning.
    pub fn step(&mut self, ground_clear: bool, dt: f32) -> bool {
        if !self.settings.enabled {
            let was_running = self.state.running;
            self.state.running = false;
            self.touch_refresh = false;
            self.last = Some(WallStep {
                velocity: self.velocity,
                state: self.state,
                events: Vec::new(),
                ended: was_running.then_some(WallEnd::Lost),
                roll: 0.0,
                refresh_air_moves: false,
            });
            return false;
        }
        let speed_now = flat(self.velocity).length();
        let entry_speed = speed_now.max(self.prev_speed);
        self.prev_speed = speed_now;
        let ahead = self.ahead.take();
        let reading = self.bridged_reading(dt);
        let gap = reading.map_or(0.0, |r| r.distance.max(0.0));
        let out = wall_step(&self.state, self.velocity, reading, ground_clear, &self.input, &self.tuning, dt);
        self.state = out.state;
        self.velocity = out.velocity;
        if out.events.contains(&WallEvent::Started) {
            self.velocity = keep_entry_speed(self.velocity, entry_speed, self.tuning.host_entry_keep);
        }
        self.touch_refresh = !self.state.running && reading.is_some_and(|r| self.touches_wall(&r));
        if self.state.running {
            let n = self.state.normal;
            match self.bump_out(ahead, gap, dt) {
                Some(out_speed) => self.velocity += n * out_speed,
                None => self.velocity -= n * self.tuning.host_wall_glue_speed,
            }
        }
        self.last = Some(out);
        self.state.running
    }

    /// Any wall touch gives back air moves, even one that doesn't start a run.
    fn touches_wall(&self, r: &WallReading) -> bool {
        let t = &self.tuning;
        is_wall(r.normal, t) && r.distance <= t.wallrun_grab_distance
    }

    /// Speed out from the wall that gets V around a bump in time.
    fn bump_out(&self, ahead: Option<(f32, f32)>, gap: f32, dt: f32) -> Option<f32> {
        let (distance, clearance) = ahead?;
        let needed = clearance - gap;
        if needed <= 0.0 {
            return None;
        }
        let n = self.state.normal;
        let along = flat(self.velocity - n * self.velocity.dot(n)).length().max(0.1);
        let time = (distance / along).max(dt);
        Some((needed / time).min(self.tuning.host_bump_speed))
    }

    /// Mid-run, a missing wall (pipe, seam, window) is assumed to still be there for a short grace.
    fn bridged_reading(&mut self, dt: f32) -> Option<WallReading> {
        let t = &self.tuning;
        let tall = self.reading_heights as f32 >= t.host_wall_min_heights;
        let reading = self.reading.filter(|_| tall);
        let counts = |r: &WallReading| is_wall(r.normal, t) && r.distance <= t.wallrun_stick_distance;
        let gap = self.state.running && !reading.as_ref().is_some_and(counts);
        if !gap {
            self.missing_for = 0.0;
            return reading;
        }
        if self.missing_for + dt > self.tuning.host_wall_lost_grace + 1e-4 {
            return reading;
        }
        self.missing_for += dt;
        Some(WallReading { normal: self.state.normal, distance: 0.0 })
    }

    pub fn running(&self) -> bool {
        self.state.running
    }

    pub fn velocity(&self) -> Vec3 {
        self.velocity
    }

    pub fn started(&self) -> bool {
        self.last.as_ref().is_some_and(|l| l.events.contains(&WallEvent::Started))
    }

    pub fn ended(&self) -> i32 {
        match self.last.as_ref().and_then(|l| l.ended) {
            None => ENDED_NONE,
            Some(WallEnd::Kicked) => ENDED_KICKED,
            Some(WallEnd::TimedOut) => ENDED_TIMED_OUT,
            Some(WallEnd::Detached) => ENDED_DETACHED,
            Some(WallEnd::Lost) => ENDED_LOST,
        }
    }

    pub fn refresh_air_moves(&self) -> bool {
        self.touch_refresh || self.last.as_ref().is_some_and(|l| l.refresh_air_moves)
    }

    pub fn roll(&self) -> f32 {
        self.last.as_ref().map_or(0.0, |l| l.roll)
    }

    pub fn advance_air(&mut self, dt: f32) {
        if !self.state.running {
            self.state.time_since_wall += dt;
        }
    }

    pub fn air_dash(&mut self) {
        if self.state.running {
            self.state = leave(&self.state);
        }
    }

    pub fn touched_ground(&mut self) {
        self.prev_speed = 0.0;
        self.state = touched_ground(&self.state);
    }

    pub fn camera_roll(&mut self, dt: f32) -> f32 {
        let target = if self.state.running { self.roll() } else { 0.0 };
        let step = self.tuning.roll_rate_deg.to_radians() * dt;
        let gap = target - self.camera_roll;
        self.camera_roll = if gap.abs() <= step { target } else { self.camera_roll + gap.signum() * step };
        self.camera_roll
    }

    pub fn ground_clearance(&self) -> f32 {
        self.tuning.wallrun_min_ground_clearance
    }

    pub fn stick_distance(&self) -> f32 {
        self.tuning.wallrun_stick_distance
    }

    pub fn reset(&mut self) {
        self.state = WallState::default();
        self.last = None;
        self.camera_roll = 0.0;
        self.missing_for = 0.0;
        self.prev_speed = 0.0;
        self.ahead = None;
        self.touch_refresh = false;
    }
}

fn flat(v: Vec3) -> Vec3 {
    Vec3::new(v.x, v.y, 0.0)
}

/// Turn the speed V had going into the wall along it, instead of losing it.
fn keep_entry_speed(velocity: Vec3, entry_speed: f32, keep: f32) -> Vec3 {
    let h = flat(velocity);
    let along = h.length();
    if along < 1e-4 {
        return velocity;
    }
    let target = along + (entry_speed - along).max(0.0) * keep;
    h * (target / along) + Vec3::Z * velocity.z
}

fn for_cyberpunk(t: &Tuning) -> Tuning {
    let mut out = *t;
    out.wall_kick_up_speed *= t.host_kick_up_scale;
    out.wallrun_grab_distance = t.host_grab_distance;
    out
}
