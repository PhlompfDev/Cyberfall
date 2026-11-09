//! Titanfall momentum slide for Cyberpunk: crouch into a landing to slide at the speed
//! V fell with, and slides keep and lose speed by Titanfall's curve instead of the game's.

use glam::Vec3;
use pilot_core::slide::{boosted_speed, slide_velocity};
use pilot_core::types::Input;
use pilot_core::Tuning;

const MIN_SCALE: f32 = 0.25;
const MAX_SCALE: f32 = 2.0;
/// Reported speed above the slide's own that still counts as no loss, m/s. Cyberpunk's
/// controller clips a little speed every frame; only a real hit should slow a slide.
const COLLISION_SLACK: f32 = 0.5;

pub struct Slide {
    base: Tuning,
    tuning: Tuning,
    enabled: bool,
    active: bool,
    velocity: Vec3,
    // The last two airborne horizontal velocities, newest first.
    air: [Vec3; 2],
    since_crouch_press: f32,
    crouch_held_in_air: bool,
    // Toggle crouch: a tap in the air switches crouch on for the landing, a second tap off.
    toggle_armed: bool,
    last_begin: Option<f32>,
    // The game hasn't been given the start speed yet, so its first report says nothing about collisions.
    fresh: bool,
    input: Input,
    crouch_held: bool,
}

impl Slide {
    pub fn new(base: &Tuning) -> Slide {
        Slide {
            base: *base,
            tuning: *base,
            enabled: true,
            active: false,
            velocity: Vec3::ZERO,
            air: [Vec3::ZERO; 2],
            since_crouch_press: f32::INFINITY,
            crouch_held_in_air: false,
            toggle_armed: false,
            last_begin: None,
            fresh: false,
            input: Input::default(),
            crouch_held: false,
        }
    }

    /// `speed` scales the slide boost and its cap, 25% to 200%.
    pub fn set_settings(&mut self, enabled: bool, speed: f32) {
        let scale = speed.clamp(MIN_SCALE, MAX_SCALE);
        self.enabled = enabled;
        self.tuning = self.base;
        self.tuning.slide_boost_speed *= scale;
        self.tuning.slide_boost_max_speed *= scale;
        if !enabled {
            self.end();
        }
    }

    pub fn enabled(&self) -> bool {
        self.enabled
    }

    pub fn tuning(&self) -> &Tuning {
        &self.tuning
    }

    /// Every airborne frame. `crouch_pressed`/`crouch_held` are hold-mode crouch, `toggle_tap` a toggle-crouch tap.
    pub fn note_air(&mut self, vx: f32, vy: f32, crouch_pressed: bool, crouch_held: bool, toggle_tap: bool, dt: f32) {
        self.toggle_armed ^= toggle_tap;
        self.air = [Vec3::new(vx, vy, 0.0), self.air[0]];
        self.since_crouch_press = if crouch_pressed { 0.0 } else { self.since_crouch_press + dt };
        self.crouch_held_in_air = crouch_held;
    }

    fn air_speed(&self) -> f32 {
        self.air[0].length().max(self.air[1].length())
    }

    /// Landing now should go straight into a slide.
    pub fn wants_landing_slide(&self) -> bool {
        let crouched = self.crouch_held_in_air || self.toggle_armed || self.since_crouch_press <= self.tuning.host_slide_crouch_window + 1e-4;
        self.enabled && crouched && self.air_speed() >= self.tuning.host_slide_entry_speed
    }

    /// Landed without sliding: the fall no longer counts.
    pub fn landed(&mut self) {
        self.air = [Vec3::ZERO; 2];
        self.since_crouch_press = f32::INFINITY;
        self.crouch_held_in_air = false;
        self.toggle_armed = false;
    }

    /// A slide starts, at game time `now` seconds. A landing slide keeps the speed from before impact.
    pub fn begin(&mut self, vx: f32, vy: f32, now: f32, from_landing: bool) {
        let current = Vec3::new(vx, vy, 0.0);
        let mut speed = current.length();
        let mut direction = current.normalize_or_zero();
        if from_landing {
            speed = speed.max(self.air_speed());
            if direction == Vec3::ZERO {
                direction = self.air[0].normalize_or_zero();
            }
        }
        // The cooldown restarts on every slide, so chained slides don't boost twice.
        let boost = self.last_begin.is_none_or(|last| now - last >= self.tuning.slide_boost_cooldown);
        if boost {
            speed = boosted_speed(speed, &self.tuning);
        }
        self.last_begin = Some(now);
        self.velocity = direction * speed;
        self.active = true;
        self.fresh = true;
        self.landed();
    }

    pub fn set_input(&mut self, forward: f32, right: f32, facing_x: f32, facing_y: f32, crouch_held: bool) {
        self.input = Input { move_forward: forward, move_right: right, yaw: facing_y.atan2(facing_x), ..Input::default() };
        self.crouch_held = crouch_held;
    }

    /// One slide frame from the game's velocity and the ground normal (zero: none seen).
    /// Returns whether the slide goes on.
    pub fn step(&mut self, vx: f32, vy: f32, vz: f32, nx: f32, ny: f32, nz: f32, dt: f32) -> bool {
        if !self.active {
            return false;
        }
        let reported = Vec3::new(vx, vy, 0.0);
        let own = Vec3::new(self.velocity.x, self.velocity.y, 0.0);
        let held = if self.fresh { own.length() } else { own.length().min(reported.length() + COLLISION_SLACK) };
        let direction = if self.fresh { own.normalize_or_zero() } else { reported.try_normalize().unwrap_or(own.normalize_or_zero()) };
        self.fresh = false;
        let normal = Vec3::new(nx, ny, nz);
        let ground = (normal.length_squared() > 1e-6).then(|| normal.normalize());
        let flat = slide_velocity(direction * held, self.input.wish_dir(), ground, &self.tuning, dt);
        self.velocity = Vec3::new(flat.x, flat.y, vz);
        flat.length() >= self.tuning.slide_exit_speed && self.crouch_held
    }

    pub fn velocity(&self) -> Vec3 {
        self.velocity
    }

    pub fn active(&self) -> bool {
        self.active
    }

    pub fn end(&mut self) {
        self.active = false;
    }
}
