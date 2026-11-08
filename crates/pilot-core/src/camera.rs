use crate::kinematics::move_toward;
use crate::state::PilotState;
use crate::tuning::Tuning;
use crate::types::{Input, Mode};

/// Ease the camera outputs toward what the current mode calls for.
pub fn update(s: &mut PilotState, input: &Input, t: &Tuning, dt: f32) {
    let eye_target = if s.mode == Mode::Sliding {
        t.slide_eye_height
    } else if s.crouched {
        t.crouch_eye_height
    } else {
        t.stand_eye_height
    };
    s.camera.eye_height = move_toward(s.camera.eye_height, eye_target, t.eye_height_rate * dt);

    let roll_target = if s.mode == Mode::Wallrunning {
        // +1 with the wall square on the right, -1 on the left, 0 facing it.
        let wall_side = input.right().dot(-s.wall_normal).clamp(-1.0, 1.0);
        -wall_side * t.wallrun_roll_deg.to_radians()
    } else {
        0.0
    };
    s.camera.roll = move_toward(s.camera.roll, roll_target, t.roll_rate_deg.to_radians() * dt);
}
