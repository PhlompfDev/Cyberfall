//! Replays recordings of real Titanfall 2 movement through pilot-core.

pub mod drive;
pub mod geometry;
pub mod recording;
pub mod score;

pub use drive::{simulate, SimTick};
pub use recording::{CoarseMode, Recording, Tick};
pub use score::{score, Score};

/// Metres per engine unit. The only place this conversion is defined.
pub const UNITS_TO_METRES: f32 = 0.0254;
