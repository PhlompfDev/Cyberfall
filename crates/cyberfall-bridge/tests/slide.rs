use cyberfall_bridge::slide::Slide;
use cyberfall_bridge::Bridge;
use pilot_core::slide::boosted_speed;

const DT: f32 = 1.0 / 60.0;

fn slide() -> Slide {
    Slide::new(Bridge::new().base_tuning())
}

/// Falling along +X at `speed` for two frames.
fn fall(s: &mut Slide, speed: f32, crouch_held: bool) {
    s.note_air(speed, 0.0, false, crouch_held, DT);
    s.note_air(speed, 0.0, false, crouch_held, DT);
}

fn frames_without_crouch(s: &mut Slide, seconds: f32) {
    let frames = (seconds / DT).round() as usize;
    for _ in 0..frames {
        s.note_air(8.0, 0.0, false, false, DT);
    }
}

/// A slide whose boost is on cooldown, so it starts at exactly `speed`.
fn sliding_unboosted(speed: f32) -> Slide {
    let mut s = slide();
    s.begin(1.0, 0.0, 0.0, false);
    s.end();
    s.begin(speed, 0.0, 1.0, false);
    s.set_input(0.0, 0.0, 1.0, 0.0, true);
    s
}

#[test]
fn a_crouched_fast_landing_wants_a_slide() {
    let mut s = slide();
    fall(&mut s, 8.0, true);
    assert!(s.wants_landing_slide());
}

#[test]
fn a_slow_landing_does_not_slide() {
    let mut s = slide();
    fall(&mut s, 4.0, true);
    assert!(!s.wants_landing_slide());
}

#[test]
fn no_crouch_no_slide() {
    let mut s = slide();
    fall(&mut s, 8.0, false);
    assert!(!s.wants_landing_slide());
}

#[test]
fn a_crouch_press_just_before_landing_counts() {
    let mut s = slide();
    s.note_air(8.0, 0.0, true, false, DT);
    frames_without_crouch(&mut s, 0.2);
    assert!(s.wants_landing_slide());
}

#[test]
fn a_crouch_press_too_early_does_not_count() {
    let mut s = slide();
    s.note_air(8.0, 0.0, true, false, DT);
    frames_without_crouch(&mut s, 0.3);
    assert!(!s.wants_landing_slide());
}

#[test]
fn landing_without_a_slide_forgets_the_fall() {
    let mut s = slide();
    fall(&mut s, 8.0, true);
    s.landed();
    assert!(!s.wants_landing_slide());
}

#[test]
fn a_landing_slide_keeps_the_speed_from_before_impact() {
    let mut s = slide();
    let t = *s.tuning();
    fall(&mut s, 6.0, true);
    s.begin(2.0, 0.0, 10.0, true);
    assert!(s.active());
    assert!((s.velocity().length() - boosted_speed(6.0, &t)).abs() < 1e-3, "{}", s.velocity().length());
    assert!((boosted_speed(6.0, &t) - 9.81).abs() < 1e-3);
    assert!(s.velocity().x > 0.0);
    assert!(!s.wants_landing_slide(), "the fall is used up");
}

#[test]
fn a_second_slide_inside_the_cooldown_does_not_boost() {
    let mut s = slide();
    s.begin(6.0, 0.0, 10.0, false);
    s.end();
    fall(&mut s, 6.0, true);
    s.begin(2.0, 0.0, 11.0, true);
    assert!((s.velocity().length() - 6.0).abs() < 1e-3, "{}", s.velocity().length());
}

#[test]
fn a_slide_after_the_cooldown_boosts_again() {
    let mut s = slide();
    s.begin(6.0, 0.0, 10.0, false);
    s.end();
    fall(&mut s, 6.0, true);
    s.begin(2.0, 0.0, 12.1, true);
    assert!((s.velocity().length() - 9.81).abs() < 1e-3, "{}", s.velocity().length());
}

#[test]
fn a_slide_slows_by_the_titanfall_curve() {
    let mut s = sliding_unboosted(10.0);
    assert!(s.step(10.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.1));
    let expected = 10.0 - (1.27 + 0.3567 * 10.0) * 0.1;
    assert!((s.velocity().length() - expected).abs() < 1e-3, "{}", s.velocity().length());
}

#[test]
fn small_collision_losses_do_not_slow_the_slide() {
    let mut s = sliding_unboosted(10.0);
    let mut reference = sliding_unboosted(10.0);
    s.step(9.8, 0.0, 0.0, 0.0, 0.0, 1.0, 0.1);
    reference.step(10.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.1);
    assert!((s.velocity().length() - reference.velocity().length()).abs() < 1e-4);
}

#[test]
fn hitting_something_does_slow_the_slide() {
    let mut s = sliding_unboosted(10.0);
    s.step(2.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.1);
    assert!(s.velocity().length() < 3.0, "{}", s.velocity().length());
}

#[test]
fn the_vertical_speed_passes_through() {
    let mut s = sliding_unboosted(10.0);
    s.step(10.0, 0.0, -1.5, 0.0, 0.0, 1.0, 0.1);
    assert_eq!(s.velocity().z, -1.5);
}

#[test]
fn a_slide_ends_below_the_exit_speed() {
    let mut s = sliding_unboosted(3.3);
    assert!(!s.step(3.3, 0.0, 0.0, 0.0, 0.0, 1.0, 0.1));
}

#[test]
fn a_slide_ends_when_crouch_is_let_go() {
    let mut s = sliding_unboosted(10.0);
    s.set_input(0.0, 0.0, 1.0, 0.0, false);
    assert!(!s.step(10.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.1));
}

#[test]
fn a_disabled_slide_never_takes_over() {
    let mut s = slide();
    s.set_settings(false, 1.0);
    fall(&mut s, 8.0, true);
    assert!(!s.enabled());
    assert!(!s.wants_landing_slide());
}

#[test]
fn the_speed_setting_scales_the_boost() {
    let base = *Bridge::new().base_tuning();
    let mut s = slide();
    s.set_settings(true, 0.5);
    assert!((s.tuning().slide_boost_speed - base.slide_boost_speed * 0.5).abs() < 1e-6);
    assert!((s.tuning().slide_boost_max_speed - base.slide_boost_max_speed * 0.5).abs() < 1e-6);
    assert_eq!(s.tuning().slide_decel, base.slide_decel);
    s.set_settings(true, 9.0);
    assert!((s.tuning().slide_boost_speed - base.slide_boost_speed * 2.0).abs() < 1e-6, "clamped to 200%");
}

#[test]
fn the_bridge_owns_a_slide() {
    let mut b = Bridge::new();
    b.slide_mut().set_settings(false, 1.0);
    assert!(!b.slide().enabled());
}

#[test]
fn vertical_speed_never_counts_as_slide_speed() {
    let mut s = sliding_unboosted(10.0);
    let mut reference = sliding_unboosted(10.0);
    s.step(10.0, 0.0, -6.0, 0.0, 0.0, 1.0, 0.1);
    reference.step(10.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.1);
    s.step(12.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.1);
    reference.step(12.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.1);
    assert!((s.velocity().length() - reference.velocity().length()).abs() < 1e-4, "{} vs {}", s.velocity().length(), reference.velocity().length());
}
