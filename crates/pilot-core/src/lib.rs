//! Titanfall 2 pilot movement. No I/O, no host types. Z-up, metres, seconds.
//! See docs/host-interface.md.

pub mod types;
pub mod world;

pub use types::{Button, CameraOut, Event, Input, Mode, UP};
pub use world::{Capsule, Ground, Hit, World};

pub mod state;
pub mod tuning;

pub use state::PilotState;
pub use tuning::Tuning;
pub mod kinematics;
pub mod collide;

pub mod body;
pub mod grounded;
mod step;
pub mod transitions;

pub use step::step;
pub use transitions::{next_mode, Trigger};
pub mod airborne;
pub mod slide;
pub mod wallrun;
pub mod mantle;
pub mod wallhang;
pub mod camera;
pub mod wall;
pub mod settings;
