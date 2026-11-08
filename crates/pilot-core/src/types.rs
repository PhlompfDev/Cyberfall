use glam::Vec3;
use serde::{Deserialize, Serialize};

pub const UP: Vec3 = Vec3::Z;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Mode {
    Grounded,
    Sliding,
    Airborne,
    Wallrunning,
    WallHanging,
    Mantling,
}

/// `pressed` is true only on the tick the button went down.
#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Button {
    pub held: bool,
    pub pressed: bool,
}

impl Button {
    pub const UP: Button = Button { held: false, pressed: false };
    pub const HELD: Button = Button { held: true, pressed: false };
    pub const PRESSED: Button = Button { held: true, pressed: true };
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Input {
    /// -1..1, positive is forward.
    pub move_forward: f32,
    /// -1..1, positive is right.
    pub move_right: f32,
    /// Radians about +Z. 0 faces +X; increasing turns left.
    pub yaw: f32,
    /// Radians, positive looks up.
    pub pitch: f32,
    pub jump: Button,
    pub crouch: Button,
    pub sprint: Button,
    pub hang: Button,
}

impl Input {
    pub fn forward(&self) -> Vec3 {
        Vec3::new(self.yaw.cos(), self.yaw.sin(), 0.0)
    }

    pub fn right(&self) -> Vec3 {
        Vec3::new(self.yaw.sin(), -self.yaw.cos(), 0.0)
    }

    /// Unit horizontal direction the player is pushing toward, or zero.
    pub fn wish_dir(&self) -> Vec3 {
        (self.forward() * self.move_forward + self.right() * self.move_right).normalize_or_zero()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub enum Event {
    Jumped,
    DoubleJumped,
    Landed { impact_speed: f32 },
    SlideStarted,
    SlideEnded,
    WallrunStarted,
    WallrunEnded,
    WallKicked,
    MantleStarted,
    MantleEnded,
}

/// Values the host applies to its camera.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct CameraOut {
    /// Eye height above the feet, metres.
    pub eye_height: f32,
    /// Radians. Positive tilts the view clockwise (right ear down).
    pub roll: f32,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::f32::consts::FRAC_PI_2;

    fn close(a: Vec3, b: Vec3) -> bool {
        (a - b).length() < 1e-5
    }

    #[test]
    fn yaw_zero_faces_plus_x_with_right_at_minus_y() {
        let input = Input::default();
        assert!(close(input.forward(), Vec3::X));
        assert!(close(input.right(), -Vec3::Y));
    }

    #[test]
    fn increasing_yaw_turns_left() {
        let input = Input { yaw: FRAC_PI_2, ..Input::default() };
        assert!(close(input.forward(), Vec3::Y));
        assert!(close(input.right(), Vec3::X));
    }

    #[test]
    fn wish_dir_is_unit_length_on_diagonals_and_zero_at_rest() {
        let diagonal = Input { move_forward: 1.0, move_right: 1.0, ..Input::default() };
        assert!((diagonal.wish_dir().length() - 1.0).abs() < 1e-5);
        assert_eq!(Input::default().wish_dir(), Vec3::ZERO);
    }

    #[test]
    fn wish_dir_follows_the_right_axis() {
        let input = Input { move_right: 1.0, ..Input::default() };
        assert!(close(input.wish_dir(), -Vec3::Y));
    }
}
