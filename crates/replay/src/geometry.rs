//! The world a recording was made in, rebuilt as boxes.

use std::fs;
use std::path::Path;

use glam::{Quat, Vec3};
use serde::Deserialize;
use world_parry::BoxWorld;

use crate::recording::{CoarseMode, Recording};

const FLOOR_HALF_SIZE: f32 = 400.0;
const FLOOR_THICKNESS: f32 = 1.0;

/// A hand-written description of what a manoeuvre touched, in metres.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Sidecar {
    /// Height of the floor's top face. Defaults to the recording's own.
    pub floor_z: Option<f32>,
    /// Seconds from the start of the take before which it is ignored.
    pub start: Option<f32>,
    /// Seconds from the start after which the take is ignored.
    pub end: Option<f32>,
    #[serde(default, rename = "box")]
    pub boxes: Vec<SidecarBox>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SidecarBox {
    pub centre: [f32; 3],
    pub half_extents: [f32; 3],
    #[serde(default)]
    pub yaw_deg: f32,
}

/// The lowest height the player stood at: where the floor is.
pub fn floor_height(recording: &Recording) -> Option<f32> {
    recording
        .ticks
        .iter()
        .filter(|tick| tick.mode == CoarseMode::Ground)
        .map(|tick| tick.position.z)
        .min_by(f32::total_cmp)
}

/// A floor under the recording plus any boxes the sidecar adds.
pub fn build(recording: &Recording, sidecar: Option<&Sidecar>, gap: f32) -> Result<BoxWorld, String> {
    let floor_z = sidecar
        .and_then(|s| s.floor_z)
        .or_else(|| floor_height(recording))
        .ok_or("the recording never touches the ground and no floor_z was given")?;
    let start = recording.ticks.first().map(|t| t.position).unwrap_or(Vec3::ZERO);
    let mut world = BoxWorld::new();
    world.add_box(
        Vec3::new(start.x, start.y, floor_z - gap - FLOOR_THICKNESS),
        Vec3::new(FLOOR_HALF_SIZE, FLOOR_HALF_SIZE, FLOOR_THICKNESS),
    );
    for b in sidecar.map(|s| s.boxes.as_slice()).unwrap_or_default() {
        world.add_rotated_box(
            Vec3::from_array(b.centre),
            Vec3::from_array(b.half_extents),
            Quat::from_rotation_z(b.yaw_deg.to_radians()),
        );
    }
    Ok(world)
}

pub fn load_sidecar(path: &Path) -> Result<Sidecar, String> {
    let text = fs::read_to_string(path).map_err(|e| format!("{}: {e}", path.display()))?;
    toml::from_str(&text).map_err(|e| format!("{}: {e}", path.display()))
}
