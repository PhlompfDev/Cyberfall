use glam::Vec3;
use serde::{Deserialize, Serialize};

use crate::types::UP;

/// Upright capsule. Full height is `2 * (half_height + radius)`.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct Capsule {
    pub radius: f32,
    pub half_height: f32,
}

impl Capsule {
    /// Distance from the feet to the capsule centre.
    pub fn centre_offset(&self) -> f32 {
        self.half_height + self.radius
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Hit {
    /// 0..1 along the requested path.
    pub fraction: f32,
    pub point: Vec3,
    /// Unit normal of the surface hit, pointing away from it.
    pub normal: Vec3,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Ground {
    pub normal: Vec3,
    /// How far below the capsule the surface is, metres.
    pub distance: f32,
    /// Where the capsule would touch it.
    pub point: Vec3,
}

/// The world as pilot-core sees it. `sweep` and `ground` take capsule centres.
pub trait World {
    /// First thing the capsule touches moving from `from` to `to`.
    fn sweep(&self, capsule: Capsule, from: Vec3, to: Vec3) -> Option<Hit>;

    /// First thing a ray from `from` to `to` touches.
    fn raycast(&self, from: Vec3, to: Vec3) -> Option<Hit>;

    /// Surface under the capsule within `max_drop`.
    fn ground(&self, capsule: Capsule, centre: Vec3, max_drop: f32) -> Option<Ground> {
        self.sweep(capsule, centre, centre - UP * max_drop)
            .map(|hit| Ground { normal: hit.normal, distance: hit.fraction * max_drop, point: hit.point })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct FloorAt(f32);

    impl World for FloorAt {
        fn sweep(&self, capsule: Capsule, from: Vec3, to: Vec3) -> Option<Hit> {
            let bottom_from = from.z - capsule.centre_offset();
            let bottom_to = to.z - capsule.centre_offset();
            if bottom_from < self.0 || bottom_to > self.0 {
                return None;
            }
            let fraction = (bottom_from - self.0) / (bottom_from - bottom_to);
            Some(Hit { fraction, point: from.lerp(to, fraction), normal: UP })
        }

        fn raycast(&self, _from: Vec3, _to: Vec3) -> Option<Hit> {
            None
        }
    }

    #[test]
    fn centre_offset_is_half_the_full_height() {
        let capsule = Capsule { radius: 0.25, half_height: 0.5 };
        assert_eq!(capsule.centre_offset(), 0.75);
    }

    #[test]
    fn default_ground_reports_the_drop_distance() {
        let capsule = Capsule { radius: 0.25, half_height: 0.5 };
        let world = FloorAt(0.0);
        // A drop limit other than 1.0, so fraction and distance differ.
        let ground = world.ground(capsule, Vec3::new(0.0, 0.0, 1.0), 2.0).unwrap();
        assert!((ground.distance - 0.25).abs() < 1e-6);
        assert_eq!(ground.normal, UP);
        assert!(world.ground(capsule, Vec3::new(0.0, 0.0, 1.0), 0.2).is_none());
    }
}
