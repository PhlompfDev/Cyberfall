//! Player settings. Multipliers of 1.0 are Titanfall.

use crate::tuning::Tuning;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct WallSettings {
    pub enabled: bool,
    pub needs_perk: bool,
    pub length: f32,
    pub speed: f32,
    pub kick: f32,
    /// 0 turns the camera lean off.
    pub tilt: f32,
}

impl WallSettings {
    pub const TITANFALL: WallSettings =
        WallSettings { enabled: true, needs_perk: true, length: 1.0, speed: 1.0, kick: 1.0, tilt: 1.0 };
}

const MIN_SCALE: f32 = 0.25;
const MAX_SCALE: f32 = 2.0;

pub fn apply(t: &Tuning, s: &WallSettings) -> Tuning {
    let scale = |x: f32| x.clamp(MIN_SCALE, MAX_SCALE);
    let kick = scale(s.kick);
    let mut out = *t;
    out.wallrun_time_limit *= scale(s.length);
    out.wallrun_speed *= scale(s.speed);
    out.wall_kick_out_speed *= kick;
    out.wall_kick_up_speed *= kick;
    out.wall_kick_look_speed *= kick;
    out.wallrun_roll_deg *= s.tilt.clamp(0.0, MAX_SCALE);
    out
}
