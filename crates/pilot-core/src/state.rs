use glam::Vec3;
use serde::{Deserialize, Serialize};

use crate::tuning::Tuning;
use crate::types::{CameraOut, Mode};

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct PilotState {
    /// Feet: the bottom of the capsule.
    pub position: Vec3,
    pub velocity: Vec3,
    pub mode: Mode,
    /// Seconds spent in `mode` before the current step.
    pub mode_time: f32,
    pub crouched: bool,
    /// Latched until forward is released.
    pub sprinting: bool,
    /// 0 on the ground, 1 after a jump or leaving a wall, 2 after a double jump.
    pub jumps_used: u8,
    pub slide_boost_cooldown: f32,
    /// Normal of the wall being run on or hung from.
    pub wall_normal: Vec3,
    /// Normal of the wall most recently left, for the same-wall rule.
    pub last_wall_normal: Vec3,
    pub time_since_wall: f32,
    pub grounded_since_wall: bool,
    pub mantle_from: Vec3,
    pub mantle_to: Vec3,
    pub camera: CameraOut,
}

impl PilotState {
    pub fn spawn(position: Vec3, t: &Tuning) -> PilotState {
        PilotState {
            position,
            velocity: Vec3::ZERO,
            mode: Mode::Grounded,
            mode_time: 0.0,
            crouched: false,
            sprinting: false,
            jumps_used: 0,
            slide_boost_cooldown: 0.0,
            wall_normal: Vec3::ZERO,
            last_wall_normal: Vec3::ZERO,
            time_since_wall: 0.0,
            grounded_since_wall: true,
            mantle_from: position,
            mantle_to: position,
            camera: CameraOut { eye_height: t.stand_eye_height, roll: 0.0 },
        }
    }
}
