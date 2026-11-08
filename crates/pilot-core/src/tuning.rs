use serde::{Deserialize, Serialize};

use crate::world::Capsule;

/// Every number the simulation uses. Metres, seconds; angles in degrees.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Tuning {
    // Body
    pub capsule_radius: f32,
    pub stand_height: f32,
    pub crouch_height: f32,
    pub stand_eye_height: f32,
    pub crouch_eye_height: f32,
    pub slide_eye_height: f32,
    pub eye_height_rate: f32,
    // Collision
    pub collision_skin: f32,
    pub max_bumps: u32,
    pub step_height: f32,
    pub max_slope_deg: f32,
    pub ground_probe_distance: f32,
    pub wall_max_normal_z: f32,
    pub surface_probe_out: f32,
    pub surface_probe_up: f32,
    // Ground
    pub gravity: f32,
    pub walk_speed: f32,
    pub sprint_speed: f32,
    pub crouch_speed: f32,
    pub ground_accel: f32,
    pub ground_friction: f32,
    pub landing_friction_grace: f32,
    // Air
    pub jump_speed: f32,
    pub double_jump_speed: f32,
    pub double_jump_boost: f32,
    pub air_accel: f32,
    pub air_wish_cap: f32,
    // Slide
    pub slide_entry_speed: f32,
    pub slide_exit_speed: f32,
    pub slide_decel: f32,
    pub slide_decay_rate: f32,
    pub slide_boost_speed: f32,
    pub slide_boost_max_speed: f32,
    pub slide_boost_cooldown: f32,
    pub slide_steer_accel: f32,
    // Wallrun
    pub wallrun_min_speed: f32,
    pub wallrun_max_away_speed: f32,
    pub wallrun_speed: f32,
    pub wallrun_accel_rate: f32,
    pub wallrun_time_limit: f32,
    pub wallrun_vertical_damping: f32,
    pub wallrun_sink_ramp: f32,
    pub wallrun_lift: f32,
    pub wallrun_grab_distance: f32,
    pub wallrun_entry_up_speed: f32,
    pub wallrun_min_ground_clearance: f32,
    pub wallrun_stick_distance: f32,
    pub wallrun_same_wall_delay: f32,
    pub wallrun_same_wall_dot: f32,
    pub wallrun_detach_dot: f32,
    pub wallrun_stall_speed: f32,
    pub wall_kick_out_speed: f32,
    pub wall_kick_up_speed: f32,
    pub wall_kick_look_speed: f32,
    pub wall_fall_off_push: f32,
    pub wallrun_roll_deg: f32,
    pub roll_rate_deg: f32,
    pub max_wall_dt: f32,
    // Mantle
    pub mantle_min_height: f32,
    pub mantle_max_height: f32,
    pub mantle_forward: f32,
    pub mantle_time: f32,
    pub mantle_exit_speed: f32,
    pub mantle_min_approach_dot: f32,
    // Cyberpunk only, read by cyberfall-bridge.
    pub host_kick_up_scale: f32,
    pub host_wall_glue_speed: f32,
    pub host_wall_lost_grace: f32,
    pub host_entry_keep: f32,
    pub host_bump_speed: f32,
    pub host_bump_lookahead: f32,
    pub host_bump_max_clearance: f32,
    pub host_grab_distance: f32,
    pub host_wall_min_heights: f32,
    pub host_late_kick_window: f32,
}

impl Tuning {
    pub fn stand_capsule(&self) -> Capsule {
        Capsule { radius: self.capsule_radius, half_height: self.stand_height * 0.5 - self.capsule_radius }
    }

    pub fn crouch_capsule(&self) -> Capsule {
        Capsule { radius: self.capsule_radius, half_height: self.crouch_height * 0.5 - self.capsule_radius }
    }

    /// A surface is walkable when its normal's z is at least this.
    pub fn max_slope_cos(&self) -> f32 {
        self.max_slope_deg.to_radians().cos()
    }
}
