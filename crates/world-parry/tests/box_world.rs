use glam::{Quat, Vec3};
use pilot_core::{Capsule, World, UP};
use world_parry::BoxWorld;

const CAPSULE: Capsule = Capsule { radius: 0.35, half_height: 0.55 };

/// A floor whose top face is the plane z = 0.
fn floor() -> BoxWorld {
    let mut world = BoxWorld::new();
    world.add_box(Vec3::new(0.0, 0.0, -0.5), Vec3::new(100.0, 100.0, 0.5));
    world
}

#[test]
fn a_falling_capsule_hits_the_floor_at_its_bottom() {
    let world = floor();
    let from = Vec3::new(0.0, 0.0, 2.9);
    let hit = world.sweep(CAPSULE, from, from - UP * 4.0).unwrap();
    // The bottom starts 2.0 above the floor and the path is 4.0 long.
    assert!((hit.fraction - 0.5).abs() < 1e-3, "fraction {}", hit.fraction);
    assert!((hit.normal - UP).length() < 1e-3, "normal {:?}", hit.normal);
    assert!(hit.point.z.abs() < 1e-3, "point {:?}", hit.point);
}

#[test]
fn a_sweep_that_stops_short_reports_nothing() {
    let world = floor();
    let from = Vec3::new(0.0, 0.0, 2.9);
    assert!(world.sweep(CAPSULE, from, from - UP * 1.9).is_none());
    assert!(world.sweep(CAPSULE, from, from - UP * 2.1).is_some());
}

#[test]
fn a_wall_reports_a_normal_facing_the_capsule() {
    let mut world = floor();
    // Wall whose near face is the plane x = 5.
    world.add_box(Vec3::new(5.5, 0.0, 2.0), Vec3::new(0.5, 50.0, 2.0));
    let from = Vec3::new(0.0, 0.0, 1.0);
    let hit = world.sweep(CAPSULE, from, from + Vec3::X * 10.0).unwrap();
    assert!((hit.normal - -Vec3::X).length() < 1e-3, "normal {:?}", hit.normal);
    let expected = (5.0 - CAPSULE.radius) / 10.0;
    assert!((hit.fraction - expected).abs() < 1e-3, "fraction {}", hit.fraction);
}

#[test]
fn the_nearest_of_several_boxes_wins() {
    let mut world = BoxWorld::new();
    world.add_box(Vec3::new(8.5, 0.0, 0.0), Vec3::new(0.5, 5.0, 5.0));
    world.add_box(Vec3::new(4.5, 0.0, 0.0), Vec3::new(0.5, 5.0, 5.0));
    let hit = world.sweep(CAPSULE, Vec3::ZERO, Vec3::X * 10.0).unwrap();
    assert!(hit.fraction < 0.5, "should hit the box at x = 4, got {}", hit.fraction);
}

#[test]
fn rays_report_point_fraction_and_normal() {
    let world = floor();
    let from = Vec3::new(1.0, 2.0, 3.0);
    let hit = world.raycast(from, from - UP * 6.0).unwrap();
    assert!((hit.fraction - 0.5).abs() < 1e-4);
    assert!((hit.point - Vec3::new(1.0, 2.0, 0.0)).length() < 1e-3);
    assert!((hit.normal - UP).length() < 1e-3);
    assert!(world.raycast(from, from - UP * 2.9).is_none());
}

#[test]
fn a_rotated_box_presents_a_tilted_surface() {
    let mut world = BoxWorld::new();
    let tilt = Quat::from_rotation_y(30f32.to_radians());
    world.add_rotated_box(Vec3::ZERO, Vec3::new(20.0, 20.0, 0.5), tilt);
    let from = Vec3::new(0.0, 0.0, 5.0);
    let hit = world.raycast(from, from - UP * 10.0).unwrap();
    let expected = tilt * UP;
    assert!((hit.normal - expected).length() < 1e-3, "normal {:?}", hit.normal);
    assert!(hit.normal.x > 0.4, "a +Y rotation tips the +X side down");
}

#[test]
fn boxes_are_listed_for_rendering() {
    let mut world = BoxWorld::new();
    world.add_box(Vec3::X, Vec3::ONE).add_box(Vec3::Y, Vec3::ONE);
    let centres: Vec<Vec3> = world.boxes().map(|b| b.centre).collect();
    assert_eq!(centres, vec![Vec3::X, Vec3::Y]);
}

#[test]
fn a_sweep_onto_a_rotated_box_reports_its_world_space_normal() {
    let mut world = BoxWorld::new();
    let tilt = Quat::from_rotation_y(30f32.to_radians());
    world.add_rotated_box(Vec3::ZERO, Vec3::new(20.0, 20.0, 0.5), tilt);
    let from = Vec3::new(0.0, 0.0, 5.0);
    let hit = world.sweep(CAPSULE, from, from - UP * 10.0).unwrap();
    assert!((hit.normal - tilt * UP).length() < 1e-3, "normal {:?}", hit.normal);
}

#[test]
fn sweeps_stay_accurate_against_very_large_boxes() {
    // parry's shape cast drifts by centimetres against a 400 m floor unless
    // the box is clipped to the sweep's neighbourhood first.
    let mut world = BoxWorld::new();
    world.add_box(Vec3::new(0.0, 0.0, -0.5), Vec3::new(200.0, 200.0, 0.5));
    let offset = CAPSULE.half_height + CAPSULE.radius;
    for (x, y) in [(0.0, 0.0), (0.3, 0.2), (57.0, -131.0), (-199.0, 199.0)] {
        for (gap, length) in [(0.3_f32, 1.0_f32), (0.05, 0.1), (2.0, 4.0)] {
            let from = Vec3::new(x, y, offset + gap);
            let hit = world.sweep(CAPSULE, from, from - UP * length).unwrap();
            let travelled = hit.fraction * length;
            assert!((travelled - gap).abs() < 1e-3, "at ({x}, {y}) gap {gap}: travelled {travelled}");
            assert!((hit.normal - UP).length() < 1e-2, "normal {:?}", hit.normal);
        }
    }
}

#[test]
fn a_sweep_far_from_a_box_misses_it() {
    let mut world = BoxWorld::new();
    world.add_box(Vec3::new(50.0, 0.0, 0.0), Vec3::new(1.0, 1.0, 1.0));
    assert!(world.sweep(CAPSULE, Vec3::ZERO, Vec3::X * 5.0).is_none());
    assert!(world.sweep(CAPSULE, Vec3::ZERO, Vec3::X * 60.0).is_some());
}
