mod common;

use common::*;
use glam::{Quat, Vec3};
use pilot_core::collide::{face_normal, free_distance, ground_at, is_wall, is_walkable, move_and_slide};
use pilot_core::UP;
use world_parry::BoxWorld;

#[test]
fn a_falling_capsule_stops_a_skin_above_the_floor() {
    let t = tuning();
    let world = flat_world();
    let cap = t.stand_capsule();
    let start = Vec3::new(0.0, 0.0, 3.0);
    let out = move_and_slide(&world, cap, start, Vec3::new(0.0, 0.0, -10.0), 1.0, &t);
    let expected = cap.centre_offset() + t.collision_skin;
    assert!((out.centre.z - expected).abs() < 1e-3, "centre z {}", out.centre.z);
    assert_eq!(out.hits.len(), 1);
    assert!((out.hits[0].normal - UP).length() < 1e-3);
    assert!(out.velocity.length() < 1e-4, "velocity into the floor is removed");
}

#[test]
fn an_unobstructed_move_covers_velocity_times_dt() {
    let t = tuning();
    let world = flat_world();
    let start = Vec3::new(0.0, 0.0, 5.0);
    let out = move_and_slide(&world, t.stand_capsule(), start, Vec3::new(3.0, -2.0, 0.0), 0.5, &t);
    assert!((out.centre - Vec3::new(1.5, -1.0, 5.0)).length() < 1e-5);
    assert!(out.hits.is_empty());
}

#[test]
fn sliding_along_a_wall_keeps_the_tangential_motion() {
    let t = tuning();
    let mut world = flat_world();
    // Near face is the plane x = 5.
    world.add_box(Vec3::new(5.5, 0.0, 2.0), Vec3::new(0.5, 50.0, 2.0));
    let cap = t.stand_capsule();
    let start = Vec3::new(0.0, 0.0, cap.centre_offset() + 0.05);
    let out = move_and_slide(&world, cap, start, Vec3::new(10.0, 10.0, 0.0), 1.0, &t);
    assert!((out.centre.x - (5.0 - cap.radius)).abs() < 0.02, "x {}", out.centre.x);
    assert!((out.centre.y - 10.0).abs() < 0.02, "y {}", out.centre.y);
    assert!(out.velocity.x.abs() < 1e-4);
    assert!((out.velocity.y - 10.0).abs() < 1e-4);
}

#[test]
fn an_inside_corner_stops_the_capsule_without_passing_either_wall() {
    let t = tuning();
    let mut world = flat_world();
    world.add_box(Vec3::new(5.5, 0.0, 2.0), Vec3::new(0.5, 50.0, 2.0));
    world.add_box(Vec3::new(0.0, 5.5, 2.0), Vec3::new(50.0, 0.5, 2.0));
    let cap = t.stand_capsule();
    let start = Vec3::new(0.0, 0.0, cap.centre_offset() + 0.05);
    let out = move_and_slide(&world, cap, start, Vec3::new(10.0, 12.0, 0.0), 1.0, &t);
    assert!(out.centre.x <= 5.0 - cap.radius + 1e-3);
    assert!(out.centre.y <= 5.0 - cap.radius + 1e-3);
    assert!(out.centre.x > 4.0 && out.centre.y > 4.0, "it should reach the corner");
}

#[test]
fn free_distance_is_clamped_by_obstacles_and_by_max() {
    let t = tuning();
    let world = flat_world();
    let cap = t.stand_capsule();
    let start = Vec3::new(0.0, 0.0, cap.centre_offset() + 0.3);
    let down = free_distance(&world, cap, start, -UP, 1.0, &t);
    assert!((down - (0.3 - t.collision_skin)).abs() < 1e-3, "down {down}");
    assert_eq!(free_distance(&world, cap, start, UP, 1.0, &t), 1.0);
}

#[test]
fn walkable_and_wall_thresholds_come_from_tuning() {
    let t = tuning();
    let tilted = |deg: f32| Vec3::new(deg.to_radians().sin(), 0.0, deg.to_radians().cos());
    assert!(is_walkable(tilted(t.max_slope_deg - 1.0), &t));
    assert!(!is_walkable(tilted(t.max_slope_deg + 1.0), &t));
    assert!(is_walkable(UP, &t));

    let leaning = |z: f32| Vec3::new((1.0 - z * z).sqrt(), 0.0, z);
    assert!(is_wall(leaning(t.wall_max_normal_z - 0.01), &t));
    assert!(is_wall(leaning(-(t.wall_max_normal_z - 0.01)), &t));
    assert!(!is_wall(leaning(t.wall_max_normal_z + 0.01), &t));
    assert!(!is_wall(-UP, &t), "a ceiling is not a wall");
    assert!(!is_wall(UP, &t), "a floor is not a wall");
}

#[test]
fn ground_at_reports_flat_ground_under_the_capsule() {
    let t = tuning();
    let world = flat_world();
    let cap = t.stand_capsule();
    let centre = Vec3::new(0.0, 0.0, cap.centre_offset() + 0.2);
    let ground = ground_at(&world, cap, centre, 0.5, &t).unwrap();
    assert!((ground.distance - 0.2).abs() < 1e-3);
    assert!((ground.normal - UP).length() < 1e-3);
    assert!(ground.point.z.abs() < 1e-3);
    assert!(ground_at(&world, cap, centre, 0.1, &t).is_none());
}

#[test]
fn ground_at_sees_a_step_corner_as_the_flat_top_it_belongs_to() {
    let t = tuning();
    let mut world = flat_world();
    // A block whose top is z = 1 and whose near edge is x = 0.
    world.add_box(Vec3::new(5.0, 0.0, 0.5), Vec3::new(5.0, 5.0, 0.5));
    let cap = t.stand_capsule();
    // Axis 0.2 short of the edge, so the rounded base lands on the corner.
    let centre = Vec3::new(-0.2, 0.0, 1.0 + cap.centre_offset() + 0.3);
    let contact = pilot_core::World::ground(&world, cap, centre, 1.0).unwrap();
    assert!(contact.normal.z < 0.95, "the raw contact normal should be tilted");
    let ground = ground_at(&world, cap, centre, 1.0, &t).unwrap();
    assert!((ground.normal - UP).length() < 1e-3, "normal {:?}", ground.normal);
    assert!((ground.point.z - 1.0).abs() < 1e-3);
}

#[test]
fn ground_at_reports_a_slope_by_its_real_normal() {
    let t = tuning();
    let mut world = BoxWorld::new();
    let tilt = Quat::from_rotation_y(30f32.to_radians());
    world.add_rotated_box(Vec3::ZERO, Vec3::new(30.0, 30.0, 0.5), tilt);
    let cap = t.stand_capsule();
    let ground = ground_at(&world, cap, Vec3::new(0.0, 0.0, 3.0), 5.0, &t).unwrap();
    assert!((ground.normal - tilt * UP).length() < 1e-2, "normal {:?}", ground.normal);
}

#[test]
fn face_normal_falls_back_when_nothing_is_below_the_contact() {
    let t = tuning();
    let world = BoxWorld::new();
    let (normal, point) = face_normal(&world, Vec3::ONE, Vec3::ONE + UP, Vec3::X, &t);
    assert_eq!(normal, Vec3::X);
    assert_eq!(point, Vec3::ONE);
}
