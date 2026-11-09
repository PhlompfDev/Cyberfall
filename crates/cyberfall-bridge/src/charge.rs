//! Charged wall kick for charge-jump legs: how long Jump is held on the wall sets the kick.

use pilot_core::Tuning;

/// (tier, kick scale) for Jump held `held` seconds. Tier 0 is a normal kick.
pub fn tier_for_hold(held: f32, t: &Tuning) -> (i32, f32) {
    if held >= t.host_charge_max_time {
        (3, t.host_charge_max_scale)
    } else if held > t.host_charge_med_time {
        (2, t.host_charge_med_scale)
    } else if held >= t.host_charge_low_time {
        (1, t.host_charge_low_scale)
    } else {
        (0, 1.0)
    }
}

/// Tuning whose kick pushes out and up `scale` times harder. The push along the look is unchanged.
pub fn charged(t: &Tuning, scale: f32) -> Tuning {
    let mut out = *t;
    out.wall_kick_out_speed *= scale;
    out.wall_kick_up_speed *= scale;
    out
}
