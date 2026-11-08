//! Settings scale the Titanfall values and touch nothing else.

mod common;

use common::tuning;
use pilot_core::settings::{apply, WallSettings};

#[test]
fn titanfall_settings_change_nothing() {
    let t = tuning();
    assert_eq!(apply(&t, &WallSettings::TITANFALL), t);
}

#[test]
fn length_scales_only_the_time_limit() {
    let t = tuning();
    let mut expected = t;
    expected.wallrun_time_limit *= 1.5;
    assert_eq!(apply(&t, &WallSettings { length: 1.5, ..WallSettings::TITANFALL }), expected);
}

#[test]
fn speed_scales_only_the_wallrun_ceiling() {
    let t = tuning();
    let mut expected = t;
    expected.wallrun_speed *= 0.5;
    assert_eq!(apply(&t, &WallSettings { speed: 0.5, ..WallSettings::TITANFALL }), expected);
}

#[test]
fn kick_scales_only_the_three_kick_speeds() {
    let t = tuning();
    let mut expected = t;
    expected.wall_kick_out_speed *= 1.25;
    expected.wall_kick_up_speed *= 1.25;
    expected.wall_kick_look_speed *= 1.25;
    assert_eq!(apply(&t, &WallSettings { kick: 1.25, ..WallSettings::TITANFALL }), expected);
}

#[test]
fn tilt_scales_only_the_camera_lean() {
    let t = tuning();
    let mut expected = t;
    expected.wallrun_roll_deg = 0.0;
    assert_eq!(apply(&t, &WallSettings { tilt: 0.0, ..WallSettings::TITANFALL }), expected);
}

#[test]
fn scales_are_clamped() {
    let t = tuning();
    let wild = WallSettings { speed: 10.0, length: 0.01, kick: 2.0, tilt: -1.0, ..WallSettings::TITANFALL };
    let tame = WallSettings { speed: 2.0, length: 0.25, kick: 2.0, tilt: 0.0, ..WallSettings::TITANFALL };
    assert_eq!(apply(&t, &wild), apply(&t, &tame));
}

#[test]
fn the_switches_do_not_change_tuning() {
    let t = tuning();
    let off = WallSettings { enabled: false, needs_perk: false, ..WallSettings::TITANFALL };
    assert_eq!(apply(&t, &off), t);
}
