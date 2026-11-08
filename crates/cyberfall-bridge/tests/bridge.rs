
use cyberfall_bridge::{Bridge, ENDED_KICKED, ENDED_LOST, ENDED_NONE};
use glam::Vec3;

/// V moving along +X, 1 m/s into a wall on its right whose face looks +Y.
fn touching(b: &mut Bridge) {
    b.set_velocity(8.0, -1.0, 0.0);
    b.set_reading(true, 0.0, 1.0, 0.0, 0.0, 3);
    b.set_input(1.0, 0.0, 1.0, 0.0, false, false);
}

#[test]
fn the_embedded_tuning_is_the_repo_tuning() {
    let b = Bridge::new();
    let repo = pilot_tuning::load(&std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../tuning/pilot.toml")).unwrap();
    assert_eq!(b.base_tuning(), &repo);
}

#[test]
fn a_facing_vector_becomes_a_yaw() {
    let mut b = Bridge::new();
    b.set_input(1.0, 0.0, 0.0, 1.0, false, false);
    assert!((b.input_yaw() - std::f32::consts::FRAC_PI_2).abs() < 1e-6);
}

#[test]
fn a_staged_step_starts_a_run_and_reports_it() {
    let mut b = Bridge::new();
    touching(&mut b);
    assert!(b.step(true, 1.0 / 60.0));
    assert!(b.running() && b.started() && b.refresh_air_moves());
    assert_eq!(b.ended(), ENDED_NONE);
    let glue = b.base_tuning().host_wall_glue_speed;
    assert!((b.velocity().y + glue).abs() < 1e-5, "V's own into-wall speed is replaced by the glue");
    assert!((b.velocity().x - 65f32.sqrt()).abs() < 1e-4, "the 1 m/s into the wall is turned along it");
}

#[test]
fn a_jump_on_the_wall_reports_a_kick() {
    let mut b = Bridge::new();
    touching(&mut b);
    b.step(true, 1.0 / 60.0);
    b.set_velocity(8.0, 0.0, 0.0);
    b.set_input(1.0, 0.0, 1.0, 0.0, true, false);
    assert!(!b.step(true, 1.0 / 60.0));
    assert_eq!(b.ended(), ENDED_KICKED);
    assert!(!b.started());
}

#[test]
fn switching_off_mid_run_ends_it_and_keeps_the_velocity() {
    let mut b = Bridge::new();
    touching(&mut b);
    b.step(true, 1.0 / 60.0);
    b.set_settings(false, true, 1.0, 1.0, 1.0, 1.0);
    b.set_velocity(8.0, 0.0, 0.5);
    assert!(!b.step(true, 1.0 / 60.0));
    assert_eq!(b.ended(), ENDED_LOST);
    assert_eq!(b.velocity(), Vec3::new(8.0, 0.0, 0.5));
    touching(&mut b);
    assert!(!b.step(true, 1.0 / 60.0), "no new run while switched off");
}

#[test]
fn settings_scale_the_run() {
    let mut b = Bridge::new();
    b.set_settings(true, true, 1.0, 0.5, 1.0, 1.0);
    assert!((b.tuning().wallrun_speed - b.base_tuning().wallrun_speed * 0.5).abs() < 1e-6);
    assert!(b.enabled() && b.needs_perk());
}

#[test]
fn time_off_the_wall_counts_only_while_airborne_and_off_the_wall() {
    let mut b = Bridge::new();
    touching(&mut b);
    b.step(true, 1.0 / 60.0);
    b.set_input(1.0, 0.0, 1.0, 0.0, false, true);
    b.step(true, 1.0 / 60.0);
    assert!(!b.running());
    b.advance_air(0.25);
    assert!((b.state().time_since_wall - 0.25).abs() < 1e-6);
    b.touched_ground();
    assert!(b.state().grounded_since_wall);
}

#[test]
fn the_camera_lean_eases_toward_the_wall_lean_and_back() {
    let mut b = Bridge::new();
    let rate = b.tuning().roll_rate_deg.to_radians();
    touching(&mut b);
    b.step(true, 1.0 / 60.0);
    let target = b.roll();
    assert!(target.abs() > 0.0, "a wall on the right leans the view");
    let first = b.camera_roll(0.01);
    assert!((first - target.signum() * rate * 0.01).abs() < 1e-6, "eases at the roll rate: {first}");
    for _ in 0..200 {
        b.camera_roll(0.01);
    }
    assert!((b.camera_roll(0.0) - target).abs() < 1e-6, "settles on the lean");
    b.set_input(1.0, 0.0, 1.0, 0.0, false, true);
    b.step(true, 1.0 / 60.0);
    for _ in 0..200 {
        b.camera_roll(0.01);
    }
    assert_eq!(b.camera_roll(0.0), 0.0, "levels out after the run");
}

#[test]
fn the_script_reads_the_distances_it_probes_with() {
    let b = Bridge::new();
    assert_eq!(b.ground_clearance(), b.tuning().wallrun_min_ground_clearance);
    assert_eq!(b.stick_distance(), b.tuning().wallrun_stick_distance);
}

#[test]
fn an_air_dash_ends_a_run_without_freeing_the_wall() {
    let mut b = Bridge::new();
    touching(&mut b);
    assert!(b.step(true, 1.0 / 60.0));
    b.air_dash();
    assert!(!b.running());
    assert!(!b.state().grounded_since_wall, "a dash is not a landing");
    // Straight back onto the same wall: the same-wall rule still holds.
    touching(&mut b);
    assert!(!b.step(true, 1.0 / 60.0));
}

#[test]
fn an_air_dash_off_the_wall_changes_nothing() {
    let mut b = Bridge::new();
    b.advance_air(0.5);
    let before = *b.state();
    b.air_dash();
    assert_eq!(*b.state(), before);
}

const DT: f32 = 1.0 / 60.0;

/// A run in progress on the +Y-facing wall.
fn on_wall() -> Bridge {
    let mut b = Bridge::new();
    touching(&mut b);
    assert!(b.step(true, DT));
    b
}

#[test]
fn a_kick_goes_less_high_than_titanfall() {
    let mut b = on_wall();
    let t = *b.base_tuning();
    b.set_input(1.0, 0.0, 1.0, 0.0, true, false);
    b.step(true, DT);
    assert_eq!(b.ended(), ENDED_KICKED);
    assert!(t.host_kick_up_scale < 1.0);
    assert!((b.velocity().z - t.wall_kick_up_speed * t.host_kick_up_scale).abs() < 1e-4, "vz {}", b.velocity().z);
}

#[test]
fn a_run_presses_v_into_the_wall() {
    let b = on_wall();
    let glue = b.base_tuning().host_wall_glue_speed;
    assert!(glue > 0.0);
    assert!((b.velocity().y + glue).abs() < 1e-4, "into-wall speed {}", -b.velocity().y);
}

#[test]
fn a_brief_gap_in_the_wall_does_not_end_a_run() {
    let mut b = on_wall();
    let grace = b.base_tuning().host_wall_lost_grace;
    let frames = (grace / DT) as i32 - 1;
    for _ in 0..frames {
        b.set_reading(false, 0.0, 0.0, 0.0, 0.0, 3);
        assert!(b.step(true, DT), "a pipe or seam is not the end of the wall");
    }
    // The wall comes back and the run carries on with a fresh grace period.
    b.set_reading(true, 0.0, 1.0, 0.0, 0.0, 3);
    assert!(b.step(true, DT));
    for _ in 0..frames {
        b.set_reading(false, 0.0, 0.0, 0.0, 0.0, 3);
        assert!(b.step(true, DT));
    }
}

#[test]
fn a_wall_that_stays_gone_ends_the_run_after_the_grace() {
    let mut b = on_wall();
    let grace = b.base_tuning().host_wall_lost_grace;
    let mut frames = 0;
    loop {
        b.set_reading(false, 0.0, 0.0, 0.0, 0.0, 3);
        if !b.step(true, DT) {
            break;
        }
        frames += 1;
        assert!(frames < 100);
    }
    assert_eq!(b.ended(), ENDED_LOST);
    assert!((frames as f32 * DT - grace).abs() <= DT * 1.5, "ran {} s on no wall", frames as f32 * DT);
}

#[test]
fn a_reading_too_far_away_counts_as_a_gap() {
    let mut b = on_wall();
    let far = b.stick_distance() * 2.0;
    b.set_reading(true, 0.0, 1.0, 0.0, far, 3);
    assert!(b.step(true, DT));
}

#[test]
fn a_hit_that_is_not_a_wall_counts_as_a_gap() {
    // The top of a pipe running along the wall: close, but facing mostly up.
    let mut b = on_wall();
    b.set_reading(true, 0.0, 0.5, 0.866, 0.0, 3);
    assert!(b.step(true, DT));
}

#[test]
fn the_kick_setting_scales_the_lower_cyberpunk_kick() {
    let mut b = Bridge::new();
    let t = *b.base_tuning();
    b.set_settings(true, true, 1.0, 1.0, 1.5, 1.0);
    let expected = t.wall_kick_up_speed * t.host_kick_up_scale * 1.5;
    assert!((b.tuning().wall_kick_up_speed - expected).abs() < 1e-4);
}

/// V's speed along the wall in `on_wall`'s run, from its velocity.
fn along(b: &Bridge) -> f32 {
    b.velocity().x
}

#[test]
fn a_bump_ahead_steers_v_out_around_it() {
    let mut b = on_wall();
    let t = *b.base_tuning();
    touching(&mut b);
    // Something sticking out 0.2 m from the wall, 0.5 m ahead of V.
    b.set_ahead(true, 0.5, 0.2);
    assert!(b.step(true, DT));
    let time_to_reach = 0.5 / along(&b);
    let expected = (0.2 / time_to_reach).min(t.host_bump_speed);
    assert!((b.velocity().y - expected).abs() < 1e-3, "out from the wall at {}", b.velocity().y);
}

#[test]
fn steering_out_is_capped() {
    let mut b = on_wall();
    let t = *b.base_tuning();
    touching(&mut b);
    b.set_ahead(true, 0.01, 0.5);
    b.step(true, DT);
    assert!((b.velocity().y - t.host_bump_speed).abs() < 1e-4);
}

#[test]
fn a_solid_wall_ahead_is_not_steered_around() {
    let mut b = on_wall();
    let glue = b.base_tuning().host_wall_glue_speed;
    touching(&mut b);
    b.set_ahead(true, 0.5, -1.0);
    b.step(true, DT);
    assert!((b.velocity().y + glue).abs() < 1e-4, "glued as usual");
}

#[test]
fn a_bump_v_already_clears_is_ignored() {
    let mut b = on_wall();
    let glue = b.base_tuning().host_wall_glue_speed;
    touching(&mut b);
    b.set_reading(true, 0.0, 1.0, 0.0, 0.25, 3);
    b.set_ahead(true, 0.5, 0.2);
    b.step(true, DT);
    assert!((b.velocity().y + glue).abs() < 1e-4, "nothing to steer around");
}

#[test]
fn the_ahead_probe_is_used_for_one_step_only() {
    let mut b = on_wall();
    let glue = b.base_tuning().host_wall_glue_speed;
    touching(&mut b);
    b.set_ahead(true, 0.5, 0.2);
    b.step(true, DT);
    touching(&mut b);
    b.step(true, DT);
    assert!((b.velocity().y + glue).abs() < 1e-4, "a stale probe does not steer");
}

#[test]
fn the_script_probes_ahead_as_far_as_the_bridge_expects() {
    let b = Bridge::new();
    let t = b.tuning();
    assert_eq!(b.bump_lookahead(), t.host_bump_lookahead);
    assert_eq!(b.bump_max_clearance(), t.host_bump_max_clearance);
}

const S45: f32 = std::f32::consts::FRAC_1_SQRT_2;

#[test]
fn an_angled_entry_keeps_its_speed() {
    let mut b = Bridge::new();
    b.set_velocity(8.0 * S45, -8.0 * S45, 0.0);
    b.set_reading(true, 0.0, 1.0, 0.0, 0.0, 3);
    b.set_input(1.0, 0.0, 1.0, 0.0, false, false);
    assert!(b.step(true, DT));
    assert!((along(&b) - 8.0).abs() < 1e-3, "along the wall at {}", along(&b));
}

#[test]
fn an_entry_keeps_the_speed_the_game_took_off_on_impact() {
    let mut b = Bridge::new();
    b.set_input(1.0, 0.0, 1.0, 0.0, false, false);
    b.set_velocity(8.0 * S45, -8.0 * S45, 0.0);
    b.set_reading(false, 0.0, 0.0, 0.0, 0.0, 3);
    assert!(!b.step(true, DT));
    // The game's collision has already clipped V against the wall.
    b.set_velocity(8.0 * S45, 0.0, 0.0);
    b.set_reading(true, 0.0, 1.0, 0.0, 0.0, 3);
    assert!(b.step(true, DT));
    assert!((along(&b) - 8.0).abs() < 1e-3, "along the wall at {}", along(&b));
}

#[test]
fn an_old_speed_is_not_carried_into_an_entry() {
    let mut b = Bridge::new();
    b.set_input(1.0, 0.0, 1.0, 0.0, false, false);
    b.set_velocity(14.0, 0.0, 0.0);
    b.set_reading(false, 0.0, 0.0, 0.0, 0.0, 3);
    b.step(true, DT);
    b.touched_ground();
    b.set_velocity(5.0, -1.0, 0.0);
    b.set_reading(true, 0.0, 1.0, 0.0, 0.0, 3);
    assert!(b.step(true, DT));
    assert!(along(&b) < 5.2, "landing forgets the speed: {}", along(&b));
}

#[test]
fn a_head_on_hit_still_does_not_start_a_run() {
    let mut b = Bridge::new();
    b.set_velocity(1.0, -8.0, 0.0);
    b.set_reading(true, 0.0, 1.0, 0.0, 0.0, 3);
    b.set_input(1.0, 0.0, 0.0, -1.0, false, false);
    assert!(!b.step(true, DT));
}

#[test]
fn keeping_entry_speed_leaves_the_entry_lift_alone() {
    let mut b = Bridge::new();
    let t = *b.tuning();
    b.set_velocity(8.0 * S45, -8.0 * S45, -2.0);
    b.set_reading(true, 0.0, 1.0, 0.0, 0.0, 3);
    b.set_input(1.0, 0.0, 1.0, 0.0, false, false);
    assert!(b.step(true, DT));
    let lifted = (-2.0 + t.wallrun_entry_up_speed).min(t.wallrun_entry_up_speed);
    assert!((b.velocity().z - lifted).abs() < 1e-4, "vz {}", b.velocity().z);
}

#[test]
fn touching_the_wall_just_left_gives_back_jumps_and_dashes() {
    let mut b = on_wall();
    b.set_input(1.0, 0.0, 1.0, 0.0, true, false);
    b.step(true, DT);
    assert_eq!(b.ended(), ENDED_KICKED);
    // Straight back onto the same wall: no new run, but the air moves return.
    touching(&mut b);
    assert!(!b.step(true, DT), "the same-wall rule still holds for runs");
    assert!(b.refresh_air_moves());
}

#[test]
fn a_wall_out_of_grab_reach_gives_nothing_back() {
    let mut b = on_wall();
    b.set_input(1.0, 0.0, 1.0, 0.0, true, false);
    b.step(true, DT);
    touching(&mut b);
    b.set_reading(true, 0.0, 1.0, 0.0, b.stick_distance(), 3);
    b.step(true, DT);
    assert!(!b.refresh_air_moves());
}

#[test]
fn a_touch_too_slow_to_run_still_gives_back_air_moves() {
    let mut b = Bridge::new();
    b.set_velocity(1.0, -1.0, 0.0);
    b.set_reading(true, 0.0, 1.0, 0.0, 0.0, 3);
    b.set_input(1.0, 0.0, 1.0, 0.0, false, false);
    assert!(!b.step(true, DT));
    assert!(b.refresh_air_moves());
}

#[test]
fn getting_shoved_off_a_wall_still_gives_back_air_moves() {
    // Touching the wall while moving away from it: no run, but the touch counts.
    let mut b = Bridge::new();
    b.set_velocity(6.0, 3.0, 0.0);
    b.set_reading(true, 0.0, 1.0, 0.0, 0.0, 3);
    b.set_input(1.0, 0.0, 1.0, 0.0, false, false);
    assert!(!b.step(true, DT));
    assert!(b.refresh_air_moves());
}

#[test]
fn a_low_barrier_gives_nothing_back() {
    let mut b = Bridge::new();
    b.set_velocity(1.0, -1.0, 0.0);
    b.set_reading(true, 0.0, 1.0, 0.0, 0.0, 1);
    b.set_input(1.0, 0.0, 1.0, 0.0, false, false);
    b.step(true, DT);
    assert!(!b.refresh_air_moves());
}

#[test]
fn a_slope_below_the_wall_just_left_gives_nothing_back() {
    let mut b = on_wall();
    b.set_input(1.0, 0.0, 1.0, 0.0, true, false);
    b.step(true, DT);
    touching(&mut b);
    b.set_reading(true, 0.0, 0.5, 0.866, 0.0, 3);
    b.step(true, DT);
    assert!(!b.refresh_air_moves());
}

#[test]
fn the_script_filters_walls_with_the_controllers_own_limit() {
    let b = Bridge::new();
    assert_eq!(b.wall_max_normal_z(), b.tuning().wall_max_normal_z);
}

#[test]
fn cyberpunk_grabs_walls_from_further_than_titanfalls_hull() {
    let b = Bridge::new();
    let t = b.base_tuning();
    assert!(t.host_grab_distance > t.wallrun_grab_distance);
    assert_eq!(b.tuning().wallrun_grab_distance, t.host_grab_distance);
    let mut b = Bridge::new();
    b.set_velocity(8.0, -1.0, 0.0);
    b.set_reading(true, 0.0, 1.0, 0.0, (t.wallrun_grab_distance + t.host_grab_distance) * 0.5, 3);
    b.set_input(1.0, 0.0, 1.0, 0.0, false, false);
    assert!(b.step(true, DT));
}

#[test]
fn a_low_wall_does_not_start_a_run() {
    // A concrete barrier: only the knee-height probe sees it.
    let mut b = Bridge::new();
    b.set_velocity(8.0, -1.0, 0.0);
    b.set_reading(true, 0.0, 1.0, 0.0, 0.0, 1);
    b.set_input(1.0, 0.0, 1.0, 0.0, false, false);
    assert!(!b.step(true, DT));
}

#[test]
fn a_wall_seen_at_two_heights_starts_a_run() {
    let mut b = Bridge::new();
    b.set_velocity(8.0, -1.0, 0.0);
    b.set_reading(true, 0.0, 1.0, 0.0, 0.0, b.tuning().host_wall_min_heights as i32);
    b.set_input(1.0, 0.0, 1.0, 0.0, false, false);
    assert!(b.step(true, DT));
}

#[test]
fn running_off_the_top_of_a_wall_ends_the_run() {
    let mut b = on_wall();
    let mut frames = 0;
    loop {
        b.set_reading(true, 0.0, 1.0, 0.0, 0.0, 1);
        if !b.step(true, DT) {
            break;
        }
        frames += 1;
        assert!(frames < 100);
    }
    assert_eq!(b.ended(), ENDED_LOST);
}
