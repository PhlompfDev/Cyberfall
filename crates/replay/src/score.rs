//! Pass criteria for a simulated recording.

use crate::drive::{coarse, SimTick};
use crate::recording::CoarseMode;

/// Speed error allowed, as a fraction of the recording's peak speed.
pub const MAX_SPEED_RMS: f32 = 0.03;
/// Final position error allowed, as a fraction of the recorded path length.
pub const MAX_POSITION_ERROR: f32 = 0.02;
pub const SPEED_ALIGN_STEPS: usize = 1;
/// How many steps early or late a mode change may be.
pub const MODE_TOLERANCE_STEPS: usize = 3;

#[derive(Clone, Debug, PartialEq)]
pub struct Score {
    /// RMS speed error over peak speed.
    pub speed_rms: f32,
    /// Distance between final positions over the recorded path length.
    pub position_error: f32,
    /// (step index, simulated, recorded) where the modes disagree.
    pub mode_mismatches: Vec<(usize, CoarseMode, CoarseMode)>,
}

impl Score {
    pub fn passed(&self) -> bool {
        self.speed_rms <= MAX_SPEED_RMS
            && self.position_error <= MAX_POSITION_ERROR
            && self.mode_mismatches.is_empty()
    }
}

pub fn score(ticks: &[SimTick]) -> Score {
    if ticks.is_empty() {
        return Score { speed_rms: 0.0, position_error: 0.0, mode_mismatches: Vec::new() };
    }
    let peak = ticks.iter().map(|t| t.recorded.velocity.length()).fold(0.0_f32, f32::max);
    // Recording and simulation can be a step out of phase.
    let mean_square = ticks
        .iter()
        .enumerate()
        .map(|(i, tick)| {
            let speed = tick.state.velocity.length();
            let lo = i.saturating_sub(SPEED_ALIGN_STEPS);
            let hi = (i + SPEED_ALIGN_STEPS).min(ticks.len() - 1);
            ticks[lo..=hi]
                .iter()
                .map(|near| (speed - near.recorded.velocity.length()).powi(2))
                .fold(f32::MAX, f32::min)
        })
        .sum::<f32>()
        / ticks.len() as f32;
    let speed_rms = if peak > 0.0 { mean_square.sqrt() / peak } else { mean_square.sqrt() };

    let path: f32 =
        ticks.windows(2).map(|w| (w[1].recorded.position - w[0].recorded.position).length()).sum();
    let last = ticks[ticks.len() - 1];
    let miss = (last.state.position - last.recorded.position).length();
    let position_error = if path > 0.0 { miss / path } else { miss };

    let mut mode_mismatches = Vec::new();
    for (i, tick) in ticks.iter().enumerate() {
        let simulated = coarse(tick.state.mode);
        let lo = i.saturating_sub(MODE_TOLERANCE_STEPS);
        let hi = (i + MODE_TOLERANCE_STEPS).min(ticks.len() - 1);
        if !ticks[lo..=hi].iter().any(|near| near.recorded.mode == simulated) {
            mode_mismatches.push((i, simulated, tick.recorded.mode));
        }
    }
    Score { speed_rms, position_error, mode_mismatches }
}
