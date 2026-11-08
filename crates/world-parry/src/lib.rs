//! A `World` made of boxes, backed by parry3d. Used by tests and the gym.

use glam::{Quat, Vec3};
use parry3d::na::{Isometry3, Point3, Quaternion, Translation3, UnitQuaternion, Vector3};
use parry3d::query::{cast_shapes, Ray, RayCast, ShapeCastOptions};
use parry3d::shape::{Capsule as ParryCapsule, Cuboid};
use pilot_core::{Capsule, Hit, World};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BoxDef {
    pub centre: Vec3,
    pub half_extents: Vec3,
    pub rotation: Quat,
}

struct Solid {
    def: BoxDef,
    pose: Isometry3<f32>,
    shape: Cuboid,
}

/// How far past the swept capsule a clipped box extends, metres.
const CLIP_MARGIN: f32 = 0.5;

impl Solid {
    /// This box clipped to the sweep's neighbourhood. parry's shape cast is
    /// centimetres off against very large boxes.
    fn clipped_to(&self, from: Vec3, to: Vec3, reach: f32) -> Option<(Isometry3<f32>, Cuboid)> {
        let a = self.pose.inverse_transform_point(&Point3::new(from.x, from.y, from.z));
        let b = self.pose.inverse_transform_point(&Point3::new(to.x, to.y, to.z));
        let half = self.shape.half_extents;
        let mut centre = Vector3::zeros();
        let mut clipped = Vector3::zeros();
        for axis in 0..3 {
            let lo = (a[axis].min(b[axis]) - reach).max(-half[axis]);
            let hi = (a[axis].max(b[axis]) + reach).min(half[axis]);
            if lo >= hi {
                return None;
            }
            centre[axis] = (lo + hi) * 0.5;
            clipped[axis] = (hi - lo) * 0.5;
        }
        let pose = self.pose * Translation3::from(centre);
        Some((pose, Cuboid::new(clipped)))
    }
}

#[derive(Default)]
pub struct BoxWorld {
    solids: Vec<Solid>,
}

fn to_na(v: Vec3) -> Vector3<f32> {
    Vector3::new(v.x, v.y, v.z)
}

fn from_na(v: Vector3<f32>) -> Vec3 {
    Vec3::new(v.x, v.y, v.z)
}

impl BoxWorld {
    pub fn new() -> BoxWorld {
        BoxWorld::default()
    }

    pub fn add_box(&mut self, centre: Vec3, half_extents: Vec3) -> &mut Self {
        self.add_rotated_box(centre, half_extents, Quat::IDENTITY)
    }

    pub fn add_rotated_box(&mut self, centre: Vec3, half_extents: Vec3, rotation: Quat) -> &mut Self {
        let pose = Isometry3::from_parts(
            Translation3::new(centre.x, centre.y, centre.z),
            UnitQuaternion::from_quaternion(Quaternion::new(rotation.w, rotation.x, rotation.y, rotation.z)),
        );
        let def = BoxDef { centre, half_extents, rotation };
        self.solids.push(Solid { def, pose, shape: Cuboid::new(to_na(half_extents)) });
        self
    }

    pub fn boxes(&self) -> impl Iterator<Item = &BoxDef> {
        self.solids.iter().map(|s| &s.def)
    }
}

impl World for BoxWorld {
    fn sweep(&self, capsule: Capsule, from: Vec3, to: Vec3) -> Option<Hit> {
        let shape = ParryCapsule::new_z(capsule.half_height, capsule.radius);
        let start = Isometry3::translation(from.x, from.y, from.z);
        // The velocity is the whole path, so time of impact is the fraction.
        let path = to_na(to - from);
        let still = Vector3::zeros();
        let options = ShapeCastOptions {
            max_time_of_impact: 1.0,
            target_distance: 0.0,
            stop_at_penetration: true,
            compute_impact_geometry_on_penetration: true,
        };
        let mut best: Option<Hit> = None;
        let reach = capsule.half_height + capsule.radius + CLIP_MARGIN;
        for solid in &self.solids {
            let Some((pose, cuboid)) = solid.clipped_to(from, to, reach) else {
                continue;
            };
            let Ok(Some(cast)) = cast_shapes(&start, &path, &shape, &pose, &still, &cuboid, options)
            else {
                continue;
            };
            if best.is_some_and(|b| b.fraction <= cast.time_of_impact) {
                continue;
            }
            let normal = pose.rotation * cast.normal2.into_inner();
            let point = pose * cast.witness2;
            best = Some(Hit {
                fraction: cast.time_of_impact,
                point: Vec3::new(point.x, point.y, point.z),
                normal: from_na(normal),
            });
        }
        best
    }

    fn raycast(&self, from: Vec3, to: Vec3) -> Option<Hit> {
        let ray = Ray::new(Point3::new(from.x, from.y, from.z), to_na(to - from));
        let mut best: Option<Hit> = None;
        for solid in &self.solids {
            let Some(cast) = solid.shape.cast_ray_and_get_normal(&solid.pose, &ray, 1.0, true) else {
                continue;
            };
            if best.is_some_and(|b| b.fraction <= cast.time_of_impact) {
                continue;
            }
            best = Some(Hit {
                fraction: cast.time_of_impact,
                point: from + (to - from) * cast.time_of_impact,
                normal: from_na(cast.normal),
            });
        }
        best
    }
}
