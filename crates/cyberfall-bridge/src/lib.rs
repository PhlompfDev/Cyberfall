//! Game-facing wrapper around the wall controller: set inputs, `step`, read results.
//! Cyberpunk-only adjustments (`host_*` in tuning/pilot.toml) are applied here.

pub mod charge;
pub mod slide;

use charge::{charged, tier_for_hold};
use slide::Slide;
use glam::Vec3;
use pilot_core::collide::is_wall;
use pilot_core::settings::{apply, WallSettings};
use pilot_core::wall::{kick_velocity, leave, touched_ground, wall_step, WallEvent, WallEnd, WallInput, WallReading, WallState, WallStep};
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
    kick_window: f32,
    kick_normal: Vec3,
    // Fastest along-wall speed of the current run.
    run_speed: f32,
    // Charge-jump legs: the kick fires on release and grows with the hold.
    has_charge_jump: bool,
    jump_held: bool,
    charging: bool,
    charge_time: f32,
    // Charge tier of the most recent kick, 0 for a normal one.
    kick_tier: i32,
    slide: Slide,
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
            kick_window: 0.0,
            kick_normal: Vec3::ZERO,
            run_speed: 0.0,
            has_charge_jump: false,
            jump_held: false,
            charging: false,
            charge_time: 0.0,
            kick_tier: 0,
            slide: Slide::new(&base),
        }
    }

    pub fn slide(&self) -> &Slide {
        &self.slide
    }

    pub fn slide_mut(&mut self) -> &mut Slide {
        &mut self.slide
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

    /// Every air frame, before `step`: whether the legs give the charge jump, and whether Jump is down.
    pub fn set_charge(&mut self, has_charge_jump: bool, jump_held: bool) {
        self.has_charge_jump = has_charge_jump;
        self.jump_held = jump_held;
    }

    pub fn charging(&self) -> bool {
        self.charging
    }

    /// Charge tier of the most recent kick: 0 normal, 3 full.
    pub fn charge_tier(&self) -> i32 {
        self.kick_tier
    }

    /// With charge-jump legs the kick waits for Jump to be let go.
    fn charge_frame(&mut self, dt: f32) -> (WallInput, Tuning) {
        let mut input = self.input;
        if !self.state.running || !self.has_charge_jump {
            return (input, self.tuning);
        }
        if !self.charging && input.jump_pressed {
            self.charging = true;
            self.charge_time = 0.0;
        }
        if !self.charging {
            return (input, self.tuning);
        }
        if self.jump_held {
            self.charge_time += dt;
            input.jump_pressed = false;
            return (input, self.tuning);
        }
        input.jump_pressed = true;
        (input, charged(&self.tuning, tier_for_hold(self.charge_time, &self.tuning).1))
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

    /// Bumps and bends in the wall don't cost speed along it.
    fn hold_run_speed(&mut self, n: Vec3) {
        let along = flat(self.velocity - n * self.velocity.dot(n));
        let speed = along.length();
        if speed > 1e-3 && speed < self.run_speed {
            self.velocity += along * (self.run_speed / speed - 1.0);
        }
        self.run_speed = self.run_speed.max(speed);
    }

    /// Jump was pressed just after leaving a wall without a kick: kick off it anyway.
    fn late_kick(&mut self) -> bool {
        let look = Vec3::new(self.input.yaw.cos(), self.input.yaw.sin(), 0.0);
        self.velocity = kick_velocity(self.velocity, self.kick_normal, look, &self.tuning);
        self.kick_window = 0.0;
        self.kick_tier = 0;
        self.touch_refresh = false;
        self.last = Some(WallStep {
            velocity: self.velocity,
            state: self.state,
            events: vec![WallEvent::Kicked],
            ended: Some(WallEnd::Kicked),
            roll: 0.0,
            refresh_air_moves: false,
        });
        false
    }

    /// A jump now still kicks off the wall just left.
    pub fn kick_window_open(&self) -> bool {
        self.kick_window > 0.0
    }

    /// One frame. Returns whether V is wallrunning.
    pub fn step(&mut self, ground_clear: bool, dt: f32) -> bool {
        if !self.settings.enabled {
            let was_running = self.state.running;
            self.state.running = false;
            self.charging = false;
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
        if !self.state.running && self.input.jump_pressed && self.kick_window > 0.0 {
            return self.late_kick();
        }
        self.kick_window = (self.kick_window - dt).max(0.0);
        let speed_now = flat(self.velocity).length();
        let entry_speed = speed_now.max(self.prev_speed);
        self.prev_speed = speed_now;
        let ahead = self.ahead.take();
        let reading = self.bridged_reading(dt);
        let gap = reading.map_or(0.0, |r| r.distance.max(0.0));
        let velocity_before = self.velocity;
        let (input, tuning) = self.charge_frame(dt);
        let mut out = wall_step(&self.state, self.velocity, reading, ground_clear, &input, &tuning, dt);
        if self.charging {
            let tier = tier_for_hold(self.charge_time, &self.tuning);
            if out.ended.is_some_and(|end| end != WallEnd::Kicked) {
                // The run ended mid-charge: kick with what was charged.
                let look = Vec3::new(self.input.yaw.cos(), self.input.yaw.sin(), 0.0);
                out.velocity = kick_velocity(velocity_before, out.state.last_normal, look, &charged(&self.tuning, tier.1));
                out.ended = Some(WallEnd::Kicked);
                out.events.push(WallEvent::Kicked);
            }
            if out.ended.is_some() {
                self.kick_tier = tier.0;
            }
        } else if out.ended == Some(WallEnd::Kicked) {
            self.kick_tier = 0;
        }
        if !out.state.running {
            self.charging = false;
        }
        self.state = out.state;
        self.velocity = out.velocity;
        if out.ended.is_some_and(|end| end != WallEnd::Kicked) {
            self.kick_window = self.tuning.host_late_kick_window;
            self.kick_normal = self.state.last_normal;
        }
        if out.events.contains(&WallEvent::Started) {
            self.velocity = keep_entry_speed(self.velocity, entry_speed, self.tuning.host_entry_keep);
            self.run_speed = 0.0;
        }
        self.touch_refresh = !self.state.running && reading.is_some_and(|r| self.touches_wall(&r));
        if self.state.running {
            let n = self.state.normal;
            self.hold_run_speed(n);
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
        self.charging = false;
        if self.state.running {
            self.state = leave(&self.state);
        }
    }

    pub fn touched_ground(&mut self) {
        self.charging = false;
        self.prev_speed = 0.0;
        self.kick_window = 0.0;
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
        self.kick_window = 0.0;
        self.charging = false;
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
