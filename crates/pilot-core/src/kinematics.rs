use glam::Vec3;

pub fn horizontal(v: Vec3) -> Vec3 {
    Vec3::new(v.x, v.y, 0.0)
}

/// Linear acceleration toward `wish_dir`, up to `wish_speed`.
pub fn accelerate(velocity: Vec3, wish_dir: Vec3, wish_speed: f32, accel: f32, dt: f32) -> Vec3 {
    let add = wish_speed - velocity.dot(wish_dir);
    if add <= 0.0 {
        return velocity;
    }
    velocity + wish_dir * (accel * dt).min(add)
}

/// Ground steering: full push toward `wish_dir`, horizontal speed capped at `limit`.
pub fn push(velocity: Vec3, wish_dir: Vec3, accel: f32, limit: f32, dt: f32) -> Vec3 {
    let pushed = velocity + wish_dir * (accel * dt);
    let flat = horizontal(pushed);
    if flat.length() <= limit {
        return pushed;
    }
    let held = flat.normalize() * limit;
    Vec3::new(held.x, held.y, pushed.z)
}

/// Air control. The small `wish_cap` is what makes air-strafing work.
pub fn air_accelerate(velocity: Vec3, wish_dir: Vec3, wish_cap: f32, accel: f32, dt: f32) -> Vec3 {
    accelerate(velocity, wish_dir, wish_cap, accel, dt)
}

/// Constant horizontal deceleration.
pub fn apply_friction(velocity: Vec3, decel: f32, dt: f32) -> Vec3 {
    let flat = horizontal(velocity);
    let speed = flat.length();
    if speed <= 0.0 {
        return velocity;
    }
    let scale = (speed - decel * dt).max(0.0) / speed;
    Vec3::new(flat.x * scale, flat.y * scale, velocity.z)
}

/// Remove the part of `velocity` that points into a surface.
pub fn clip(velocity: Vec3, normal: Vec3) -> Vec3 {
    velocity - normal * velocity.dot(normal)
}

pub fn move_toward(current: f32, target: f32, max_delta: f32) -> f32 {
    let gap = target - current;
    if gap.abs() <= max_delta {
        target
    } else {
        current + gap.signum() * max_delta
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use glam::Vec3;

    const DT: f32 = 1.0 / 60.0;

    #[test]
    fn horizontal_drops_only_z() {
        assert_eq!(horizontal(Vec3::new(1.0, 2.0, 3.0)), Vec3::new(1.0, 2.0, 0.0));
    }

    #[test]
    fn accelerate_adds_accel_per_second_whatever_the_wish_speed() {
        let slow = accelerate(Vec3::ZERO, Vec3::X, 4.0, 90.0, DT);
        let fast = accelerate(Vec3::ZERO, Vec3::X, 7.0, 90.0, DT);
        assert!((slow.x - 90.0 * DT).abs() < 1e-6);
        assert_eq!(slow, fast, "the rate does not depend on the target speed");
    }

    #[test]
    fn accelerate_reaches_wish_speed_and_never_passes_it() {
        let mut v = Vec3::ZERO;
        for _ in 0..600 {
            v = accelerate(v, Vec3::X, 5.0, 10.0, DT);
            assert!(v.x <= 5.0 + 1e-5);
        }
        assert!((v.x - 5.0).abs() < 1e-5);
    }

    #[test]
    fn accelerate_leaves_a_faster_velocity_alone() {
        let fast = Vec3::new(8.0, 0.0, 0.0);
        assert_eq!(accelerate(fast, Vec3::X, 5.0, 10.0, DT), fast);
    }

    #[test]
    fn air_accelerate_from_rest_stops_at_the_cap() {
        let mut v = Vec3::ZERO;
        for _ in 0..600 {
            v = air_accelerate(v, Vec3::X, 1.5, 12.0, DT);
            assert!(v.x <= 1.5 + 1e-5);
        }
        assert!((v.x - 1.5).abs() < 1e-5);
    }

    #[test]
    fn air_accelerate_sideways_adds_speed_to_a_fast_velocity() {
        let fast = Vec3::new(6.0, 0.0, 0.0);
        let v = air_accelerate(fast, Vec3::Y, 1.5, 12.0, DT);
        assert!((v.y - 12.0 * DT).abs() < 1e-6);
        assert_eq!(v.x, 6.0);
        // Pushing along the velocity, already above the cap, adds nothing.
        assert_eq!(air_accelerate(fast, Vec3::X, 1.5, 12.0, DT), fast);
    }

    #[test]
    fn friction_is_a_constant_deceleration_at_any_speed() {
        let fast = apply_friction(Vec3::new(9.0, 0.0, -3.0), 30.0, DT);
        let slow = apply_friction(Vec3::new(2.0, 0.0, 0.0), 30.0, DT);
        assert!((9.0 - fast.x - 30.0 * DT).abs() < 1e-5);
        assert!((2.0 - slow.x - 30.0 * DT).abs() < 1e-5);
        assert_eq!(fast.z, -3.0, "friction is horizontal only");
    }

    #[test]
    fn friction_keeps_the_direction_and_never_reverses() {
        let v = apply_friction(Vec3::new(3.0, 4.0, 0.0), 30.0, DT);
        assert!((v.y / v.x - 4.0 / 3.0).abs() < 1e-5);
        let crawl = apply_friction(Vec3::new(0.05, 0.0, 0.0), 30.0, DT);
        assert_eq!(crawl.x, 0.0);
        assert_eq!(apply_friction(Vec3::ZERO, 30.0, DT), Vec3::ZERO);
    }

    #[test]
    fn push_reaches_the_limit_and_holds_it() {
        let mut v = Vec3::ZERO;
        for _ in 0..30 {
            v = push(v, Vec3::X, 60.0, 5.0, DT);
            assert!(v.length() <= 5.0 + 1e-5);
        }
        assert!((v.x - 5.0).abs() < 1e-5);
        let first = push(Vec3::ZERO, Vec3::X, 60.0, 5.0, DT);
        assert!((first.x - 60.0 * DT).abs() < 1e-6);
    }

    #[test]
    fn push_sideways_turns_without_adding_speed() {
        let running = Vec3::new(5.0, 0.0, -1.0);
        let v = push(running, Vec3::Y, 60.0, 5.0, DT);
        assert!((horizontal(v).length() - 5.0).abs() < 1e-5, "speed is held");
        let turned = (v.y / v.x).atan();
        assert!((turned - (60.0 * DT / 5.0_f32).atan()).abs() < 1e-5, "the whole push went sideways");
        assert_eq!(v.z, -1.0);
    }

    #[test]
    fn clip_removes_the_component_into_the_surface() {
        let v = clip(Vec3::new(3.0, 4.0, -5.0), Vec3::Z);
        assert_eq!(v, Vec3::new(3.0, 4.0, 0.0));
    }

    #[test]
    fn move_toward_steps_without_overshooting() {
        assert_eq!(move_toward(0.0, 1.0, 0.25), 0.25);
        assert_eq!(move_toward(0.9, 1.0, 0.25), 1.0);
        assert_eq!(move_toward(1.0, 0.0, 0.25), 0.75);
    }
}
