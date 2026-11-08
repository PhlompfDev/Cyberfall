//! The gym's test course, in pilot-core coordinates (Z up, metres).

use glam::{Quat, Vec3};
use world_parry::BoxWorld;

/// Feet position the player starts and respawns at.
pub const SPAWN: Vec3 = Vec3::new(0.0, 0.0, 0.01);

pub fn build() -> BoxWorld {
    let mut world = BoxWorld::new();

    // Floor: top face at z = 0. The long clear strip runs along +X from spawn.
    world.add_box(Vec3::new(0.0, 0.0, -0.5), Vec3::new(160.0, 160.0, 0.5));

    // Wallrun lanes along +X: wall pairs 4, 6 and 8 metres apart.
    for (lane, gap) in [4.0_f32, 6.0, 8.0].into_iter().enumerate() {
        let lane_y = 20.0 + lane as f32 * 14.0;
        for side in [-1.0_f32, 1.0] {
            let wall_y = lane_y + side * (gap * 0.5 + 0.25);
            world.add_box(Vec3::new(45.0, wall_y, 6.0), Vec3::new(30.0, 0.25, 6.0));
        }
    }

    // Wallrun circuit: staggered panels on alternating sides of a 7 m lane.
    for panel in 0..8 {
        let side = if panel % 2 == 0 { -1.0 } else { 1.0 };
        let centre = Vec3::new(20.0 + panel as f32 * 9.0, -60.0 + side * 3.75, 6.0);
        world.add_box(centre, Vec3::new(5.0, 0.25, 6.0));
    }

    // Mantle ledges: near faces at x = 15, tops at four heights.
    for (i, top) in [1.0_f32, 1.5, 2.0, 2.5].into_iter().enumerate() {
        let centre = Vec3::new(18.0, -12.0 - i as f32 * 8.0, top * 0.5);
        world.add_box(centre, Vec3::new(3.0, 3.0, top * 0.5));
    }

    // Ramps rising toward -X at 15, 30 and 40 degrees.
    for (i, degrees) in [15.0_f32, 30.0, 40.0].into_iter().enumerate() {
        let tilt = Quat::from_rotation_y(degrees.to_radians());
        let centre = Vec3::new(-25.0, -12.0 + i as f32 * 12.0, 0.0);
        world.add_rotated_box(centre, Vec3::new(14.0, 4.0, 0.5), tilt);
    }

    // Stairs climbing toward +Y: five 0.3 m steps, each 0.8 m deep.
    for stair in 0..5 {
        let height = 0.3 * (stair + 1) as f32;
        let centre = Vec3::new(-10.0, 60.0 + 0.4 + stair as f32 * 0.8, height * 0.5);
        world.add_box(centre, Vec3::new(3.0, 0.4, height * 0.5));
    }

    world
}

#[cfg(test)]
mod tests {
    use super::*;
    use pilot_core::{step, Button, Input, Mode, PilotState};
    use std::path::PathBuf;

    fn tuning() -> pilot_core::Tuning {
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../tuning/pilot.toml");
        pilot_tuning::load(&path).unwrap()
    }

    #[test]
    fn the_course_has_every_kind_of_obstacle() {
        let world = build();
        let boxes: Vec<_> = world.boxes().collect();
        assert!(boxes.len() >= 20, "found {} boxes", boxes.len());
        assert!(boxes.iter().all(|b| b.half_extents.min_element() > 0.0));
        assert!(boxes.iter().any(|b| b.rotation != glam::Quat::IDENTITY), "ramps are rotated boxes");
    }

    #[test]
    fn the_spawn_point_is_stable_ground() {
        let t = tuning();
        let world = build();
        let mut state = PilotState::spawn(SPAWN, &t);
        for _ in 0..120 {
            state = step(&state, &Input::default(), &t, &world, 1.0 / 60.0).0;
        }
        assert_eq!(state.mode, Mode::Grounded);
        assert!((state.position - SPAWN).length() < 0.05, "drifted to {:?}", state.position);
    }

    #[test]
    fn sprinting_across_the_course_stays_sane() {
        let t = tuning();
        let world = build();
        let mut state = PilotState::spawn(SPAWN, &t);
        let input = Input { move_forward: 1.0, sprint: Button::HELD, ..Input::default() };
        for _ in 0..600 {
            state = step(&state, &input, &t, &world, 1.0 / 60.0).0;
            assert!(state.position.is_finite() && state.velocity.is_finite());
            assert!(state.position.z > -0.05);
        }
        assert!(state.position.x > 20.0, "it should have run a long way, x {}", state.position.x);
    }
}
