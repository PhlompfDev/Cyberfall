use cyberfall_bridge::charge::tier_for_hold;
use cyberfall_bridge::{Bridge, ENDED_KICKED};

const DT: f32 = 1.0 / 60.0;

/// V running along +X on a wall whose face looks +Y.
fn on_wall(has_charge_jump: bool) -> Bridge {
    let mut b = Bridge::new();
    b.set_velocity(8.0, -1.0, 0.0);
    b.set_reading(true, 0.0, 1.0, 0.0, 0.0, 3);
    b.set_input(1.0, 0.0, 1.0, 0.0, false, false);
    b.set_charge(has_charge_jump, false);
    assert!(b.step(true, DT));
    b
}

/// One frame along the wall with the given buttons.
fn frame(b: &mut Bridge, jump_pressed: bool, jump_held: bool, crouch: bool) -> bool {
    b.set_velocity(8.0, 0.0, 0.0);
    b.set_reading(true, 0.0, 1.0, 0.0, 0.0, 3);
    b.set_input(1.0, 0.0, 1.0, 0.0, jump_pressed, crouch);
    b.set_charge(true, jump_held);
    b.step(true, DT)
}

/// Press Jump, then keep holding it for `frames` more frames.
fn charge_for(b: &mut Bridge, frames: usize) {
    assert!(frame(b, true, true, false), "a held press doesn't kick yet");
    for _ in 0..frames {
        assert!(frame(b, false, true, false), "still on the wall while charging");
    }
}

#[test]
fn tiers_follow_the_hold_time() {
    let t = *Bridge::new().tuning();
    assert_eq!(tier_for_hold(0.1, &t), (0, 1.0));
    assert_eq!(tier_for_hold(0.15, &t), (1, 1.15));
    assert_eq!(tier_for_hold(0.25, &t), (2, 1.3));
    assert_eq!(tier_for_hold(0.3, &t), (3, 1.5));
    assert_eq!(tier_for_hold(0.5, &t), (3, 1.5));
}

#[test]
fn without_charge_jump_the_kick_fires_on_press() {
    let mut b = on_wall(false);
    b.set_velocity(8.0, 0.0, 0.0);
    b.set_input(1.0, 0.0, 1.0, 0.0, true, false);
    b.set_charge(false, true);
    assert!(!b.step(true, DT));
    assert_eq!(b.ended(), ENDED_KICKED);
    assert_eq!(b.charge_tier(), 0);
    assert!(!b.charging());
}

#[test]
fn with_charge_jump_a_held_press_keeps_running() {
    let mut b = on_wall(true);
    assert!(frame(&mut b, true, true, false));
    assert!(b.charging());
    assert_ne!(b.ended(), ENDED_KICKED);
}

#[test]
fn a_full_charge_kicks_harder() {
    let mut b = on_wall(true);
    charge_for(&mut b, 20);
    assert!(!frame(&mut b, false, false, false));
    assert_eq!(b.ended(), ENDED_KICKED);
    assert_eq!(b.charge_tier(), 3);
    let t = *b.tuning();
    assert!((b.velocity().z - t.wall_kick_up_speed * 1.5).abs() < 1e-3, "vz {}", b.velocity().z);
    assert!((b.velocity().y - t.wall_kick_out_speed * 1.5).abs() < 1e-3, "vy {}", b.velocity().y);
    assert!(!b.charging());
}

#[test]
fn a_tap_with_charge_jump_is_a_normal_kick() {
    let mut b = on_wall(true);
    assert!(frame(&mut b, true, true, false));
    assert!(!frame(&mut b, false, false, false));
    assert_eq!(b.ended(), ENDED_KICKED);
    assert_eq!(b.charge_tier(), 0);
    assert!((b.velocity().z - b.tuning().wall_kick_up_speed).abs() < 1e-4);
}

#[test]
fn a_press_and_release_in_one_frame_is_a_normal_kick() {
    let mut b = on_wall(true);
    assert!(!frame(&mut b, true, false, false));
    assert_eq!(b.ended(), ENDED_KICKED);
    assert_eq!(b.charge_tier(), 0);
}

#[test]
fn letting_go_of_the_wall_while_charging_still_kicks() {
    let mut b = on_wall(true);
    charge_for(&mut b, 14);
    assert!(!frame(&mut b, false, true, true));
    assert_eq!(b.ended(), ENDED_KICKED);
    assert_eq!(b.charge_tier(), 2);
    assert!((b.velocity().z - b.tuning().wall_kick_up_speed * 1.3).abs() < 1e-3);
}

#[test]
fn landing_clears_a_charge() {
    let mut b = on_wall(true);
    charge_for(&mut b, 5);
    b.touched_ground();
    assert!(!b.charging());
}
